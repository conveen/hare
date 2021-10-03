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

from django.shortcuts import get_object_or_404
from rest_framework import generics, status
from rest_framework.request import Request as APIRequest
from rest_framework.response import Response as APIResponse

from hare.api import serializers
from hare.core import models


logger = logging.getLogger(__name__)


class ListCreateShortcut(generics.ListCreateAPIView):
    """List all shortcuts or create a new one."""

    queryset = models.Destination.objects.all()
    serializer_class = serializers.ShortcutSerializer


class GetUpdateDeleteShortcut(generics.RetrieveUpdateDestroyAPIView):
    """Get, update, or delete shortcut."""

    queryset = models.Destination.objects.all()
    serializer_class = serializers.ShortcutSerializer


class CreateDeleteAlias(generics.CreateAPIView, generics.DestroyAPIView):
    """Create or delete alias for shortcut."""

    queryset = models.Alias.objects.all()
    serializer_class = serializers.AliasSerializer

    def destination_id(self) -> int:
        lookup_url_kwarg = self.lookup_url_kwarg or self.lookup_field
        return self.kwargs[lookup_url_kwarg]

    def alias_name(self) -> str:
        return self.kwargs["name"]

    def get_object(self) -> models.Alias:
        queryset = self.filter_queryset(self.get_queryset())

        alias = get_object_or_404(
            queryset,
            destination_id=self.destination_id(),
            name=self.alias_name(),
        )

        self.check_object_permissions(self.request, alias)

        return alias

    def create(self, request: APIRequest, *args, **kwargs) -> APIResponse:
        serializer = self.get_serializer(data={"name": self.alias_name()})
        serializer.is_valid(raise_exception=True)

        serializer.save(destination_id=self.destination_id())
        return APIResponse(serializer.data, status=status.HTTP_201_CREATED)
