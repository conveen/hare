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
import unittest

import django.test as django_unittest


class TestUnit:
    """Single unit test.

    Uses the ``TestCase.assert*`` methods to compare a test
    value to an expected value. Intended to be used within a
    ``TestCase`` to enable running a sequence of tests without
    creating a separate ``test_<name>`` function for each.
    """

    __slots__ = (
        "args",
        "assertion",
        "expected_value",
        "kwargs",
        "name",
    )

    def __init__(
        self,
        name: str,
        expected_value: typing.Any,
        *args,
        assertion: str = "assertEqual",
        **kwargs,
    ) -> None:
        self.name = name
        self.expected_value = expected_value
        self.assertion = assertion
        self.args: typing.Tuple[typing.Any, ...] = args
        self.kwargs: typing.Dict[str, typing.Any] = kwargs

    def run_test(
        self,
        test_instance: typing.Union[django_unittest.TestCase, unittest.TestCase],
    ) -> None:
        """Run unit test with provided ``TestCase`` instance."""
        with test_instance.subTest(test_name=self.name):
            assertion = getattr(test_instance, self.assertion)
            assertion(self.expected_value, *self.args, **self.kwargs)


def run_test_units(
    test_instance: typing.Union[django_unittest.TestCase, unittest.TestCase],
    test_units: typing.Sequence[TestUnit],
) -> None:
    """Run sequence of test units."""
    for test_unit in test_units:
        test_unit.run_test(test_instance)
