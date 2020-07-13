## -*- coding: UTF8 -*-
## database.py
## Copyright (c) 2020 conveen
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

import os
from string import Formatter
from typing import List
from urllib.parse import quote_plus, urlsplit, urlunsplit

from sqlalchemy.exc import InvalidRequestError
from sqlalchemy.ext.declarative import declarative_base, declared_attr
from sqlalchemy.orm import relationship
from sqlalchemy.schema import Column, ForeignKey
from sqlalchemy.sql.expression import text
from sqlalchemy.types import Boolean, Integer, UnicodeText, TIMESTAMP
from lc_sqlalchemy_dbutils.manager import DBManager
from lc_sqlalchemy_dbutils.schema import TimestampDefaultExpression


__author__ = "conveen"


##### Database Schema Definition #####


class BaseTableMixin:
    """Mixin class containing base columns that should be present
    on every table:
        1) id: primary key
        2) created_at: timestamp of record creation in database
    Also sets table name as lowercase of each class name.
    """
    __slots__ = ()

    @declared_attr
    def __tablename__(cls):     # pylint: disable=E0213
        return cls.__name__.lower()

    id = Column(Integer, primary_key=True, nullable=False)
    created_at = Column(TIMESTAMP(timezone=True), server_default=TimestampDefaultExpression())

    def __repr__(self):
        # Get class name (not table name)
        cls_name = type(self).__name__
        # Get columns for table
        columns = [(column.name, getattr(self, column.name)) for column in self.__table__.columns]
        # "<class_name>(<column_1>=<value1>, <column_2>=<value2>, ..., <column_N>=<valueN>)"
        return "{}({})".format(cls_name,
                               ", ".join("{}={}".format(column, value) for column, value in columns))


BaseTable = declarative_base(cls=BaseTableMixin)


class Destination(BaseTable):   # type: ignore
    """destination table

    A destination represents a unique _URL_ and set of zero or more parameters.
    For example, a destination could be https://www.youtube.com/results?search_query={},
    where '{}' is a single parameter. Destinations could also have no
    parameters, like https://time.is/, or have parameters as part of the URL (not a URL parameter).

    Fallback destinations are URLs with a single parameter, and are used if the specified
    destination does not exist, or there is otherwise an error in redirecting an a destination.
    Common choices could be DuckDuckGo, Wikipedia, or Google.
    """
    url = Column(UnicodeText, nullable=False, index=True, unique=True)
    num_args = Column(Integer, nullable=False)
    is_fallback = Column(Boolean, nullable=False, index=True)
    is_default_fallback = Column(Boolean, nullable=False, server_default=text("false"))
    description = Column(UnicodeText, nullable=False)
    aliases = relationship("Alias", backref="destination")  # type: ignore


class DestinationForeignKeyMixin:
    """Mixin class for tables that have a Foreign Key relationship
    with the destination table. The ondelete and onupdate behavior
    is set to CASCADE so that any changes made through the SQLAlchemy ORM
    to the destination table will be propogated to any table with a relation to it.
    Note that ondelete and onupdate are SQLAlchemy constructs, and do not change
    database (non-ORM) behavior.
    """
    __slots__ = ()

    @declared_attr
    def dest_id(cls):   # pylint: disable=E0213
        return Column(Integer,
                      ForeignKey("destination.id", onupdate="CASCADE", ondelete="CASCADE"),
                      nullable=False,
                      index=True)


class Alias(DestinationForeignKeyMixin, BaseTable):     # type: ignore
    """alias table

    Aliases act as references, or pointers, to destinations, and are the shortcuts users type to
    get redirected to a destination. For example, the shortcuts "ddg", "duckduckgo", and "go" could
    all reference a search on DuckDuckGo.

    There is a one-to-many relationship between destination and alias, and a one-to-one relationship
    between alias and destination. In other words, a destination can have many aliases, but an alias
    must uniquely point to a single destination (enforced by a unique constraint on dest_id and name).
    """
    name = Column(UnicodeText, nullable=False, unique=True, index=True)


