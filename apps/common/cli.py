# -*- coding: UTF8 -*-
# cli.py
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

from argparse import ArgumentParser

def initialize_parser(prog, description):
    parser = ArgumentParser(prog=prog, description=description)
    parser.add_argument(
        '--host',
        type=str,
        default='127.0.0.1',
        help='IP address or hostname to listen on',
        dest='host'
    )
    parser.add_argument(
        '--port',
        type=int,
        default=5000,
        help='Port to listen on',
        dest='port'
    )
    parser.add_argument(
        '--debug',
        action='store_true',
        help='Whether to run server in debug mode (default: False)',
        dest='debug'
    )
    parser.add_argument(
        '--ssl',
        action='store_true',
        help='Whether to run server in https mode (default: False)',
        dest='ssl'
    )
    return parser
