## -*- coding: UTF8 -*-
## routes.py
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
# pylint: disable=W0702

from collections import defaultdict
import os
from typing import Any, Callable, ClassVar, DefaultDict, Dict, List, Optional, Tuple, Type
from urllib.parse import quote_plus

import flask
from lc_flask_reqparser import RequestParser
from lc_flask_routes import (
    BaseRouteWithParserMixin,
    RouteRegistryMixin,
    RouteResponse,
    WerkzeugLocalProxy,
)
from lc_sqlalchemy_dbutils.manager import DBManager

import hare_engine.src.database as db


__author__ = "conveen"


class RouteRegistry(RouteRegistryMixin, type):
    """Route registry metaclass for hare_engine."""
    __slots__ = ()
    _REGISTRY: ClassVar[Dict[str, Type["BaseRoute"]]] = dict()


class BaseRoute(BaseRouteWithParserMixin, metaclass=RouteRegistry):
    """Base route class for hare_engine."""
    __slots__ = ()


def QueryURLParameter(arg: str) -> Tuple[str, Optional[List[str]]]:     # pylint: disable=C0103
    """
    Args:
        arg     => query with alias and zero or more arguments
    Description:
        Parse alias and arguments (if any) from query URL parameter.
    Preconditions:
        The alias and arguments are separated by one or more spaces.
    Raises:
        ValueError: if arg does not contain any non-whitespace characters
    """
    # Strip whitespace
    arg = arg.strip()
    # Ensure arg contains at least one non-whitespace character
    if not arg:
        raise ValueError("query must contain at least an alias")

    # Split alias from arguments
    split_arg = arg.split(" ")
    alias = split_arg.pop(0)
    arguments = [quote_plus(arg) for arg in split_arg if arg != ""]
    return (alias, arguments)


class IndexRoute(BaseRoute):
    """Route: /
    Endpoint: "index"
    Description: Resolve alias and apply arguments from query (if any) to destination URL.

    If destination does not accept arguments, they will be ignored. If
    the destination accepts N arguments, and N+k arguments are supplied, then
    will combine the Nth and k arguments into a single, Nth argument.

    If no alias provided, or alias provided does not exist, then will use fallback destination.

    GET request URL should follow the format:
        "http(s)?://hare_domain.tld?(fallback=<fallback_alias>&)query=<alias(+arg_1+...+arg_N)?>"
    POST requests should follow the same structure with a JSON body.
    """
    __slots__ = ()

    ROUTE_MAP = {"/": {"endpoint": "index", "methods": ["GET", "POST"]}}

    @classmethod
    def gen_request_parser(cls) -> RequestParser:
        return (RequestParser()
                .add_argument("query", type=QueryURLParameter, required=True)
                .add_argument("fallback"))

    @classmethod
    def _resolve_destination_for_query(cls,
                                       dbm: DBManager,
                                       alias: Optional[str],
                                       fallback_alias: Optional[str]) -> db.Destination:
        """
        Args:
            alias           => destination alias, or first destination parameter if no alias provided
            fallback_alias  => optional fallback destination alias
        Description:
            Resolves destination for provided alias and fallback alias using the following algorithm:
                If neither alias nor fallback_alias provided, return default fallback
                If alias provided, and does resolve to destination, return it
                If alias provided but does not resolve
                    If fallback alias provided and does resolve, return it
                    If fallback alias provided and does not resolve, return default fallback
                If no alias provided, but fallback alias provided, return it
        Preconditions:
            dbm has existing session with database.
        Raises:
            HTTPException: if db.gen_default_fallback throws ValueError, as default fallback must exist in DB
        """
        destination = None
        if alias:
            destination = db.gen_destination_for_alias(dbm, alias)
        if not destination and fallback_alias:
            destination = db.gen_destination_for_alias(dbm, fallback_alias)
        if not destination:
            try:
                return db.gen_default_fallback(dbm)
            except ValueError:
                # See: db.gen_default_fallback
                flask.abort(500)
        return destination

    @classmethod
    def get(cls,
            app: WerkzeugLocalProxy,
            _: WerkzeugLocalProxy,
            __: WerkzeugLocalProxy,
            route_kwargs: Dict[str, Any]) -> RouteResponse:
        # Get request parameters
        # See: cls.gen_request_parser
        fallback_alias = route_kwargs.get("fallback")
        query = route_kwargs.get("query")
        # If query provided, separate into alias and arguments
        alias, arguments = (None, list()) if not query else query

        # Resolve destination for provided alias and fallback
        if alias == "list":
            return flask.redirect(flask.url_for("list_destinations"))
        destination = cls._resolve_destination_for_query(app.dbm, alias, fallback_alias)

        # If destination doesn't take arguments, redirect immediately
        if destination.num_args == 0:
            return flask.redirect(destination.url)
        # If destination is fallback, combine all arguments (including parsed alias) into single
        if destination.is_fallback:
            arguments.insert(0, alias)
            arguments = [" ".join(arguments)]
        # Otherwise, validate number of arguments against destination
        else:
            # If number of arguments is less than expected, return 400 Bad Request
            # TODO: Redirect to /list with message that number of args didn't match expectation
            if len(arguments) < destination.num_args:
                flask.abort(400)
            # If number of arguments is more than expected, merge remainder into last argument
            # and truncate arguments list
            if len(arguments) > destination.num_args:
                arguments[destination.num_args-1] = " ".join(arguments[destination.num_args-1:])
                arguments = arguments[:destination.num_args]

        try:
            return flask.redirect(destination.url.format(*arguments))
        except:
            # If formatting URL fails, attempt to redirect to default fallback
            # with empty query ("")
            try:
                return flask.redirect(db.gen_default_fallback(app.dbm).url.format(""))
            except:
                flask.abort(500)

    @classmethod
    def post(cls,
             app: WerkzeugLocalProxy,
             request: WerkzeugLocalProxy,
             session: WerkzeugLocalProxy,
             route_kwargs: Dict[str, Any]) -> RouteResponse:
        return cls.get(app, request, session, route_kwargs)


