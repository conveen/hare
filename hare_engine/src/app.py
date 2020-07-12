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

from os import environ

from flask import Flask
from lc_sqlalchemy_dbutils.manager import DBManager

import hare_engine.src.database as db
from hare_engine.src.routes import RouteRegistry

def gen_app() -> Flask:
    """
    Args:
        N/A
    Description:
        Generate Flask app and conduct the following initialization tasks:
            1) Resolve and validate the config object by using the HARE_CONFIG_OBJECT environment variable
               See: https://flask.palletsprojects.com/en/1.1.x/api/#flask.Config.from_object
            2) Set the app secret key
            3) Register routes on the app using the RouteRegistry
            4) Initialize the database manager and attach it to the app
    Preconditions:
        N/A
    Raises:
        ValueError: if either HARE_DATABASE_URI or HARE_SECRET_KEY are not specified in the config
    """
    app = Flask(__name__)

    # Resolve configuration object and validate options. If none provided, use default.
    config_object = environ.get("HARE_CONFIG_OBJECT")
    if config_object is not None:
        app.config.from_object(config_object)
    else:
        app.config.from_object("hare_engine.default_config")
    for option in ("HARE_DATABASE_URI", "HARE_SECRET_KEY"):
        if not app.config.get(option):
            raise ValueError("must provide {} in config".format(option))

    # Read secret key from config, is required for Sessions to work properly
    # See: https://flask.palletsprojects.com/en/1.1.x/api/#sessions
    # NOTE: "HARE_SECRET_KEY" is guaranteed to have a value due to check above
    with open(app.config.get("HARE_SECRET_KEY")) as scf:    # type: ignore
        app.secret_key = scf.read()

    # Attach routes to app
    RouteRegistry.register_routes(app)

    # Initialize database manager and attach to app
    # NOTE: "HARE_DATABASE_URI" is guaranteed to have a value due to check above
    app.dbm = DBManager(app.config.get("HARE_DATABASE_URI"), db.BaseTable.metadata, True)  # type: ignore
    app.dbm.connect()
    if app.config.get("HARE_DATABASE_BOOTSTRAP_ON_STARTUP"):
        app.dbm.bootstrap_db()
    return app
