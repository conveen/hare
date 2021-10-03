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
import multiprocessing as mp
from pathlib import Path
import typing

from django.core.management import base as command
from django.core.management.color import no_style

from hare.core.management import subprocess_command
from hare.core.management.commands import (
    run_black,
    run_mypy,
    run_pylint,
)


logger = logging.getLogger(__name__)

NO_STYLE = no_style()
TOOLS = {
    "black": run_black.Command,
    "mypy": run_mypy.Command,
    "pylint": run_pylint.Command,
}


class DelayedExecutor:
    """Curry Django command execution by first providing state (args, kwargs)."""

    __slots__ = (
        "args",
        "command_cls",
        "kwargs",
    )

    def __init__(self, *args, **kwargs) -> None:
        self.args = args
        self.kwargs = kwargs

    def __call__(self, command_cls: typing.Type[command.BaseCommand]) -> None:
        command_cls().execute(*self.args, **self.kwargs)


class Command(subprocess_command.SubprocessCommand):
    help = "Run CI tools on source files - Black, Pylint, and MyPy."

    def handle(self, *args, **options) -> None:
        source_paths_gen: typing.Generator[Path, None, None] = options["source_paths"]
        source_paths: typing.List[str] = [f"{source_path}" for source_path in source_paths_gen]

        options.pop("stdout", None)
        options.pop("stderr", None)
        options["source_paths"] = source_paths

        with mp.Pool(processes=len(TOOLS)) as pool:
            pool.map(DelayedExecutor(**options), TOOLS.values())
