#!/usr/bin/env python3
## -*- coding: UTF8 -*-
## bootstrap.py
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

import hare_engine.src.database as db


__author__ = "conveen"
logger = logging.getLogger(__name__)


def bootstrap(args: Namespace) -> int:
    """
    Args:
        args    => Arguments parsed from command line (see: gen_command_parser)
    Description:
        Bootstrap Hare database with common destinations.
    Preconditions:
        N/A
    Raises:
        N/A
    """
    try:
        dbm = DBManager(args.connection_url, db.BaseTable.metadata).connect()
        logger.info("Successfully connected to database")
    except Exception as exc:
        logger.error("Failed to connect to database ({})".format(exc))
        return 1
    try:
        dbm.bootstrap_db()
    except Exception as exc:
        logger.error("Failed to create tables and indexes in database ({})".format(exc))
        return 1
    try:
        dbm.gen_session()
    except Exception as exc:
        logger.error("Failed to create database session ({})".format(exc))
        return 1

    destinations = (
        ("https://https://duckduckgo.com/?q={}", "Search DuckDuckGo", ["d", "ddg", "duckduckgo"], True, not args.google_default),
        ("https://www.google.com/search?q={}", "Search Google", ["g", "google"], True, args.google_default),
        ("https://en.wikipedia.org/w/index.php?search={}", "Search Wikipedia", ["w", "wp", "wikipedia"], False, False),
        ("https://www.reddit.com/r/{}", "Reddit - browse to specific subreddit", ["r", "reddit"], False, False),
        ("https://www.reddit.com/search/?q={}", "Search Reddit", ["rs", "reddits", "redditsearch"], False, False),
        ("https://twitter.com/search?q={}", "Search Twitter", ["tw", "twitter"], False, False),
        ("https://twitter.com/{}", "Twitter - browse to specific user", ["twu", "twitteruser"], False, False),
        ("https://facebook.com/{}", "Facebook - browse to specific user", ["fbu", "facebookuser"], False, False),
        ("https://www.youtube.com/results?search_query={}", "Search YouTube", ["yt", "youtube"], False, False),
        ("https://www.youtube.com/c/{}", "YouTube - browse to specific channel", ["ytc", "youtubechannel"], False, False),
        ("https://time.gov/?t=24", "National Institute of Standards and Technology (NIST) Official Time", ["time"], False, False),
        ("https://www.worldtimebuddy.com/{}-to-{}-converter", "Convert between two timezones", ["tzc", "converttz"], False, False),
        ("https://www.nytimes.com", "New York Times - main page", ["nyt", "newyorktimes"], False, False),
        ("https://www.nytimes.com/search?query={}", "Search New York Times", ["nyts", "newyorktimessearch"], False, False),
        ("https://cnn.com/", "CNN - main page", ["cnn"], False, False),
        ("https://www.cnn.com/search?q={}", "Search CNN", ["cnns", "cnnsearch"], False, False),
        ("https://www.foxnews.com/", "Fox News - main page", ["fox", "foxnews"], False, False),
        ("https://www.foxnews.com/search-results/search?q={}", "Search Fox News", ["foxs", "foxnewssearch"], False, False),
        ("https://apnews.com/", "Associated Press News - main page", ["apn", "apnews"], False, False),
        ("https://apnews.com/{}", "Associated Press - browse to specific topic", ["apnt", "apnewstopic"], False, False),
        ("https://www.npr.org/", "NPR News - main page", ["npr"], False, False),
        ("https://www.npr.org/search?query={}", "Search NPR News", ["nprs"], False, False),
    )

    # For each destination
    for url, description, aliases, is_fallback, is_default_fallback in destinations:
        # Try to add destination to database
        logger.info("Attempting to add destination {} with aliases {}".format(url, aliases))
        try:
            db.add_destination_with_aliases(dbm, url, description, aliases, is_fallback, is_default_fallback)
            logger.info("Successfully added destination to database")
        except Exception as exc:
            if isinstance(exc, ValueError):
                logger.warning("Arguments failed validation ({})".format(value_exc))
            else:
                logger.warning("Failed to add destination to database ({})".format(exc))
    return 0


def gen_command_parser() -> ArgumentParser:
    parser = ArgumentParser(description=("Bootstrap Hare database with common destinations like "
                                         "Google, DuckDuckGo, Wikipedia, etc."))
    parser.add_argument("connection_url", help="Database connection URL")
    parser.add_argument("-g",
                        "--google",
                        action="store_true",
                        help="Set Google as default fallback instead of DuckDuckGo (default: False)",
                        dest="google_default")
    parser.set_defaults(func=bootstrap)
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
