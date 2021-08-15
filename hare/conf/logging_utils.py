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
from typing import Any, Mapping, Tuple, Union


class LogRecord(logging.LogRecord):
    """Extends base LogRecord implementation to allow for format-style strings."""

    def __init__(  # pylint: disable=too-many-arguments
        self,
        name: str,
        level: int,
        pathname: str,
        lineno: int,
        msg: str,
        args: Union[Tuple[Any, ...], Mapping[str, Any]],
        exc_info: Any,
        func: str = None,
        sinfo: Any = None,
        style: str = "%",
        **kwargs,
    ):
        super().__init__(name, level, pathname, lineno, msg, args, exc_info, func, sinfo, **kwargs)  # type: ignore
        self.style = style

    def getMessage(self) -> str:
        msg = str(self.msg)
        if self.args:
            if self.name.startswith("hare") and self.style == "{":
                msg = msg.format(*self.args)
            else:
                msg = msg % self.args
        return msg


def log_factory(*args, **kwargs) -> LogRecord:
    """Logging factory that uses the LogRecord class with format-style strings."""
    kwargs["style"] = "{"
    return LogRecord(*args, **kwargs)
