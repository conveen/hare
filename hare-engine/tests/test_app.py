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
from pathlib import Path
import unittest
from unittest import mock

import flask

from hare_engine.app import gen_app
from hare_engine.default_config import CONFIG as DEFAULT_FILE_CONFIG
from tests.common import TempEnviron
from tests.custom_config_valid import CONFIG as CUSTOM_VALID_CONFIG


__author__ = "conveen"


class TestConfiguration(unittest.TestCase):
    """Tests for _gen_and_validate_config."""

    def test_default_config(self):
        app = gen_app()
        for option in ("HARE_DATABASE_URL", "HARE_DATABASE_BOOTSTRAP_ON_STARTUP"):
            self.assertEqual(DEFAULT_FILE_CONFIG[option], app.config[option])
        self.assertEqual(DEFAULT_FILE_CONFIG["HARE_SECRET_KEY"], app.config["SECRET_KEY"])

    def test_custom_valid_config(self):
        with TempEnviron(HARE_CONFIG_MODULE="tests.custom_config_valid"), \
            mock.patch("hare_engine.app.DBManager.connect"), \
            mock.patch("hare_engine.app.DBManager.bootstrap_db"):
            app = gen_app()
            self.assertEqual(CUSTOM_VALID_CONFIG["HARE_DATABASE_URL"], app.config["HARE_DATABASE_URL"])
            self.assertEqual(False, app.config["HARE_DATABASE_BOOTSTRAP_ON_STARTUP"])
            self.assertEqual(CUSTOM_VALID_CONFIG["HARE_SECRET_KEY"], app.config["SECRET_KEY"])

    def test_env_variables(self):
        database_url = "mysql://scott:tiger@localhost/foo"
        secret_key = "Env variable secret key"
        with TempEnviron(HARE_DATABASE_URL=database_url, HARE_DATABASE_BOOTSTRAP_ON_STARTUP="True", HARE_SECRET_KEY=secret_key), \
            mock.patch("hare_engine.app.DBManager.connect"), \
            mock.patch("hare_engine.app.DBManager.bootstrap_db"):
            app = gen_app()
            self.assertEqual(database_url, app.config["HARE_DATABASE_URL"])
            self.assertEqual(True, app.config["HARE_DATABASE_BOOTSTRAP_ON_STARTUP"])
            self.assertEqual(secret_key, app.config["SECRET_KEY"])

    def test_default_config_with_env_override(self):
        database_url = "postgres://scott:tiger@localhost/foo"
        with TempEnviron(HARE_DATABASE_URL=database_url), \
            mock.patch("hare_engine.app.DBManager.connect"), \
            mock.patch("hare_engine.app.DBManager.bootstrap_db"):
            app = gen_app()
            self.assertEqual(database_url, app.config["HARE_DATABASE_URL"])
            self.assertEqual(DEFAULT_FILE_CONFIG["HARE_DATABASE_BOOTSTRAP_ON_STARTUP"], app.config["HARE_DATABASE_BOOTSTRAP_ON_STARTUP"])
            self.assertEqual(DEFAULT_FILE_CONFIG["HARE_SECRET_KEY"], app.config["SECRET_KEY"])

    def test_invalid_config(self):
        with TempEnviron(HARE_CONFIG_MODULE="tests.custom_config_invalid"):
            self.assertRaises(ModuleNotFoundError, gen_app)
