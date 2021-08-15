## MIT License
##
## Copyright (c) 2021 conveen
##
## Permission is hereby granted, free of charge, to any person obtaining a copy
## of this software and associated documentation files (the "Software"), to deal
## in the Software without restriction, including without limitation the rights
## to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
## copies of the Software, and to permit persons to whom the Software is
## furnished to do so, subject to the following conditions:
##
## The above copyright notice and this permission notice shall be included in all
## copies or substantial portions of the Software.
##
## THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
## IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
## FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
## AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
## LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
## OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
## SOFTWARE.

import logging
from string import Formatter
import typing
from urllib.parse import urlsplit, urlunsplit

from django.core.validators import URLValidator
from django.db import models, transaction


logger = logging.getLogger(__name__)


########################
##### Database API #####
########################


def _validate_netloc_url(url: str) -> str:
    """Validate URL according to RFC 1808.

    URL must be valid according to the RFC 1808 specification _and_
    point to a network location with the ``http`` or `https`` scheme.
    See https://datatracker.ietf.org/doc/html/rfc1808 for more information.
    If no scheme provided, will default to ``http`` (like most browsers).

    Raises:
        ValueError: if URL is invalid, or
                    if URL doesn't point to network location or
                    if URL uses a non-http(s) scheme.
    """
    # Ensure URL is valid netloc url according to RFC 1808
    # See: https://tools.ietf.org/html/rfc1808.html
    try:
        scheme, netloc, path, query, fragment = urlsplit(url)
    except ValueError as exc:
        raise ValueError(f"Invalid URL ({exc})") from exc
    # URL must point to network location (website)
    if not netloc:
        raise ValueError("URL must point to website")
    # If no scheme is provided, will default to http
    if not scheme:
        scheme = "http"
        url = urlunsplit(
            (
                scheme,
                netloc,
                path,
                query,
                fragment,
            )
        )
    elif scheme not in {"http", "https"}:
        raise ValueError(f"Invalid scheme {scheme}")
    return url


def _gen_num_args_from_url(url: str) -> int:
    """Parse number of position arguments from URL format string.

    Uses the built-in ``string.Formatter`` class to parse the number of
    positional arguments. Does not support keyword arguments.

    Raises:
        ValueError: if URL contains keyword arguments.
    """
    format_args = list(Formatter().parse(url))
    num_args = 0
    # If only one entry and field_name is None, then doesn't have any format args
    # Each entry in format_args is 4-tuple with (iteral_text, field_name, format_spec, conversion)
    # See: https://docs.python.org/3.6/library/string.html#string.Formatter.parse
    if len(format_args) != 1 or format_args[0][1] is not None:
        for _, field_name, __, ___ in format_args:
            if field_name:
                raise ValueError('must not have keyword arguments ("{}")'.format(field_name))
            if field_name == "":
                num_args += 1
    return num_args