##### Database Routines (and helpers) #####


def _validate_netloc_url(url: str) -> str:
    """
    Args:
        url     => URL to validate
    Description:
        Helper for add_destination_with_aliases.
        Validate that URL is valid netloc URL according to RFC 1808.
        If no scheme is provided, defaults to http (like most browsers).
    Preconditions:
        N/A
    Raises:
        ValueError: if URL is invalid, or
                    if URL doesn't point to network location
    """
    # Ensure URL is valid netloc url according to RFC 1808
    # See: https://tools.ietf.org/html/rfc1808.html
    try:
        scheme, netloc, path, query, fragment = urlsplit(url)
    except ValueError as exc:
        raise ValueError("invalid URL ({})".format(exc))
    # URL must point to network location (website)
    if not netloc:
        raise ValueError("URL must point to website")
    # If no scheme is provided, will default to http (as most browsers do)
    if not scheme:
        scheme = "http"
        url = urlunsplit((scheme, netloc, path, query, fragment,))     # pylint: disable=E1121
    return url


def _gen_num_args_from_url(url: str) -> int:
    """
    Args:
        url     => URL to parse
    Description:
        Helper for add_destination_with_aliases.
        Parses number of positional arguments in `url`.
    Preconditions:
        N/A
    Raises:
        N/A
    """
    format_args = list(Formatter().parse(url))
    # If only one entry and field_name is None, then doesn't have any format args
    # Each entry in format_args is 4-tuple with (iteral_text, field_name, format_spec, conversion)
    # See: https://docs.python.org/3.8/library/string.html#string.Formatter.parse
    if len(format_args) == 1 and format_args[0][1] is None:
        num_args = 0
    else:
        num_args = len(format_args)
        for _, field_name, __, ___ in format_args:
            if field_name:
                raise ValueError("must not have keyword arguments (\"{}\")".format(field_name))
    return num_args


def add_destination_with_aliases(dbm: DBManager,
                                 url: str,
                                 description: str,
                                 aliases: List[str],
                                 is_fallback: bool = False,
                                 is_default_fallback: bool = False):
    """
    Args:
        See destination and alias table schemata.
    Description:
        Validate arguments and add destination to database with provided alias(es).
        Some validation is also done on the database. Any exceptions raised by the underlying
        database driver, or any validation errors (ValueError), must be handled by the caller.
    Preconditions:
        Database manager has existing session with database
    Raises:
        ValueError: if arguments don't pass validation
        Consult database driver and DBManager class for other possible errors
    """
    # Ensure number of aliases >= 1
    # NOTE: Checked here due to lack of runtime type-checking
    if not aliases:
        raise ValueError("must provide at least one alias")

    # Validate URL
    url = _validate_netloc_url(url)

    # Parse num_args from url
    num_args = _gen_num_args_from_url(url)

    # Ensure num_args is 1 if is fallback destination
    if is_fallback:
        if num_args != 1:
            raise ValueError("fallback destinations must have one argument")
    # Ensure if destination is default fallback, is also fallback
    elif is_default_fallback and not is_fallback:
        is_fallback = True

    # Flush pending transaction if within one to ensure
    # can rollback destination addition transaction if fails
    try:
        dbm.commit()
    except InvalidRequestError:
        pass

    # If is_default_fallback, determine if existing default fallback(s)
    # Cannot have two default fallbacks, so must atomically add new
    # destination as default and remove flag from existing.
    if is_default_fallback:
        existing_defaults = dbm.query(Destination, is_default_fallback=True).all()
        if existing_defaults:
            for destination in existing_defaults:
                destination.is_default_fallback = False

    # Add destination and alias records to session
    destination = Destination(url=url,
                              num_args=num_args,
                              is_fallback=is_fallback,
                              is_default_fallback=is_default_fallback,
                              description=description)
    dbm.add(destination)
    for alias in aliases:
        dbm.add(Alias(name=quote_plus(alias), destination=destination))

    # Try to commit transaction. If fails, rollback transaction and pass through exception
    try:
        dbm.commit()
    except:
        dbm.rollback()
        raise


