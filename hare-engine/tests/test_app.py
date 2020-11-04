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
from tempfile import gettempdir
import unittest

import flask

from hare_engine.app import _gen_and_validate_config
from hare_engine import default_config


__author__ = "conveen"


class TestConfiguration(unittest.TestCase):
    """Tests for _gen_and_validate_config."""

    @staticmethod
    def _gen_flask_app():
        return flask.Flask(__name__)

    def setUp(self):
        self.have_secret_key_file = True
        if not Path(default_config.SECRET_KEY_PATH).is_file():
            self.have_secret_key_file = False
            with open(default_config.SECRET_KEY_PATH, "w") as scf:
                scf.write("THIS IS A FAKE SECRET KEY")

    def test_load_default_config(self):
        """Test that default config file is loaded when no custom config file
        or env vars are provided.
        """
        app = self._gen_flask_app()
        app.config.from_mapping(_gen_and_validate_config())
        for option in ("HARE_DATABASE_URL", "HARE_DATABASE_BOOTSTRAP_ON_STARTUP"):
            self.assertEqual(getattr(default_config, option), app.config.get(option))
        with open(default_config.SECRET_KEY_PATH) as scf:
            expected = scf.read()
        self.assertEqual(expected, app.config.get("SECRET_KEY"))

    def test_load_custom_config(self):
        """Test that custom config file is loaded from HARE_CONFIG_PATH."""
        # Create temporary copy of default config
        config_file_path = Path(gettempdir()).joinpath("config.py")
        # Change HARE_DATABASE_URL to new value
        with open(config_file_path, "w") as config_file:
            config_file.write("HARE_DATABASE_URL = \"sqlite:///hare.db\"\n")
            config_file.write("SECRET_KEY = \"THIS IS A SECRET\"\n")
        os.environ["HARE_CONFIG_PATH"] = str(config_file_path)

        try:
            app = self._gen_flask_app()
            app.config.from_mapping(_gen_and_validate_config())
            self.assertEqual("sqlite:///hare.db", app.config.get("HARE_DATABASE_URL"))
            self.assertEqual("THIS IS A SECRET", app.config.get("SECRET_KEY"))
        finally:
            Path(config_file_path).unlink()
            del os.environ["HARE_CONFIG_PATH"]

    def test_load_config_envvar_override(self):
        """Test that environment variables override default config options."""
        os.environ["HARE_DATABASE_URL"] = "sqlite:///hare.db"
        os.environ["SECRET_KEY"] = "THIS IS A FAKE SECRET"

        try:
            app = self._gen_flask_app()
            app.config.from_mapping(_gen_and_validate_config())
            self.assertEqual("sqlite:///hare.db", app.config.get("HARE_DATABASE_URL"))
            self.assertEqual("THIS IS A FAKE SECRET", app.config.get("SECRET_KEY"))
        finally:
            del os.environ["HARE_DATABASE_URL"]
            del os.environ["SECRET_KEY"]

    def tearDown(self):
        if not self.have_secret_key_file:
            Path(default_config.SECRET_KEY_PATH).unlink()
