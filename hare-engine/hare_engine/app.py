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

import flask
from lc_sqlalchemy_dbutils.manager import DBManager

import hare_engine.database as db
from hare_engine.routes import RouteRegistry


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
