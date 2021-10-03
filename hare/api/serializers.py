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

from rest_framework import serializers

from hare.core import models
from hare.core import models_utils


logger = logging.getLogger(__name__)
RequestData = typing.Dict[str, typing.Any]


class AliasSerializer(serializers.ModelSerializer):
    """Serializer for ``models.Alias``."""

    class Meta:
        model = models.Alias
        fields = ["id", "name"]


class ShortcutSerializer(serializers.ModelSerializer):
    """Serializer for reading a shortcut.

    A shortcut is defined as a ``models.Destination`` and its associated ``models.Alias`` objects.
    """

    aliases = AliasSerializer(many=True)

    def validate_url(self, value: str) -> str:  # pylint: disable=no-self-use
        """Validate the URL according to ``models_utils.validate_netloc_url``."""
        try:
            return models_utils.validate_netloc_url(value)
        except ValueError as exc:
            raise serializers.ValidationError("Invalid URL") from exc

        return value

    def validate(self, attrs: RequestData) -> RequestData:
        """Validate URL arguments and parse ``num_args``."""
        try:
            attrs["num_args"] = models_utils.gen_num_args_from_url(attrs["url"])
        except ValueError as exc:
            raise serializers.ValidationError(
                {
                    "url": "URL must not contain keyword arguments (i.e., '{arg1}')",
                }
            ) from exc

        return attrs

    def create(self, validated_data: RequestData) -> models.Destination:
        aliases = validated_data.pop("aliases")
        destination = models.Destination.objects.create(**validated_data)
        models.Alias.objects.bulk_create([models.Alias(**alias, destination=destination) for alias in aliases])
        return destination

    def update(self, instance: models.Destination, validated_data: RequestData) -> models.Destination:
        validated_data.pop("aliases", None)
        return super().update(instance, validated_data)

    class Meta:
        model = models.Destination
        fields = ["id", "url", "num_args", "is_fallback", "is_default_fallback", "description", "aliases"]
        extra_kwargs = {
            # num_args is parsed from url on each POST/PUT
            "num_args": {"read_only": True},
            # Cannot be set via API, must be set by DB query
            "is_default_fallback": {"read_only": True},
        }
