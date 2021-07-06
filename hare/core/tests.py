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
# pylint: disable=protected-access

import unittest

import django.test as django_unittest

from hare.core import models
from hare.core.tests_utils import run_test_units, TestUnit


class TestDestinationManagerUtils(unittest.TestCase):
    """Test DestinationManager util functions."""

    def test_validate_netloc_url(self) -> None:
        """Tests for _validate_netloc_url."""
        tests = [
            TestUnit("not_url", ValueError, models._validate_netloc_url, "This is not a URL", assertion="assertRaises"),
            TestUnit(
                "unc_path",
                ValueError,
                models._validate_netloc_url,
                "\\\\fileshare02\\share_name",
                assertion="assertRaises",
            ),
            TestUnit(
                "domain_without_preceding_slashes",
                ValueError,
                models._validate_netloc_url,
                "www.python.org",
                assertion="assertRaises",
            ),
            TestUnit(
                "invalid_url_without_preceding_slashes",
                ValueError,
                models._validate_netloc_url,
                "www.python.org/downloads/",
                assertion="assertRaises",
            ),
            TestUnit(
                "invalid_url_without_domain",
                ValueError,
                models._validate_netloc_url,
                "/downloads/python3.6.9",
                assertion="assertRaises",
            ),
            TestUnit(
                "invalid_url_with_ip_address_without_preceding_slashes",
                ValueError,
                models._validate_netloc_url,
                "172.84.99.127:8080/g",
                assertion="assertRaises",
            ),
            TestUnit(
                "amazon_arn",
                ValueError,
                models._validate_netloc_url,
                "arn:aws:iam::123456789012:user/username@domain.com",
                assertion="assertRaises",
            ),
            TestUnit(
                "valid_url_without_scheme",
                "http://www.python.org/downloads",
                models._validate_netloc_url("//www.python.org/downloads"),
            ),
            TestUnit(
                "valid_url_without_scheme_with_argument",
                "http://www.python.org/downloads/{}",
                models._validate_netloc_url("//www.python.org/downloads/{}"),
            ),
            TestUnit(
                "valid_url_without_scheme_with_arguments",
                "http://www.python.org/downloads/{}/{}",
                models._validate_netloc_url("//www.python.org/downloads/{}/{}"),
            ),
            TestUnit(
                "valid_url_with_scheme",
                "https://www.virustotal.com/gui/",
                models._validate_netloc_url("https://www.virustotal.com/gui/"),
            ),
            TestUnit(
                "valid_url_with_scheme_with_argument",
                "https://www.virustotal.com/gui/ip-address/{}/detection",
                models._validate_netloc_url("https://www.virustotal.com/gui/ip-address/{}/detection"),
            ),
            TestUnit(
                "valid_url_with_scheme_with_arguments",
                "https://www.virustotal.com/gui/ip-address/{}/{}",
                models._validate_netloc_url("https://www.virustotal.com/gui/ip-address/{}/{}"),
            ),
        ]

        run_test_units(self, tests)
