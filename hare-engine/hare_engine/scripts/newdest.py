#!/usr/bin/env python3
## -*- coding: UTF8 -*-
## newdest.py
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

from argparse import ArgumentParser, Namespace
import logging
from pathlib import Path
import sys
# Add hare_engine directory to path, assuming this script is in hare_engine/scripts
sys.path.insert(0, str(Path(__file__).parent.parent.parent.absolute()))

from lc_sqlalchemy_dbutils.manager import DBManager

import hare_engine.database as db


__author__ = "conveen"
__version__ = "0.1.0"
logger = logging.getLogger(__name__)


def newdest(args: Namespace) -> int:
    """
    Args:
        args    => Arguments parsed from command line (see: gen_command_parser)
    Description:
        Add new destination to database.
    Preconditions:
        N/A
    Raises:
        N/A
    """
    try:
        dbm = DBManager(args.connection_url, db.BaseTable.metadata).connect()
        dbm.gen_session()
        logger.info("Successfully connected to database")
    except Exception as exc:
        logger.error("Failed to connect to database ({})".format(exc))
        return 1

    logger.info("Attempting to add destination {} with aliases {}".format(args.destination_url, args.aliases))
    try:
        db.add_destination_with_aliases(dbm,
                                        args.destination_url,
                                        args.description,
                                        args.aliases,
                                        args.is_fallback,
                                        args.is_default_fallback)
        logger.info("Successfully added destination to database")
    except Exception as exc:
        if isinstance(exc, ValueError):
            logger.error("Arguments failed validation ({})".format(value_exc))
        else:
            logger.error("Failed to add destination to database ({})".format(exc))
        return 1
    return 0


def gen_command_parser() -> ArgumentParser:
    """Generate command line parser."""
    parser = ArgumentParser(description="Add new destination with alias(es) to database")
    parser.add_argument("-V", "--version", action="version", version="%(prog)s {}".format(__version__))
    parser.add_argument("connection_url", help="Database connection URL")
    parser.add_argument("destination_url", help="Destination URL")
    parser.add_argument("description", help="Destination description")
    parser.add_argument("-a",
                        "--alias",
                        required=True,
                        action="append",
                        help="Alias(es) for destination",
                        dest="aliases")
    parser.add_argument("--fallback",
                        action="store_true",
                        help="Whether destination is fallback destination (default: False)",
                        dest="is_fallback")
    parser.add_argument("--default-fallback",
                        action="store_true",
                        help="Whether destination is the default fallback destination (default: False)",
                        dest="is_default_fallback")
    parser.set_defaults(func=newdest)
    return parser


def main() -> int:
    logging.basicConfig(datefmt="%Y-%m-%d %H:%M:%S",
                        format="%(asctime)s\t%(levelname)s\t%(module)s:%(lineno)s\t%(message)s",
                        stream=sys.stdout,
                        level=logging.INFO)
    parser = gen_command_parser()
    args = parser.parse_args()
    return args.func(args)


if __name__ == "__main__":
    main()
