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

from string import Formatter
import typing
from urllib.parse import urlsplit, urlunsplit

from django.core.validators import URLValidator
from django.db import models


########################
##### Database API #####
########################

######################################
##### Database Schema Definition #####
######################################


class Destination(models.Model):
    """destination table.

    A destination represents a unique _URL_ and set of zero or more parameters.
    For example, a destination could be https://www.youtube.com/results?search_query={},
    where '{}' is a single parameter. Destinations could also have no
    parameters, like https://time.is/, or have parameters as part of the URL (not a URL parameter).

    Fallback destinations are URLs with a single parameter, and are used if the specified
    destination does not exist, or there is otherwise an error in redirecting an a destination.
    Common choices could be DuckDuckGo, Wikipedia, or Google.
    """

    url = models.CharField(max_length=2000, unique=True, validators=[URLValidator()])
    num_args = models.IntegerField()
    is_fallback = models.BooleanField(db_index=True)
    is_default_fallback = models.BooleanField(db_index=True, default=False)
    description = models.TextField()

    class Meta:
        db_table = "destination"


class Alias(models.Model):
    """alias table.

    Aliases act as references, or pointers, to destinations, and are the shortcuts users type to
    get redirected to a destination. For example, the shortcuts "ddg", "duckduckgo", and "go" could
    all reference a search on DuckDuckGo.

    There is a one-to-many relationship between destination and alias, and a one-to-one relationship
    between alias and destination. In other words, a destination can have many aliases, but an alias
    must uniquely point to a single destination (enforced by a unique constraint on dest_id and name).
    """

    destination = models.ForeignKey(
        "Destination",
        on_delete=models.CASCADE,
        related_name="aliases",
        related_query_name="alias",
    )
    name = models.CharField(max_length=100, unique=True)

    class Meta:
        db_table = "alias"
