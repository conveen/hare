#!/usr/bin/env python3
## -*- coding: UTF8 -*-
## newdest.py
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

import sys
from argparse import ArgumentParser

from apps.engine.src.database import *
from apps.common.manager import DBManager

def newdest(args):
    dbm = DBManager(conn_string=args.conn_string, metadata=BaseTable.metadata, scoped=False)
    try:
        dbm.initialize(None, bootstrap=True)
        dbm.create_session()
    except Exception as e:
        print('::: Failed to create connection to database:', str(e))
    else:
        priority = None
        if args.is_fallback:
            last_fallback = dbm.query(Destination).order_by(Destination.priority.desc()).first()
            if last_fallback is None:
                priority = 1
            else:
                priority = last_fallback.priority + 1
        destination = Destination(\
            url         = args.url,
            is_fallback = args.is_fallback,
            priority    = priority\
        )
        destination.aliases = [ Alias(name=alias) for alias in args.aliases ]
        if args.argument is not None:
            destination.args = [ Argument(name=args.argument, type=args.argument_type, default=args.argument_default) ]
        print('::: Destination:', destination)
        for alias in destination.aliases:
            print('    ::: Alias:', alias)
        if destination.args is not None and len(destination.args) > 0:
            for arg in destination.args:
                print('    ::: Argument:', arg)
        try:
            dbm.add(destination)
            dbm.commit()
            print('::: Successfully added destination to database')
        except Exception as e:
            dbm.rollback()
            print('::: Failed to add destination to database:', str(e))
        finally:
            dbm.session.close()

def main():
    parser = ArgumentParser(prog='newdest.py', description='CL application to add new destinations to database')
    parser.add_argument('-c', '--connect', type=str, required=True, help='Connection string for database', dest='conn_string')
    parser.add_argument('-u', '--url', type=str, required=True, help='URL of new destination', dest='url')
    parser.add_argument('-f', '--fallback', action='store_true', help='Whether new destination is a fallback destination or not', dest='is_fallback')
    parser.add_argument('-a', '--alias', type=str, action='append', required=True, help='Alias(es) for new destination', dest='aliases')
    parser.add_argument('-A', '--arg-name', type=str, help='Argument that destination takes', dest='argument')
    parser.add_argument('-t', '--arg-type', type=str, default='nargs', choices=['string', 'nargs'], help='Argument type (choices: string, nargs)', dest='argument_type')
    parser.add_argument('-d', '--arg-default', type=str, default=None, help='Default value for argument', dest='argument_default')
    parser.set_defaults(func=newdest)
    args = parser.parse_args()
    args.func(args)
    sys.exit(0)

if __name__ == '__main__':
    main()
