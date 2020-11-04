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

from lc_sqlalchemy_dbutils.manager import DBManager

from hare_engine import database as db


__author__ = "conveen"


class TestDBBase(unittest.TestCase):
    """Base class for DB tests."""

    def setUp(self):
        self.dbm = DBManager("sqlite://", db.BaseTable.metadata)
        self.dbm.connect().bootstrap_db()
        self.session = self.dbm.gen_session()


class TestAddDestination(TestDBBase):
    """Tests for add_destination_with_aliases and helpers."""

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
                    self.assertRaises(expected, db._validate_netloc_url, url)
                else:
                    self.assertEqual(expected or url, db._validate_netloc_url(url))

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
                    self.assertRaises(expected, db._gen_num_args_from_url, url)
                else:
                    self.assertEqual(expected, db._gen_num_args_from_url(url))

    def test_add_destination_no_aliases(self):
        """Test that add_destination_with_aliases raises ValueError
        is not aliases provided.
        """
        self.assertRaises(ValueError,
                          db.add_destination_with_aliases,
                          self.dbm,
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
                                  db.add_destination_with_aliases,
                                  self.dbm,
                                  url,
                                  "This test should raise ValueError",
                                  ["ddgs"],
                                  True)


class TestGenDestination(TestDBBase):
    """Tests for gen_default_fallback and gen_destination_for_alias."""

    def setUp(self):
        super().setUp()
        db.add_destination_with_aliases(self.dbm,
                                     "https://www.reddit.com/r/{}",
                                     "Reddit - subreddit",
                                     ["r", "reddit"],
                                     False,
                                     False)

    def _insert_default_fallback(self):
        """Helper function to insert DuckDuckGo entry into database."""
        db.add_destination_with_aliases(self.dbm,
                                     "https://duckduckgo.com?q={}",
                                     "DuckDuckGo",
                                     ["ddg"],
                                     True,
                                     True)

    def test_gen_default_fallback_is_destination(self):
        """Test that gen_default_fallback returns the
        Destination record for default fallback when one exists."""
        # Start transaction and add default fallback to database
        # NOTE: Must begin a nested transaction, as autocommit=False by default
        #       which automatically starts a transaction when Session is created.
        # See: https://docs.sqlalchemy.org/en/13/orm/session_api.html#sqlalchemy.orm.session.SessionTransaction
        self.session.begin_nested()
        self._insert_default_fallback()

        destination = db.gen_default_fallback(self.dbm)
        self.assertEqual(db.Destination, type(destination))
        self.assertEqual("https://duckduckgo.com?q={}", destination.url)

        self.session.rollback()

    def test_gen_default_fallback_without_default_fallback(self):
        """Test that gen_default_fallback raises a ValueError
        when no default fallback exists in database.
        """
        self.assertRaises(ValueError, db.gen_default_fallback, self.dbm)

    def test_gen_destination_for_alias_is_destination(self):
        """Test that gen_destination_for_alias returns
        (the correct) Destination record.
        """
        destination = db.gen_destination_for_alias(self.dbm, "reddit")
        self.assertIsInstance(destination, db.Destination)
        self.assertEqual("https://www.reddit.com/r/{}", destination.url)

    def test_gen_destination_for_alias_invalid_alias(self):
        """Test that gen_destination_for_alias returns None for
        non-existent alias.
        """
        self.assertIsNone(db.gen_destination_for_alias(self.dbm, "twitter"))
