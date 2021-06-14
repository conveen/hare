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


# Default application secret key path is .app_secret.key in hare_engine directory.
# NOTE: This file should _never_ be checked in to source control, and should be
#       fetched from secret manager or env variables in production environments.
# See: https://flask.palletsprojects.com/en/1.1.x/quickstart/#sessions
try:
    SECRET_KEY = (Path(__file__)
                  .with_name(".app_secret.key")
                  .read_text())
except FileNotFoundError:
    SECRET_KEY = ""

CONFIG = {
    # Default database connection URL is SQLite database in hare-engine directory
    "HARE_DATABASE_URL": "sqlite:///{}".format(Path(__file__)
                                               .parent
                                               .joinpath("hare.db")
                                               .resolve()),
    "HARE_DATABASE_BOOTSTRAP_ON_STARTUP": True,
    "HARE_SECRET_KEY": SECRET_KEY,
}
