## MIT License
## 
## Copyright (c) 2021 conveen
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

import logging
from os import environ as ENV
import typing

from hare.core import logging_utils


def gen_logging_setting(debug: bool) -> typing.Dict[str, typing.Any]:
    """LOGGING setting."""
    # Allow "{" style formatting with logging.* methods
    logging.setLogRecordFactory(logging_utils.log_factory)
    return {
        "version": 1,
        "disable_existing_loggers": False,
        "formatters": {
            "debug": {
                "()": "django.utils.log.ServerFormatter",
                "format": "[{asctime}] {levelname} {name}:{lineno} {message}",
                "datefmt": "%d/%b/%Y %H:%M:%S",
                "style": "{",
            },
            "production": {
                "()": "django.utils.log.ServerFormatter",
                "format": "[{asctime}] {levelname} {message}",
                "datefmt": "%d/%b/%Y %H:%M:%S",
                "style": "{",
            },
        },
        "handlers": {
            "console": {
                "class": "logging.StreamHandler",
                "formatter": "debug" if debug else "production",
            },
        },
        "loggers": {
            "hare": {
                "handlers": ["console"],
                "level": "DEBUG" if debug else ENV.get("HARE_LOG_LEVEL", "INFO"),
                "propogate": True,
            },
        },
    }
