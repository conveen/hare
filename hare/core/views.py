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

from django.db import DatabaseError, IntegrityError
from django.http import HttpResponse, HttpRequest, JsonResponse

from hare.core import models


logger = logging.getLogger(__name__)


def health_check(request: HttpRequest) -> HttpResponse:
    """Check application health.

    Checks the database connection by creating (and saving) a test record
    in the health check table and then subsequently deleting it.
    Django only opens connections to databases when it needs to, so creating a
    small test record forces Django to open a connection.

    Implementation inspired by django-health-check:
    https://github.com/KristianOellegaard/django-health-check.
    """
    try:
        record = models.HealthCheck.objects.create(check_field=True)
        record.check_field = False
        record.save()
        record.delete()

        return JsonResponse({"status": "ok"})
    except DatabaseError as exc:
        logger.error("Health check failed ({})", exc)

        return JsonResponse({"status": "error"}, status=503)