def ListURLParameter(delim: str) -> Callable[[str], List[str]]:     # pylint: disable=C0103
    """
    Arg:
        delim   => delimiter
    Description:
        Create anonymous function to parse `delim`-separated
        list of items.
    Preconditions:
        N/A
    Raises:
        N/A
    """
    def f(arg: str) -> List[str]:   # pylint: disable=C0103
        return arg.strip().split(delim)
    return f


class NewDestinationRoute(BaseRoute):
    """Route: /new
    Endpoint: "new_destination"
    Description: Add new destination with alias(es) to database.

    Destination and alias parameter validation is done by either database
    or db.add_destination_with_aliases. See that function's docstring for
    more information. Note that setting the default fallback destination
    is not currently supported via this endpoint and must be done by the DBA.
    """
    __slots__ = ()

    ROUTE_MAP = {"/new": {"endpoint": "new_destination", "methods": ["POST"]}}

    @classmethod
    def gen_request_parser(cls) -> RequestParser:
        return (RequestParser()
                .add_argument("url", required=True)
                .add_argument("description", required=True)
                .add_argument("aliases", type=ListURLParameter(","), required=True)
                .add_argument("is_fallback", type=bool, default=False)
                .add_argument("no_redirect", type=bool, default=False))

    @classmethod
    def post(cls,
             app: WerkzeugLocalProxy,
             _: WerkzeugLocalProxy,
             __: WerkzeugLocalProxy,
             route_kwargs: Dict[str, Any]) -> RouteResponse:
        try:
            db.add_destination_with_aliases(app.dbm,
                                            route_kwargs.get("url"),            # type: ignore
                                            route_kwargs.get("description"),    # type: ignore
                                            route_kwargs.get("aliases"),        # type: ignore
                                            route_kwargs.get("is_fallback"))    # type: ignore
        except:
            flask.abort(400)
        if not route_kwargs.get("no_redirect"):
            return flask.redirect(flask.url_for("list_destinations"))
        return "", 200


