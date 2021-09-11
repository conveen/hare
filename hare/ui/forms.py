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
import typing

from django import forms


logger = logging.getLogger(__name__)


class DelimitedCharField(forms.CharField):
    """Form field containing delimited list of values.

    Default delimiter is ``","`` but any delimiter can be used.
    Both single and multi-character delimiters are supported.
    All values in the list are kept as ``str``, so the resulting type is ``List[str]``.
    Unless ``keep_empty_values`` is ``True``, empty values are discarded.
    """
    
    def __init__(self, delimiter=",", keep_empty_values=False, **kwargs) -> None:
        super().__init__(**kwargs)
        self.delimiter = delimiter
        self.keep_empty_values = keep_empty_values

    def to_python(self, value: typing.Any) -> typing.Any:
        value = super().to_python(value)
        if value == self.empty_value:
            return value

        # Need to check if _stripped_ element is empty
        # Using list comprehension here would force two calls to .strip()
        # If keep_empty_values then we short circuit and keep no matter what
        values = []
        for element in value.split(self.delimiter):
            element = element.strip()
            if self.keep_empty_values or element:
                values.append(element)
        return values

    def prepare_value(self, value: typing.Any) -> typing.Any:
        if isinstance(value, list):
            return self.delimiter.join(value)

        return value


class NewDestinationForm(forms.Form):
    """Form to create new destination with alias(es).

    Intended to be used with ``hare.core.models.Destination.objects.create_with_aliases``.
    """
    url = forms.URLField(
        label="URL",
        help_text="Shortcut URL, such as \"https://duckduckgo.com/?q={}\". Use \"{}\" for parameter placeholders.",
        # NOTE: Must be exactly the same as hare.core.models.Destination.url
        max_length=2000,
    )
    description = forms.CharField(
        label="Description",
        help_text=(
            "Description of the shortcut, such as \"Search DuckDuckGo.\" "
            "This will help others understand the shortcut URL and parameters."
        ),
    )
    aliases = DelimitedCharField(
    ## aliases = forms.CharField(
        label="Aliases",
        help_text="One or more aliases for the shortcut separated by commas (i.e., \"d,ddg,duckduckgo\").",
    )
    is_fallback = forms.BooleanField(
        label="Set as fallback",
        help_text=(
            "Whether the shortcut can be used as a fallback. "
            "Fallbacks must take a single parameter, like DuckDuckGo, Google, or Bing."
        ),
        # Setting required=False makes default False but can also be True
        required=False,
    )
