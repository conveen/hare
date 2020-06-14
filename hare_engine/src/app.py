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

from os import getcwd, path, environ
from flask import Flask

from apps.engine.src.routes import RouteRegistry
from apps.engine.src.database import BaseTable
from apps.engine.src.cli import initialize_parser
from apps.common.manager import DBManager

def get_engine():
    app = Flask('__main__')
    config_object = environ.get('HARE_CONFIG_OBJECT')
    if config_object is not None:
        app.config.from_object(config_object)
    else:
        app.config.from_object('apps.engine.config.DefaultConfig')
    with open(app.config.get('HARE_SECRET_KEY'), 'r', encoding='UTF8') as scf:
        app.secret_key = scf.read()
    manager = DBManager(metadata=BaseTable.metadata)
    manager.initialize(
        app, 
        bootstrap=app.config.get('HARE_DATABASE_BOOTSTRAP_ON_STARTUP')
    )
    RouteRegistry.initialize(app)
    return app

def engine_main():
    app = get_engine()
    parser = initialize_parser()
    args = parser.parse_args()
    if app.config.get('HARE_SSL_ENABLE') or args.ssl:
        app.run(host=args.host, port=args.port, debug=args.debug, ssl_context='adhoc')
    else:
        app.run(host=args.host, port=args.port, debug=args.debug)
