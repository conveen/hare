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

import unittest

import flask
from lc_sqlalchemy_dbutils.manager import DBManager

import hare_engine.database as db
from hare_engine import routes


__author__ = "conveen"


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
                    self.assertRaises(ValueError, routes.QueryURLParameter, arg)
                else:
                    self.assertEqual(expected, routes.QueryURLParameter(arg))


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
        routes.IndexRoute.register_routes(self.app)
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
