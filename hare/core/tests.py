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

import typing
import unittest

import django.test as django_unittest

from hare.core import models
from hare.core.tests_utils import run_test_units, TestUnit


class TestDestinationManagerUtils(unittest.TestCase):
    """Tests for DestinationManager util functions
    models_utils.validate_netloc_url and models_utils.gen_num_args_from_url.
    """

    def test_models_utils_validate_netloc_url(self) -> None:
        """Tests for ``models_utils.validate_netloc_url``.
        See: ``hare.core.models.models_utils.validate_netloc_url`` for valid URL requirements.
        """
        tests = [
            TestUnit(
                "not_url",
                ValueError,
                models.models_utils.validate_netloc_url,
                "This is not a URL",
                assertion="assertRaises",
            ),
            TestUnit(
                "unc_path",
                ValueError,
                models.models_utils.validate_netloc_url,
                "\\\\fileshare02\\share_name",
                assertion="assertRaises",
            ),
            TestUnit(
                "domain_without_preceding_slashes",
                ValueError,
                models.models_utils.validate_netloc_url,
                "www.python.org",
                assertion="assertRaises",
            ),
            TestUnit(
                "invalid_url_without_preceding_slashes",
                ValueError,
                models.models_utils.validate_netloc_url,
                "www.python.org/downloads/",
                assertion="assertRaises",
            ),
            TestUnit(
                "invalid_url_without_domain",
                ValueError,
                models.models_utils.validate_netloc_url,
                "/downloads/python3.6.9",
                assertion="assertRaises",
            ),
            TestUnit(
                "invalid_url_with_ip_address_without_preceding_slashes",
                ValueError,
                models.models_utils.validate_netloc_url,
                "172.84.99.127:8080/g",
                assertion="assertRaises",
            ),
            TestUnit(
                "amazon_arn",
                ValueError,
                models.models_utils.validate_netloc_url,
                "arn:aws:iam::123456789012:user/username@domain.com",
                assertion="assertRaises",
            ),
            TestUnit(
                "valid_url_without_scheme",
                "http://www.python.org/downloads",
                models.models_utils.validate_netloc_url("//www.python.org/downloads"),
            ),
            TestUnit(
                "valid_url_without_scheme_with_argument",
                "http://www.python.org/downloads/{}",
                models.models_utils.validate_netloc_url("//www.python.org/downloads/{}"),
            ),
            TestUnit(
                "valid_url_without_scheme_with_arguments",
                "http://www.python.org/downloads/{}/{}",
                models.models_utils.validate_netloc_url("//www.python.org/downloads/{}/{}"),
            ),
            TestUnit(
                "valid_url_with_scheme",
                "https://www.virustotal.com/gui/",
                models.models_utils.validate_netloc_url("https://www.virustotal.com/gui/"),
            ),
            TestUnit(
                "valid_url_with_scheme_with_argument",
                "https://www.virustotal.com/gui/ip-address/{}/detection",
                models.models_utils.validate_netloc_url("https://www.virustotal.com/gui/ip-address/{}/detection"),
            ),
            TestUnit(
                "valid_url_with_scheme_with_arguments",
                "https://www.virustotal.com/gui/ip-address/{}/{}",
                models.models_utils.validate_netloc_url("https://www.virustotal.com/gui/ip-address/{}/{}"),
            ),
        ]

        run_test_units(self, tests)

    def test_models_utils_gen_num_args_from_url(self) -> None:
        """Test that ``models_utils.gen_num_args_from_url`` parses the correct
        number of *positional* arguments from format strings.

        Technically ``models_utils.gen_num_args_from_url`` can parse non-URL strings,
        but it is only used after a call to ``models_utils.validate_netloc_url``
        and thus only URL data are tested here.
        """
        tests = [
            TestUnit(
                "no_arguments_with_scheme",
                0,
                models.models_utils.gen_num_args_from_url("https//en.wikipedia.org/w/index.php"),
            ),
            TestUnit(
                "no_arguments_without_scheme",
                0,
                models.models_utils.gen_num_args_from_url("//en.wikipedia.org/w/index.php"),
            ),
            TestUnit(
                "single_positional_argument_in_path_with_scheme",
                1,
                models.models_utils.gen_num_args_from_url("https://www.reddit.com/r/{}"),
            ),
            TestUnit(
                "single_positional_argument_in_path_without_scheme",
                1,
                models.models_utils.gen_num_args_from_url("//www.reddit.com/r/{}"),
            ),
            TestUnit(
                "single_positional_argument_in_query_with_scheme",
                1,
                models.models_utils.gen_num_args_from_url("https://www.shodan.io/search?query={}"),
            ),
            TestUnit(
                "single_positional_argument_in_query_without_scheme",
                1,
                models.models_utils.gen_num_args_from_url("//www.shodan.io/search?query={}"),
            ),
            TestUnit(
                "two_positional_arguments_in_path",
                2,
                models.models_utils.gen_num_args_from_url("https://www.worldtimebuddy.com/{}-to-{}-converter"),
            ),
            TestUnit(
                "ten_positional_arguments_in_path_and_query",
                10,
                models.models_utils.gen_num_args_from_url("https://time.is/{}#{}?query={}{}{}{}{}{}{}{}"),
            ),
            TestUnit(
                "single_keyword_argument",
                ValueError,
                models.models_utils.gen_num_args_from_url,
                "https://en.wikipedia.org/w/index.php?search={search}",
                assertion="assertRaises",
            ),
            TestUnit(
                "two_keyword_arguments",
                ValueError,
                models.models_utils.gen_num_args_from_url,
                "https://en.wikipedia.org/w/index.php?search={search}&title={title}",
                assertion="assertRaises",
            ),
            TestUnit(
                "single_keyword_argument_one_positional",
                ValueError,
                models.models_utils.gen_num_args_from_url,
                "https://en.wikipedia.org/w/index.php?search={search}&title={}",
                assertion="assertRaises",
            ),
        ]

        run_test_units(self, tests)


