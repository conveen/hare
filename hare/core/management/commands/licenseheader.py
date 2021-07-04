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

from pathlib import Path
import typing

from django.core.management import base as command

from hare.core.management.utils import GlobPattern
from hare.core.settings import BASE_DIR
import logging


logger = logging.getLogger(__name__)


def _extract_first_from_str(content: str) -> str:
    """Extract first line from str without splitting by line."""
    if not content:
        return content

    newline_pos = content.find("\n")
    if newline_pos == -1:
        return content
    return content[:newline_pos]


def _prepend_comments_to_lines(content: str, num_symbols: typing.Optional[int] = 2) -> str:
    """Generate new string with one or more hashtags prepended to each line."""
    prepend_str = f"{'#' * num_symbols} "
    return "\n".join(f"{prepend_str}{line}" for line in content.splitlines())


class Command(command.BaseCommand):
    help = "Add/update license header to/in source files."

    def add_arguments(self, parser: command.CommandParser):
        parser.add_argument(
            "-l", "--license",
            type=Path,
            default=BASE_DIR.parent.joinpath("LICENSE"),
            help="Path to LICENSE file",
            dest="license_path",
        )
        parser.add_argument(
            "-p", "--pattern",
            type=GlobPattern,
            default=GlobPattern("**/*.py"),
            help="Glob pattern from root of the package for which files to add license header to",
            dest="source_paths",
        )
        parser.add_argument(
            "-U", "--update",
            action="store_true",
            help="Update the license header in-place instead of appending",
            dest="update_header",
        )

    def handle(self, *args, **options) -> None:
        license_path: Path = options["license_path"]
        license_path = license_path.resolve()
        source_paths: typing.Generator[Path, None, None] = options["source_paths"]
        update_header: bool = options["update_header"]

        if not license_path.is_file():
            logger.error("LICENSE file path does not exist or is not a file ({})", license_path)
            return

        license_content = _prepend_comments_to_lines(license_path.read_text())
        license_first_line = _extract_first_from_str(license_content)
        license_length = len(license_content)

        for source_path in source_paths:
            if not source_path.is_file():
                logger.warning("{} either does not exist or is not file, skipping", source_path)
                continue

            source_content = source_path.read_text()
            source_first_line = _extract_first_from_str(source_content)

            if (not update_header and (not source_content or source_first_line != license_first_line)) or update_header:
                if update_header:
                    logger.info("Updating license header in {}", source_path)
                    source_content = source_content[license_length:]
                else:
                    logger.info("Adding license header to {}", source_path)

                with source_path.open("w") as source_file:
                    source_file.write(f"{license_content}{source_content}")
