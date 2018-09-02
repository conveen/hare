# -*- coding: UTF8 -*-
# app.py
#Copyright (c) 2018 conveen
#
#Permission is hereby granted, free of charge, to any person obtaining a copy
#of this software and associated documentation files (the "Software"), to deal
#in the Software without restriction, including without limitation the rights
#to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
#copies of the Software, and to permit persons to whom the Software is
#furnished to do so, subject to the following conditions:
#
#The above copyright notice and this permission notice shall be included in all
#copies or substantial portions of the Software.
#
#THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
#IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
#FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
#AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
#LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
#OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
#SOFTWARE.

from os import getcwd, path
from flask import Flask

from apps.engine.src.routes import RouteRegistry
from apps.engine.src.database import BaseTable
from apps.engine.src.cli import initialize_parser
from apps.common.manager import DBManager

def get_engine():
    app = Flask('__main__')
    with open(path.join(path.abspath(path.dirname(__file__)), '.app_secret.key'), 'r', encoding='utf8') as scf:
        app.secret_key = scf.read()
    app.config['SQLALCHEMY_DATABASE_URI'] = 'sqlite:///' + path.join(path.abspath(path.dirname(__file__)), 'hare.db')
    app.config['SQLALCHEMY_COMMIT_ON_TEARDOWN'] = True
    manager = DBManager(metadata=BaseTable.metadata, scoped=True)
    manager.initialize(app, bootstrap=True, create_session=True)
    RouteRegistry.initialize(app)
    return app

def engine_main():
    app = get_engine()
    parser = initialize_parser()
    args = parser.parse_args()
    if args.ssl:
        app.run(host=args.host, port=args.port, debug=args.debug, ssl_context='adhoc')
    else:
        app.run(host=args.host, port=args.port, debug=args.debug)
