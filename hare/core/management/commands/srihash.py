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

from base64 import b64encode as base64_encode
from hashlib import sha384
import itertools
import logging
from pathlib import Path
import typing

from django.core.management import base as command

from hare.core.management.utils import GlobPattern


logger = logging.getLogger(__name__)


def gen_sri_hash(source_path: Path) -> str:
    """Generate SHA-384 subresource integrity hash (SRI).

    See: https://developer.mozilla.org/en-US/docs/Web/Security/Subresource_Integrity
    for more information about SRIs.
    """
    return base64_encode(sha384(source_path.read_bytes()).digest()).decode("utf-8")


class Command(command.BaseCommand):
    help = "Generate SHA-384 subresource integrity hash (SRI) for web assets."

    def add_arguments(self, parser: command.CommandParser):
        parser.add_argument(
            "-p",
            "--pattern",
            type=GlobPattern,
            default=itertools.chain(GlobPattern("**/*.css"), GlobPattern("**/*.js")),
            help="Glob pattern from root of the package for which files to include",
            dest="source_paths",
        )

    def handle(self, *args, **options) -> None:
        source_paths: typing.Generator[Path, None, None] = options["source_paths"]

        for source_path in source_paths:
            if not source_path.is_file():
                logger.warning("{} either does not exist or is not file, skipping", source_path)
                continue

            sri_hash = gen_sri_hash(source_path)
            logger.info("{}: {}", source_path.name, sri_hash)
