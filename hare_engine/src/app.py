## -*- coding: UTF8 -*-
## app.py
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
from typing import Dict

import flask
from lc_sqlalchemy_dbutils.manager import DBManager

import hare_engine.src.database as db
from hare_engine.src.routes import RouteRegistry


def _gen_and_validate_config() -> flask.Config:
    """
    Args:
        N/A
    Description:
        Resolve and validate configuration options by using the HARE_CONFIG_PATH environment variable,
        and envrironment variables containing configuration options themselves (overwrite options from config).
        See: https://flask.palletsprojects.com/en/1.1.x/api/#flask.Config.from_pyfile
    Preconditions:
        Flask app is started from the project root directory.
    Raises:
        ValueError: if either HARE_DATABASE_URL or (SECRET_KEY || SECRET_KEY_PATH) are not resolved
                    in config object or env variables
    """
    # Initialize config object
    config = flask.Config(Path.cwd(), defaults={
        "HARE_DATABASE_URL": None,
        "SECRET_KEY": None,
        "SECRET_KEY_PATH": None,
        "HARE_DATABASE_BOOTSTRAP_ON_STARTUP": False,
    })

    # Read options from config file
    default_config_file = Path.cwd().joinpath("hare_engine").joinpath("default_config.py")
    config_file = os.environ.get("HARE_CONFIG_PATH", default_config_file)
    try:
        config.from_pyfile(config_file)
    except (FileNotFoundError, IsADirectoryError):
        pass

    # Read options from environment variables
    for option in config:
        env_value = os.environ.get(option)
        if env_value:
            config[option] = env_value

    # Validate configuration options
    if not config.get("HARE_DATABASE_URL"):
        raise ValueError("must provide HARE_DATABASE_URL configuration option")

    # Secret key is required for Flask sessions to work
    # See: https://flask.palletsprojects.com/en/1.1.x/api/#sessions
    if not config.get("SECRET_KEY"):
        secret_key_path = config.get("SECRET_KEY_PATH")
        if secret_key_path and Path(secret_key_path).is_file():
            with open(secret_key_path) as scf:
                config["SECRET_KEY"] = scf.read()
        else:
            raise ValueError("must provide either SECRET_KEY or SECRET_KEY_PATH configuration option")
    return config


def gen_app() -> flask.Flask:
    """
    Args:
        N/A
    Description:
        Generate Flask app and conduct the following initialization tasks:
            1) Resolve and validate configuration options (see: _gen_and_validate_config_options)
            2) Set the app secret key
            3) Register routes on the app using the RouteRegistry
            4) Initialize the database manager and attach it to the app
    Preconditions:
        N/A
    Raises:
        ValueError: if either HARE_DATABASE_URL or HARE_SECRET_KEY are not specified in the config
    """
    app = flask.Flask(__name__)

    # Resolve configuration object and validate options.
    app.config.from_mapping(_gen_and_validate_config())

    # Attach routes to app
    RouteRegistry.register_routes(app)

    # Initialize database manager and attach to app
    # NOTE: "HARE_DATABASE_URL" is guaranteed to have a value due to check above
    app.dbm = DBManager(app.config.get("HARE_DATABASE_URL"), db.BaseTable.metadata, True)  # type: ignore
    app.dbm.connect()
    if app.config.get("HARE_DATABASE_BOOTSTRAP_ON_STARTUP"):
        app.dbm.bootstrap_db()
    return app


if os.environ.get("ENVIRONMENT") == "TEST":
    from tempfile import gettempdir
    import unittest

    from hare_engine import default_config


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
                Path(default_config.HARE_SECRET_KEY_PATH).unlink()
