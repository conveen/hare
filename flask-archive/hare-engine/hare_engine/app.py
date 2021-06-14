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

from collections import ChainMap
from importlib import import_module
from os import environ as ENV

import flask
from lc_sqlalchemy_dbutils.manager import DBManager

import hare_engine.database as db
from hare_engine.default_config import CONFIG as DEFAULT_FILE_CONFIG
from hare_engine.routes import RouteRegistry


def gen_app() -> flask.Flask:
    """
    Args:
        N/A
    Description:
        Generate Flask app and conduct the following initialization tasks:
            1) Resolve and validate configuration options
            2) Set the app secret key
            3) Register routes on the app using the RouteRegistry
            4) Initialize the database manager and attach it to the app
    Preconditions:
        N/A
    Raises:
        ValueError: if either HARE_DATABASE_URL or HARE_SECRET_KEY are not provided
    """
    app = flask.Flask(__name__)

    # Try to resolve configuration file path, or use default.
    file_config_module_path = ENV.get("HARE_CONFIG_MODULE")
    if file_config_module_path:
        file_config_module = import_module(file_config_module_path)
        file_config = file_config_module.CONFIG     # type: ignore
    else:
        file_config = DEFAULT_FILE_CONFIG

    # Create config hierarchy from env variables and config file and validate
    config = ChainMap(ENV, file_config)
    for option in ("HARE_DATABASE_URL", "HARE_SECRET_KEY"):
        if not option in config:
            raise ValueError("must provide {} in config or env variable".format(option))
    app.config["HARE_DATABASE_URL"] = config["HARE_DATABASE_URL"]
    app.config["SECRET_KEY"] = config["HARE_SECRET_KEY"]
    app.config["HARE_DATABASE_BOOTSTRAP_ON_STARTUP"] = "HARE_DATABASE_BOOTSTRAP_ON_STARTUP" in config
    app.config["JSON_AS_ASCII"] = False
    app.config["JSON_SORT_KEYS"] = False

    # Register routes
    RouteRegistry.register_routes(app)

    # Initialize database manager and attach to app
    # NOTE: "HARE_DATABASE_URL" is guaranteed to have a value due to check above
    app.dbm = DBManager(app.config["HARE_DATABASE_URL"], db.BaseTable.metadata, True)  # type: ignore
    app.dbm.connect()
    if app.config["HARE_DATABASE_BOOTSTRAP_ON_STARTUP"]:
        app.dbm.bootstrap_db()
    return app
