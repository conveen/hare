# -*- coding: UTF8 -*-
# routes.py
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

from flask import redirect, send_from_directory
from urllib.parse import quote_plus as uriencode

from apps.common.routes import *
from apps.common.flask import JSONResponse, RequestParser
import apps.engine.src.database as db

class RouteRegistry(RouteRegistryMixin, type):
    '''
    '''
    _REGISTRY = dict()

class BaseRoute(BaseRouteMixin, metaclass=RouteRegistry):
    '''
    '''
    pass

class IndexRoute(BaseRoute):
    '''
    Main route: resolve alias and args to URL string
    '''
    _ROUTE      = '/'
    _METHODS    = ['GET']

    @staticmethod
    def _resolve_alias(current_ctx, alias):
        '''
        '''
        return current_ctx.app.dbm.query(db.Destination)\
            .join(db.Alias)\
            .filter(db.Alias.name == alias).first()
    @staticmethod
    def _direct_to_fallback(current_ctx, fallback, query):
        '''
        '''
        destination = None
        if fallback is not None:
            destination = current_ctx.app.dbm.query(db.Destination, is_fallback=True)\
                .join(db.Alias)\
                .filter(db.Alias.name == fallback).first()
        if destination is None:
            destination = current_ctx.app.dbm.query(db.Destination, is_fallback=True)\
                .order_by(db.Destination.priority).first()
        return redirect(destination.url%uriencode(query), code=302)
    @staticmethod
    def _direct_to_destination(destination, arguments):
        '''
        '''
        # NOTE: current version only allows one argument per destination,
        # but future versions may change to allow arbitrary number of arguments
        if len(destination.args) > 0 or destination.is_fallback:
            if arguments is not None:
                arg = ' '.join(arguments).strip()
            else:
                arg = destination.args[0].default
            return redirect(destination.url%uriencode(arg), code=302)
        return redirect(destination.url, code=302)

    @classmethod
    def _make_response(cls, current_ctx):
        '''
        @BaseRoute._make_response
        '''
        try:
            args = RequestParser(add_help=False)\
                .add_argument('fallback', type=str)\
                .add_argument('query', type=str)\
                .parse_args()
        except:
            # if can't parse URL params, redirect to global default
            return redirect('https://www.w3.org/Protocols/rfc2616/rfc2616-sec10.html', code=302)
        # if not query specified, redirect to global default
        if args.query is None:
            return redirect('https://www.w3.org/Protocols/rfc2616/rfc2616-sec10.html', code=302)
        split_query = args.query.strip().split(' ')
        # get alias provided by user
        args.alias = split_query.pop(0)
        # if arguments provided by user, capture them
        if len(split_query) > 0:
            args.args = split_query
        else:
            args.args = None
        destination = cls._resolve_alias(current_ctx, args.alias)
        if destination is None:
            return cls._direct_to_fallback(current_ctx, args.fallback, args.query)
        else:
            return cls._direct_to_destination(destination, args.args)

class NewDestinationRoute(BaseRoute):
    '''
    Route to create new destination
    '''
    _ROUTE      = '/new'
    _METHODS    = ['POST']

    @staticmethod
    def _make_response(current_ctx):
        '''
        @BaseRoute._make_response
        '''
        try:
            args = RequestParser(add_help=False)\
                .add_argument('url', type=str, required=True)\
                .add_argument('is_fallback', type=bool, default=False)\
                .add_argument('aliases', type=RequestParser.CommaSeparatedList, required=True)\
                .add_argument('argument', type=str)\
                .add_argument('argument_type', type=str, default='nargs')\
                .add_argument('argument_default', type=str)\
                .parse_args()
            if len(args.argument) == 0:
                args.argument = None
        except Exception as e:
            return JSONResponse({\
                'err':'Invalid or insufficient request parameters'\
            }, status_code=412).compile()
        print(args)
        priority = None
        if args.is_fallback:
            last_fallback = current_ctx.app.dbm\
                .query(db.Destination)\
                .order_by(db.Destination.priority.desc())\
                .first()
            if last_fallback is None:
                priority = 1
            else:
                priority = last_fallback.priority + 1
        destination = db.Destination(\
            url         = args.url,
            is_fallback = args.is_fallback,
            priority    = priority\
        )
        destination.aliases = [ db.Alias(name=alias) for alias in args.aliases ]
        if args.argument is not None:
            destination.args = [ db.Argument(name=args.argument, type=args.argument_type, default=args.argument_default) ]
        try:
            current_ctx.app.dbm.add(destination)
            current_ctx.app.dbm.commit()
            return JSONResponse({\
                'msg':'Successfully added destination to database'\
            }, status_code=200).compile()
        except Exception as e:
            print(str(e))
            current_ctx.app.dbm.rollback()
            return JSONResponse({\
                'err':'Failed to add destination to database'\
            }, status_code=500).compile()