class DestinationManager(models.Manager):
    """Destination objects manager."""

    def clear_default_fallbacks(self) -> None:
        """Remove the ``is_default_fallback`` flag (set to ``False``) for all destinations."""
        self.filter(is_default_fallback=True).update(is_default_fallback=False)

    def create_with_aliases(
        self,
        url: str,
        description: str,
        aliases: typing.List[str],
        is_fallback: bool = False,
        is_default_fallback: bool = False,
    ) -> "Destination":
        """Add destination with one or more aliases.

        Arguments must pass the following validation steps:
            * One or more unique aliases
            * URL must be valid according to the RFC 1808 specification and use the ``http`` or ``https`` schemes
            * URL must only contain positional formatting arguments, not keyword arguments
            * If the URl is a (default) fallback destination it must accept exactly one argument

        If the URL is a default fallback destination and one or more exist already,
        this function will remove the ``is_default_fallback`` flag on the existing
        destinations and set it for the new one.

        Raises:
            ValueError: if the URL is invalid (see: ``_validate_netloc_url``) or
                        if the URL contains keyword format arguments (see: ``_gen_num_args_from_url``) or
                        if the URL is a (default) fallback destination and doesn't have exactly one argument.
        """
        unique_aliases = {name for name in aliases if name}
        if not unique_aliases:
            raise ValueError("Must provide one or more non-empty aliases")

        url = _validate_netloc_url(url)
        num_args = _gen_num_args_from_url(url)

        # Must wrap this block in a transaction so that if the destination is a default fallback
        # and doesn't have exactly one argument, the transaction will be rolled back and
        # the existing default fallback will be maintained.
        # May explore alternative implementations later on to avoid the transaction overhead.
        with transaction.atomic():
            if is_default_fallback:
                is_fallback = True
                # Clear all existing default fallbacks so new destination is the only one
                self.clear_default_fallbacks()
            if is_fallback and num_args != 1:
                raise ValueError("Fallback destinations must have exactly one argument")

        destination = self.create(
            url=url,
            num_args=num_args,
            is_fallback=is_fallback,
            is_default_fallback=is_default_fallback,
            description=description,
        )
        Alias.objects.bulk_create([Alias(name=name, destination=destination) for name in unique_aliases])
        return destination

    def default_fallback(self) -> "Destination":
        """Get default fallback destination.

        A default fallback destination must exist in the database.
        If not, it's considered a database error. In the case multiple
        destinations are marked as default fallbacks, this function will
        get the first one returned in the query set.

        Raises:
            Destination.DoesNotExist: if no default fallback destination found.
        """
        default_fallback = self.filter(is_default_fallback=True).first()
        if not default_fallback:
            raise Destination.DoesNotExist()

        return default_fallback

    def from_alias(self, alias: str) -> typing.Optional["Destination"]:
        """Resolve destination for ``alias``, if it exists.

        Raises:
            Destination.MultipleObjectsReturned: if multiple destinations returned for an alias
                                                 (which should be impossible)
        """
        try:
            return self.filter(alias__name=alias).get()
        except Destination.DoesNotExist:
            return None
        except Destination.MultipleObjectsReturned:
            logger.warning("Multiple destinations returned for alias {}", alias)
            raise


######################################
##### Database Schema Definition #####
######################################


class HealthCheck(models.Model):
    """database_health_check table.

    Used by the health check API endpoint to check the database connection for reads and writes.
    """

    check_field = models.BooleanField()

    class Meta:
        db_table = "database_health_check"


class Destination(models.Model):
    """destination table.

    A destination represents a unique _URL_ and set of zero or more parameters.
    For example, a destination could be https://www.youtube.com/results?search_query={},
    where '{}' is a single parameter. Destinations could also have no
    parameters, like https://time.is/, or have parameters as part of the URL (not a URL parameter).

    Fallback destinations are URLs with a single parameter, and are used if the specified
    destination does not exist, or there is otherwise an error in redirecting an a destination.
    Common choices could be DuckDuckGo, Wikipedia, or Google.
    """

    url = models.CharField(max_length=2000, unique=True, validators=[URLValidator()])
    num_args = models.IntegerField()
    is_fallback = models.BooleanField(db_index=True)
    is_default_fallback = models.BooleanField(db_index=True, default=False)
    description = models.TextField()

    objects = DestinationManager()

    class Meta:
        db_table = "destination"


class Alias(models.Model):
    """alias table.

    Aliases act as references, or pointers, to destinations, and are the shortcuts users type to
    get redirected to a destination. For example, the shortcuts "ddg", "duckduckgo", and "go" could
    all reference a search on DuckDuckGo.

    There is a one-to-many relationship between destination and alias, and a one-to-one relationship
    between alias and destination. In other words, a destination can have many aliases, but an alias
    must uniquely point to a single destination (enforced by a unique constraint on dest_id and name).
    """

    destination = models.ForeignKey(
        "Destination",
        on_delete=models.CASCADE,
        related_name="aliases",
        related_query_name="alias",
    )
    name = models.CharField(max_length=100, unique=True)

    class Meta:
        db_table = "alias"
