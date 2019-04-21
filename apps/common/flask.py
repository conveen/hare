# -*- coding: UTF8 -*-
# flask.py
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

from argparse import Namespace, ArgumentParser, ArgumentTypeError

from flask import jsonify, _request_ctx_stack

class JSONResponse(object):
    '''
    Wrapper class around flask.jsonify
    '''
    def __init__(self, response_data, **kwargs):
        self.response_data = response_data
        self.kwargs = kwargs
    def compile(self):
        try:
            response = jsonify(**self.response_data)
            for kwarg in self.kwargs:
                setattr(response, kwarg, self.kwargs[kwarg])
            return response
        except:
            return None

class RequestParser(ArgumentParser):
    '''
    Class for parsing HTTP request arguments as command line arguments
    using the builtin ArgumentParser class
    '''
    @staticmethod
    def CommaSeparatedList(arg):
        '''
        Args:
            arg: String => parameter to parse
        Returns:
            List<String>
            parameter split by comma into a list
        Preconditions:
            arg is of type String
        '''
        try:
            args = set([item.strip() for item in arg.strip().split(',') if item != ''])
            if len(args) == 0:
                raise Exception('No valid items found in list')
            return args
        except Exception as e:
            raise ArgumentTypeError('Failed to parse CSV list (%s)'%str(e))

    def add_argument(self, urlparam, **kwargs):
        '''
        Args:
            urlparam: String    => name of URL urlparameter
        Procedure:
            @ArgumentParser.add_argument
        Preconditions:
            urlparam is of type String
            urlparam is a valid positional CL parameter
        '''
        super(RequestParser, self).add_argument('--%s'%urlparam, **kwargs)
        return self
    def parse_args(self, ctx=None, namespace=None, drop_unknown=True):
        '''
        Args:
            ctx: flask.ctx.RequestContext   => the context from which to pull
                                               the request arguments
            namespace: Namespace            => existing Namespace object
            drop_unknown: Boolean           => drop uknown args
        Returns:
            @ArgumentParser.parse_args
        Preconditions:
            ctx is of type flask.ctx.RequestContext (assumed True)
            namespace is of type Namespace          (assumed True)
            drop_unknown is of type Boolean (assumed True)
        '''
        if ctx is None:
            if _request_ctx_stack.top is not None:
                ctx = _request_ctx_stack.top
            else:
                raise Exception(\
                    'Request context stack is empty, check to make sure within app context'\
                )
        if ctx.request.method == 'GET':
            args = ctx.request.args.items()
        elif ctx.request.method in ['POST', 'PUT']:
            args = ctx.request.form.items()
        else:
            return Namespace()
        split_args = list()
        for key,value in args:
            split_args.append('--%s'%key)
            split_args.append(value)
        return self.parse_known_args(split_args, namespace, drop_unknown)
    def parse_known_args(self, args=None, namespace=None, drop_unknown=True):
        '''
        Args:
            args: List<String>      => list of arguments to parse
            namespace: Namespace    => existing Namespace object
            drop_unknown: Boolean   => drop uknown args
        Returns:
            @ArgumentParser.parse_known_args
        Preconditions:
            args is of type List<String>    (assumed True)
            namespace is of type Namespace  (assumed True)
            drop_unknown is of type Boolean (assumed True)
        '''
        pnamespace, pargs = super(RequestParser, self).parse_known_args(args, namespace)
        return pnamespace if drop_unknown else (pnamespace, pargs)
    def error(self, message):
        '''
        Args:
            message: String => error message
        Procedure:
            Raises exception with message as error message
            Overwrites ArgumentParser.error which prints to stderr and exits
        Preconditions:
            message is of type String   (assumed True)
        '''
        raise Exception('Failed to parse provided arguments (%s)'%message)
