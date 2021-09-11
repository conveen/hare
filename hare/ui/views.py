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

from collections import OrderedDict
import logging
import typing

from django.db import DatabaseError
from django.http import HttpRequest, HttpResponse, HttpResponseRedirect
from django.views import generic
from django.urls import reverse

from hare.core import models
from hare.ui.forms import NewDestinationForm


logger = logging.getLogger(__name__)


def index(request: HttpRequest) -> HttpResponse:
    """Redirect to ``ListDestinations`` endpoint."""
    return HttpResponseRedirect(reverse("list-destinations"))


class ListDestinations(generic.FormView):
    """List and add destinations with descriptions and aliases.

    The ``GET`` handler lists all available destinations with descriptions and aliases
    using the ``ui/list-destinations.html`` template.

    The ``POST`` handler adds a new destination with aliases from the form rendered by
    the ``ui/list-destinations.html`` template. This view handles the ``POST`` request
    primarily so that Django can do the form validation server-side and pass any errors to
    the client using Django's Form machinery.
    Validation of the form is done in three stages:
        1) ``NewDestinationForm``
        2) ``models.Destination.objects.create_with_aliases``
        3) ``models.Destination``
    See the docstring for ``models.Destination.objects.create_with_aliases`` for more information.
    Note that setting the default fallback destination is not currently supported via this endpoint.
    """

    form_class = NewDestinationForm
    template_name = "ui/list-destinations.html"

    def gen_destinations_with_aliases(  # pylint: disable=no-self-use
        self,
    ) -> typing.Dict[int, typing.Dict[str, typing.Union[str, typing.List[str]]]]:
        """Retrieve list of destinations and aliases from database.

        Aliases are aggregated per-destination and sorted in alphabetical order for display.
        """
        # OrderedDict + ordering by Destination.description ensures the table is sorted by description in the UI
        destinations: typing.Dict[int, typing.Dict[str, typing.Any]] = OrderedDict()
        try:
            for alias in models.Alias.objects.select_related("destination").order_by("destination__description").all():
                if alias.destination.id not in destinations:
                    destinations[alias.destination.id] = {
                        "url": alias.destination.url,
                        "description": alias.destination.description,
                        "aliases": [alias.name],
                    }
                else:
                    destinations[alias.destination.id]["aliases"].append(alias.name)

            for destination in destinations.values():
                destination["aliases"] = sorted(destination["aliases"])
        except DatabaseError as exc:
            logger.warning("Failed to fetch destinations from database ({})", exc)
        return destinations

    def get_context_data(self, **kwargs) -> typing.Dict[typing.Any, typing.Any]:
        context = super().get_context_data(**kwargs)
        context["destinations"] = self.gen_destinations_with_aliases()
        return context

    def form_valid(self, form: NewDestinationForm) -> HttpResponse:
        try:
            _destination = models.Destination.objects.create_with_aliases(
                form.cleaned_data["url"],
                form.cleaned_data["description"],
                form.cleaned_data["aliases"],
                form.cleaned_data["is_fallback"],
                False,
            )
        except (DatabaseError, ValueError) as exc:
            logger.warning(
                "Failed to create new destination {} with aliases {} ({})",
                form.cleaned_data["url"],
                form.cleaned_data["aliases"],
                exc,
            )
        # TODO: Redirect with flash message stating successful/failed addition
        return HttpResponseRedirect(reverse("list-destinations"))
