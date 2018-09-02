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

from functools import wraps
from flask import _request_ctx_stack

from apps.common.flask import JSONResponse

class RouteRegistryMixin(object):
    '''
    '''
    _REGISTRY = None

    @classmethod
    def registry(cls):
        return cls._REGISTRY
    @classmethod
    def retrieve(cls, name):
        return cls.registry().get(name)
    @classmethod
    def _add_class(cls, new_cls):
        if not hasattr(new_cls, '__name__') or new_cls.__name__ is None:
            return False
        if not hasattr(new_cls, '_ROUTE') or new_cls._ROUTE is None:
            return False
        if not hasattr(new_cls, '_METHODS') or new_cls._METHODS is None:
            return False
        if not hasattr(new_cls, 'response') or not callable(new_cls.response):
            return False
        endpoint_name = new_cls.__name__.replace('Route', '').lower()
        setattr(new_cls, endpoint_name, lambda *args, **kwargs: new_cls.response(*args, **kwargs))
        getattr(new_cls, endpoint_name).__name__ = endpoint_name
        cls.registry()[new_cls.__name__] = new_cls
    @classmethod
    def initialize(cls, app):
        for key in cls.registry():
            route = cls.retrieve(key)
            app.route(route._ROUTE, methods=route._METHODS)(getattr(route, route.__name__.replace('Route', '').lower()))
    
    def __new__(cls, name, bases, attrs):
        new_cls = type.__new__(cls, name, bases, attrs)
        cls._add_class(new_cls)
        return new_cls

class BaseRouteMixin(object):
    '''
    '''
    _ROUTE      = None
    _METHODS    = None

    @staticmethod
    def _adapt_request():
        '''
        '''
        return _request_ctx_stack.top if _request_ctx_stack.top is not None else None
    @staticmethod
    def _make_response(current_ctx, *args, **kwargs):
        '''
        '''
        return None
    @staticmethod
    def _adapt_response(response):
        '''
        '''
        response.headers['Server'] = 'Hare v1.0'
        return response
    @classmethod
    def response(cls, *args, **kwargs):
        '''
        '''
        try:
            current_ctx = cls._adapt_request()
            response = cls._make_response(current_ctx, *args, **kwargs)
        except:
            try:
                current_ctx.app.dbm.session.rollback()
            except:
                pass
            return None
        else:
            return cls._adapt_response(response)
