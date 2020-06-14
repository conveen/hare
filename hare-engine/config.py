## -*- coding: UTF8 -*-
## config.py
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

from pathlib import Path


__author__ = "conveen"


class DefaultConfig:
    """Default configuration object.
    See: https://flask.palletsprojects.com/en/1.1.x/api/#flask.Config.from_object
    """

    # Default database connection URL is SQLite database in local directory
    HARE_DATABASE_URI = "sqlite:///{}".format(Path(__file__).joinpath("hare.db").absolute())

    # Attempt to create all tables in database on application startup
    HARE_DATABASE_BOOTSTRAP_ON_STARTUP = True

    # Default application secret key path is ".app_secret.key" in local directory
    # NOTE: This file should _never_ be checked in to source control, and should be
    #       fetched from secret manager or encrypted file in production environments
    # See: https://flask.palletsprojects.com/en/1.1.x/quickstart/#sessions
    HARE_SECRET_KEY = str(Path(__file__).joinpath(".app_secret.key").absolute())
