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

from flask import make_response, render_template, redirect, url_for
import requests

from apps.common.routes import *

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
    Index route: redirect to /manage
    '''
    _ROUTE      = '/'
    _METHODS    = ['GET']

    @staticmethod
    def _make_response(current_ctx):
        '''
        @BaseRoute._make_response
        '''
        return redirect(url_for('manage'))

class ManageRoute(BaseRoute):
    '''
    Main route: serve webpage to add new destination
    '''
    _ROUTE      = '/manage'
    _METHODS    = ['GET']

    @staticmethod
    def _make_response(current_ctx):
        '''
        @BaseRoute._make_response
        '''
        return make_response(render_template('manage.html'))

class NewRoute(BaseRoute):
    '''
    Route to register new destination
    '''
    _ROUTE      = '/new'
    _METHODS    = ['POST']

    @staticmethod
    def _make_response(current_ctx):
        '''
        @BaseRoute._make_response
        '''
        try:
            res = requests.post(\
                '%s/new'%(current_ctx.app.config.get('HARE_ENGINE_HOST').rstrip('/')),
                data = current_ctx.request.form.to_dict()\
            )
        except:
            pass
        return redirect(url_for('manage'))
