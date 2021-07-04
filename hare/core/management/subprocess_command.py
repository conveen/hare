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
import subprocess
import typing

from django.core.management import base as command

from hare.core.management.utils import GlobPattern


class SubprocessCommand(command.BaseCommand):
    program: typing.ClassVar[str] = ""

    def add_arguments(self, parser: command.CommandParser):
        parser.add_argument(
            "-p",
            "--pattern",
            type=GlobPattern,
            default=GlobPattern("**/*.py"),
            help="Glob pattern from root of the package for which files to include",
            dest="source_paths",
        )
        parser.add_argument(
            "command_extra_args",
            nargs="*",
            help=f"Passthrough arguments to {self.program}",
        )

    def run_subprocess(
        self,
        command_extra_args: typing.List[str],
        source_paths: typing.List[str],
        **subprocess_kwargs,
    ) -> None:
        """Run program with arguments and source paths."""
        subprocess.run([self.program] + command_extra_args + source_paths, check=True, **subprocess_kwargs)

    def handle(self, *args, **options) -> None:
        if not self.program:
            raise ValueError("must supply program to run")

        command_extra_args: typing.List[str] = options["command_extra_args"]
        source_paths_gen: typing.Generator[Path, None, None] = options["source_paths"]
        source_paths: typing.List[str] = [f"{source_path}" for source_path in source_paths_gen]

        self.run_subprocess(command_extra_args, source_paths)