class TestDestinationManager(django_unittest.TestCase):
    """Tests for the database API implemented in ``DestinationManager``."""

    def setUp(self) -> None:
        self.destinations: typing.Dict[str, models.Destination] = {}

        for url, description, aliases, is_fallback, is_default_fallback in (
            ("https://www.reddit.com/r/{}", "Reddit", ["r", "reddit"], False, False),
            ("https://en.wikipedia.org/w/index.php?search={}", "Wikipedia", ["wp", "wikipedia"], False, False),
            ("https://google.com/search/?q={}", "Google", ["g", "google"], True, False),
            ("https://duckduckgo.com/?q={}", "DuckDuckGo", ["ddg", "duckduckgo"], True, True),
        ):
            destination = models.Destination.objects.create(
                url=url,
                num_args=1,
                is_fallback=is_fallback,
                is_default_fallback=is_default_fallback,
            )
            models.Alias.objects.bulk_create([models.Alias(name=name, destination=destination) for name in aliases])
            self.destinations[description] = destination

    def test_clear_default_fallbacks(self) -> None:
        """Test that ``DestinationManager.clear_default_fallbacks``
        removes existing default fallback destinations.
        """
        models.Destination.objects.clear_default_fallbacks()
        self.assertEqual(
            0,
            len(models.Destination.objects.filter(is_default_fallback=True).all()),
        )

        # Set all destinations as default fallbacks then clear them all
        models.Destination.objects.update(is_default_fallback=True)
        models.Destination.objects.clear_default_fallbacks()
        self.assertEqual(
            0,
            len(models.Destination.objects.filter(is_default_fallback=True).all()),
        )

    def test_create_with_aliases_no_aliases(self) -> None:
        """Test that ``DestinationManager.create_with_aliases`` raises a
        ``ValueError`` when no aliases are provided.
        """
        aliases_list: typing.Tuple[typing.List[str], ...] = ([], [""], ["", ""], ["", "", ""])
        tests = [
            TestUnit(
                f"{len(aliases)}_empty_aliases",
                ValueError,
                models.Destination.objects.create_with_aliases,
                "https://www.youtube.com/results?search_query={}",
                "Youtube",
                aliases,
                False,
                False,
                assertion="assertRaises",
            )
            for aliases in aliases_list
        ]
        run_test_units(self, tests)

    def test_create_with_aliases_duplicate_aliases(self) -> None:
        """Test that ``DestinationManager.create_with_aliases`` dedupes
        aliases before insertion into the database.
        """
        youtube = models.Destination.objects.create_with_aliases(
            "https://www.youtube.com/results?search_query={}",
            "Youtube",
            ["youtube", "youtube"],
            False,
            False,
        )
        aliases = {alias.name for alias in youtube.aliases.all()}
        self.assertEqual({"youtube"}, aliases)

        kaggle = models.Destination.objects.create_with_aliases(
            "https://www.kaggle.com/search?q={}",
            "Kaggle",
            ["kgl", "kaggle", "kgl"],
            False,
            False,
        )
        aliases = {alias.name for alias in kaggle.aliases.all()}
        self.assertEqual({"kgl", "kaggle"}, aliases)

    def test_create_with_aliases_default_fallback_set_is_fallback(self) -> None:
        """Test that ``DestinationManager.create_with_aliases`` sets ``is_fallback``
        to ``True`` if ``is_default_fallback`` is ``True``.
        """
        destination = models.Destination.objects.create_with_aliases(
            "https://scholar.google.com/scholar?q={}",
            "Google Scholar",
            ["gs", "scholar", "googlescholar"],
            False,
            True,
        )
        self.assertTrue(destination.is_fallback)

    def test_create_with_aliases_default_fallback_invalid_arguments(self) -> None:
        """Test that ``DestinationManager.create_with_aliases`` raises a
        ValueError and retains the existing default fallback if the new destination
        is a default fallback and the number of arguments isn't exactly one.
        """
        self.assertRaises(
            ValueError,
            models.Destination.objects.create_with_aliases,
            "https://www.kaggle.com/search",
            "Kaggle",
            ["kgl", "kaggle"],
            True,
            True,
        )
        self.assertEqual(self.destinations["DuckDuckGo"], models.Destination.objects.default_fallback())

    def test_create_with_aliases(self) -> None:
        """Test that ``DestinationManager.create_with_aliases`` creates
        a destination with the provided aliases if all conditions are met.
        """
        url = "https://search.yahoo.com/search?p={}"
        description = "Yahoo Search"
        aliases = ["yh", "yahoo"]
        destination = models.Destination.objects.create_with_aliases(url, description, aliases, True, False)
        self.assertEqual(url, destination.url)
        self.assertEqual(description, destination.description)
        self.assertEqual(set(aliases), {alias.name for alias in destination.aliases.all()})
        self.assertTrue(destination.is_fallback)
        self.assertFalse(destination.is_default_fallback)

    def test_default_fallback_single_destination(self) -> None:
        """Test that ``DestinationManager.default_fallback`` returns the
        correct destination when there's only one default fallback.
        """
        self.assertEqual(self.destinations["DuckDuckGo"], models.Destination.objects.default_fallback())

    def test_default_fallback_multiple_destinations(self) -> None:
        """Test that ``DestinationManager.default_fallback`` returns the
        destination with the lowest ``id`` when there are multiple default fallbacks.
        """
        bing = models.Destination.objects.create(
            url="https://www.bing.com/search?q={}",
            num_args=1,
            description="Bing",
            is_fallback=True,
            is_default_fallback=True,
        )
        models.Alias.objects.create(name="bing", destination=bing)

        self.assertEqual(self.destinations["DuckDuckGo"], models.Destination.objects.default_fallback())

        self.destinations["DuckDuckGo"].is_default_fallback = False
        self.destinations["DuckDuckGo"].save()
        self.assertEqual(bing, models.Destination.objects.default_fallback())

    def test_default_fallback_not_exist(self) -> None:
        """Test that ``DestinationManager.default_fallback`` raises
        ``Destination.DoesNotExist`` error when there are no default fallbacks.
        """
        self.destinations["DuckDuckGo"].is_default_fallback = False
        self.destinations["DuckDuckGo"].save()
        self.assertRaises(models.Destination.DoesNotExist, models.Destination.objects.default_fallback)

    def test_from_alias_correct_destination(self) -> None:
        """Test that ``DestinationManager.from_alias`` returns the correct
        destination by alias.
        """
        for description, alias in (("Reddit", "r"), ("Wikipedia", "wp"), ("Google", "g"), ("DuckDuckGo", "ddg")):
            self.assertEqual(self.destinations[description], models.Destination.objects.from_alias(alias))

    def test_from_alias_invalid_alias(self) -> None:
        """Test that ``DestinationManager.from_alias`` returns ``None`` for nonexistent aliases."""
        self.assertIsNone(models.Destination.objects.from_alias("ddd"))

        # Remove existing Destination then try to retrieve by alias
        # Requires ON DELETE CASCADE to be enabled for aliases
        models.Destination.objects.filter(id=self.destinations["Reddit"].id).delete()
        self.assertIsNone(models.Destination.objects.from_alias("r"))