if os.environ.get("ENVIRONMENT") == "TEST":
    import unittest


    class TestAddDestination(unittest.TestCase):
        """Tests for add_destination_with_aliases and helpers."""

        def setUp(self):
            self.manager = DBManager("sqlite://")

        def test_validate_netloc_url(self):
            """Test that _validate_netloc_url raises exceptions
            on invalid input, and replaces scheme when necessary.
            """
            tests = [
                ("Not a URL", "This is not a URL", ValueError),
                ("UNC path to network share", "\\\\fileshare02\\share_name", ValueError),
                ("Domain without preceding slashes (//)", "www.python.org", ValueError),
                ("URL without preceding slashes (//)", "www.python.org/downloads/", ValueError),
                ("URL path without domain", "/downloads/python3.6.9", ValueError),
                ("IP address and port without preceding slashes (//)", "172.84.99.127:8080/g", ValueError),
                ("Amazon ARN", "arn:aws:iam::123456789012:user/username@domain.com", ValueError),
                ("Valid network location without scheme",
                 "//www.python.org/downloads",
                 "http://www.python.org/downloads",),
                ("Valid network location with scheme",
                 "https://www.python.org/downloads",
                 None,),
            ]

            for test_name, url, expected in tests:
                with self.subTest(test_name=test_name):
                    if expected is ValueError:
                        self.assertRaises(expected, _validate_netloc_url, url)
                    else:
                        self.assertEqual(expected or url, _validate_netloc_url(url))

        def test_gen_num_args_from_url(self):
            """Test that _gen_num_args_from_url parses the correct number of positional
            arguments from a format string, and raises a ValueError if format string
            contains keyword arguments.
            """
            tests = [
                ("One keyword argument (search)", "https://en.wikipedia.org/w/index.php?search={search}", ValueError),
                ("Two keyword arguments (search, title)",
                 "https://en.wikipedia.org/w/index.php?search={search}",
                 ValueError),
                ("Single keyword, single positional", "https://en.wikipedia.org/wiki/{}?title={title}", ValueError),
                ("No arguments", "https://en.wikipedia.org/wiki/Joan_of_Arc", 0),
                ("Single argument", "https://en.wikipedia.org/wiki/{}", 1),
                ("Two arguments", "https://en.wikipedia.org/wiki/{}?title={}", 2),
                ("Ten arguments", "https://en.wikipedia.org/wiki/{}?title={}{}{}{}{}{}{}{}{}", 10),
            ]

            for test_name, url, expected in tests:
                with self.subTest(test_name=test_name):
                    if expected is ValueError:
                        self.assertRaises(expected, _gen_num_args_from_url, url)
                    else:
                        self.assertEqual(expected, _gen_num_args_from_url(url))

        def test_add_destination_no_aliases(self):
            """Test that add_destination_with_aliases raises ValueError
            is not aliases provided.
            """
            self.assertRaises(ValueError,
                              add_destination_with_aliases,
                              self.manager,
                              "https://wikipedia.org/wiki/{}",
                              "Wikipedia",
                              list())

        def test_add_destination_fallback(self):
            """Test that add_destination_with_aliases raises ValueError
            if destination is fallback and number of arguments isn't 1.
            """
            tests = [
                ("Fallback no arguments", "https://duckduckgo.com"),
                ("Fallback more than one argument", "https://duckduckgo.com?q={}&ia={}"),
            ]

            for test_name, url in tests:
                with self.subTest(test_name=test_name):
                    self.assertRaises(ValueError,
                                      add_destination_with_aliases,
                                      self.manager,
                                      url,
                                      "This test should raise ValueError",
                                      ["ddgs"],
                                      True)