class ListDestinationsRoute(BaseRoute):
    """Route: /list
    Endpoint: "list_destinations"
    Description: List all available destinations and descriptions with aliases.

    The `output` URL parameter determines the return type:
        1) html => HTML page with table of destinations (default)
        2) json => Array of JSON objects containing destination information

    The same information is returned for each destination by both methods, and
    if no `output` URL parameter is specified, it defaults to HTML.
    """
    __slots__ = ()

    ROUTE_MAP = {"/list": {"endpoint": "list_destinations", "methods": ["GET"]}}

    @classmethod
    def gen_request_parser(cls) -> RequestParser:
        return (RequestParser()
                .add_argument("output", choices=["html", "json"], default="html"))

    @classmethod
    def get(cls,
            app: WerkzeugLocalProxy,
            _: WerkzeugLocalProxy,
            __: WerkzeugLocalProxy,
            route_kwargs: Dict[str, Any]) -> RouteResponse:
        # Get all destinations `url` and `description` fields, with list of aliases for each
        # NOTE: This is currently done in two steps with some server-side (not DB-side)
        #       matching logic - get all destinations, get all aliases, then match.
        #       This is because there's not easy cross-RDBMS way to aggregate values,
        #       in this case aliases, in SQLAlchemy. This will need to be fixed later.
        try:
            aliases_query = app.dbm.query(db.Alias).with_entities(db.Alias.dest_id, db.Alias.name)
            aliases_map: DefaultDict[str, List[str]] = defaultdict(list)
            for alias in aliases_query:
                aliases_map[alias.dest_id].append(alias.name)
        except:
            flask.abort(500)
        try:
            destinations_query = (app.dbm
                                  .query(db.Destination)
                                  .with_entities(db.Destination.id,     # pylint: disable=E1101
                                                 db.Destination.url,
                                                 db.Destination.description)
                                  .order_by(db.Destination.url))
            destinations = [{
                                "aliases":", ".join(sorted(aliases_map.get(destination.id), key=len)),  # type: ignore
                                "description":destination.description,
                                "url":destination.url,
                            } for destination in destinations_query]
        except:
            flask.abort(500)

        # If output type is JSON, return JSON array
        if route_kwargs.get("output") == "json":
            return flask.jsonify(destinations)
        # Otherwise return HTML page
        return flask.render_template("list.html", destinations=destinations)


if os.environ.get("ENVIRONMENT") == "TEST":
    import unittest


    class TestQueryURLParameter(unittest.TestCase):
        """Tests for QueryURLParameter."""

        def test_query_url_parameter(self):
            """Test that QueryURLParameter correctly splits a space-separated
            string of components into a tuple of (first_component, List<remainder>).
            """
            tests = [
                ("Empty string", "", ValueError),
                ("String with only spaces", "            ", ValueError),
                ("String with single letter component with spaces", "    c    ", ("c", list())),
                ("String with single component with spaces", "    component    ", ("component", list())),
                ("String with single component no spaces", "component", ("component", list())),
                ("String with single component no spaces (symbols and UTF8 characters)",
                 "arn:aws:iam:::123456789:user/😀",
                 ("arn:aws:iam:::123456789:user/😀", list())),
                ("String with three components with spaces",
                 "   this is a test query    ",
                 ("this", ["is", "a", "test", "query"])),
                ("String with nine components no spaces",
                 "First second third fourth fifth sixth seventh eighth ninth",
                 ("First", ["second", "third", "fourth", "fifth", "sixth", "seventh", "eighth", "ninth"])),
            ]

            for test_name, arg, expected in tests:
                with self.subTest(test_name=test_name):
                    if expected is ValueError:
                        self.assertRaises(ValueError, QueryURLParameter, arg)
                    else:
                        self.assertEqual(expected, QueryURLParameter(arg))


    class TestRouteBase(unittest.TestCase):
        """Base test class for routes."""

        def setUp(self):
            self.app = flask.Flask(type(self).__name__)
            self.app.testing = True
            self.app.dbm = DBManager("sqlite://", db.BaseTable.metadata)
            self.app.dbm.connect().bootstrap_db()

        def _gen_test_client(self):
            """Generate test client for self.app."""
            return self.app.test_client()


    class TestIndexRoute(TestRouteBase):
        """Tests for IndexRoute."""

        def setUp(self):
            super().setUp()
            IndexRoute.register_routes(self.app)
            self.session = self.app.dbm.gen_session()
            # Add some destinations to the database
            db.add_destination_with_aliases(self.app.dbm,
                                            "https://www.reddit.com/r/{}",
                                            "Reddit sub",
                                            ["r", "reddit"],
                                            False,
                                            False)
        def _insert_default_fallback(self):
            """Helper function to insert DuckDuckGo entry into database."""
            db.add_destination_with_aliases(self.app.dbm,
                                            "https://duckduckgo.com?q={}",
                                            "DuckDuckGo",
                                            ["ddg"],
                                            True,
                                            True)

        def test_query_alias_not_exist_with_fallback(self):
            """Test that query with alias and fallback, where alias doesn't exist,
            gets redirected to the fallback destination with alias + rest of query
            passed as single parameter.
            """
            self.session.begin_nested()
            self._insert_default_fallback()

            with self._gen_test_client() as client:
                response = client.get("/", query_string={
                    "fallback": "ddg",
                    "query": "cats are animals right?"
                })
                self.assertEqual(302, response.status_code)
                self.assertEqual("https://duckduckgo.com?q=cats%20are%20animals%20right%3F", response.location)

            self.session.rollback()

        def tearDown(self):
            self.session.close()
