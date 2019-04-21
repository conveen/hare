#!/usr/bin/env python3
# -*- coding: UTF8 -*-
# bootstrap.py
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

import sys
from os import system, path
from argparse import ArgumentParser

from apps.engine.src.database import *
from apps.common.manager import DBManager

def get_destinations():
    return destinations

def bootstrap(args):
    destinations = list()
    destinations.append("./newdest.py --connect '%s' --url 'https://google.com/search?q=%%s' --fallback --alias google --alias ggl --alias g"%(args.conn_string))
    destinations.append("./newdest.py --connect '%s' --url 'https://duckduckgo.com/?q=%%s' --fallback --alias duckduckgo --alias ddg --alias d"%(args.conn_string))
    destinations.append("./newdest.py --connect '%s' --url 'https://en.wikipedia.org/w/index.php?search=%%s' --alias wp --alias wikipedia -A page -d 'Main_Page' --fallback"%(args.conn_string))
    destinations.append("./newdest.py --connect '%s' --url 'https://github.com' --alias gh --alias ghub --alias github"%(args.conn_string))
    for dest in destinations:
        system(dest)

def main():
    parser = ArgumentParser(prog='bootstrap.py', description='bootstrap Hare DB')
    parser.add_argument('-c', '--connect', type=str, required=True, help='Connection string for database', dest='conn_string')
    parser.set_defaults(func=bootstrap)
    args = parser.parse_args()
    args.func(args)
    sys.exit(0)

if __name__ == '__main__':
    main()
