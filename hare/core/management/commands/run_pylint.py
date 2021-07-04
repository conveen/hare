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

import typing

from hare.core.management.subprocess_command import SubprocessCommand
from hare.core.settings import BASE_DIR


class Command(SubprocessCommand):
    help = "Run Pylint code linter."
    program = "pylint"

    def run_subprocess(
        self,
        command_extra_args: typing.List[str],
        source_paths: typing.List[str],
        **subprocess_kwargs,
    ) -> None:
        pylintrc_path = f"{BASE_DIR.parent.joinpath('pylintrc')}"
        command_extra_args = ["--rcfile", pylintrc_path] + command_extra_args
        super().run_subprocess(command_extra_args, source_paths)
