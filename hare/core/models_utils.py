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
from urllib.parse import urlsplit, urlunsplit


logger = logging.getLogger(__name__)


def validate_netloc_url(url: str) -> str:
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


def gen_num_args_from_url(url: str) -> int:
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
                raise ValueError(f'must not have keyword arguments ("{field_name}")')
            if field_name == "":
                num_args += 1
    return num_args
