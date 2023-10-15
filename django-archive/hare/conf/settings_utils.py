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
from pathlib import Path
import typing

from django.core.exceptions import ImproperlyConfigured

from hare.conf import logging_utils


ENV_VAR_PREFIX = "HARE"
# Only support SQLite3 and Postgres (with Psycopg2 driver)
SUPPORTED_DATABASES = {
    "sqlite3": "django.db.backends.sqlite3",
    "postgres": "django.db.backends.postgres",
}


def gen_allowed_hosts_setting() -> typing.List[str]:
    """ALLOWED_HOSTS setting."""
    allowed_hosts = ENV.get(f"{ENV_VAR_PREFIX}_ALLOWED_HOSTS")
    if not allowed_hosts:
        return []

    return [host for host in allowed_hosts.split(",") if host]


def gen_databases_setting(base_dir: Path) -> typing.Dict[str, str]:
    """DATABASES setting."""
    sqlite3_engine = SUPPORTED_DATABASES["sqlite3"]
    postgres_engine = SUPPORTED_DATABASES["postgres"]
    engine = ENV.get(f"{ENV_VAR_PREFIX}_DB_ENGINE", sqlite3_engine)

    if engine == sqlite3_engine:
        name = ENV.get(f"{ENV_VAR_PREFIX}_DB_NAME", str(base_dir.joinpath("hare.db")))
        return {"ENGINE": engine, "NAME": name}

    if engine != postgres_engine:
        raise ImproperlyConfigured(f"Unsupported database engine {engine}")

    name = ENV.get(f"{ENV_VAR_PREFIX}_DB_NAME", "hare")
    port = ENV.get(f"{ENV_VAR_PREFIX}_DB_PORT", "5432")
    host = ENV.get(f"{ENV_VAR_PREFIX}_DB_HOST")
    if not host:
        raise ImproperlyConfigured("Must supply database host")
    user = ENV.get(f"{ENV_VAR_PREFIX}_DB_USER")
    if not user:
        raise ImproperlyConfigured("Must supply database user")
    password = ENV.get(f"{ENV_VAR_PREFIX}_DB_PASSWORD")
    if not password:
        raise ImproperlyConfigured("Must supply database password")
    return {
        "ENGINE": engine,
        "NAME": name,
        "HOST": host,
        "PORT": port,
        "USER": user,
        "PASSWORD": password,
    }


def gen_debug_setting() -> bool:
    """DEBUG setting."""
    environment = ENV.get(f"{ENV_VAR_PREFIX}_ENV", "development")
    return environment != "production"


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
                "level": "DEBUG" if debug else ENV.get(f"{ENV_VAR_PREFIX}_LOG_LEVEL", "INFO"),
                "propogate": True,
            },
        },
    }
