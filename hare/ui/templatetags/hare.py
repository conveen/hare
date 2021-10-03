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

from django import forms, template
from django.conf import settings
from django.contrib import messages
from django.contrib.messages.storage.base import Message
from django.forms.boundfield import BoundField
from django.utils.html import format_html
from django.utils.safestring import mark_safe


logger = logging.getLogger(__name__)
register = template.Library()


def notification(message: Message, delay: int = 2000) -> str:
    """Render toast for single message.

    Does not escape the message text and thus *must not* directly be used as a tag.
    """
    if message.level == messages.constants.WARNING:
        icon = "exclamation-circle-fill.svg"
    elif message.level == messages.constants.ERROR:
        icon = "x-circle-fill.svg"
    else:
        icon = "check-circle-fill.svg"

    return f"""
    <div class="toast" style="min-width: 350px;" role="alert" aria-live="assertive" aria-atomic="true" data-delay="{delay}">
        <div class="toast-header">
            <img src="{settings.STATIC_URL}ui/svg/{icon}" title="Notification level">
            <small class="pl-1">just now</small>
            <button type="button" class="ml-auto mb-1 close" data-dismiss="toast" aria-label="Close">
                <span aria-hidden="true">&times;</span>
            </button>
        </div>
        <div class="toast-body">{message}</div>
    </div>
    """


@register.simple_tag(takes_context=True)
def notifications(context: template.RequestContext) -> str:
    """Render stack of toasts for messages."""
    storage = messages.get_messages(context.request)
    if not storage:
        return ""

    toasts = mark_safe("\n".join(notification(message) for message in storage))
    return format_html(
        """
    <div id="notifications-wrapper" class="position-fixed pt-5 pr-3" style="z-index: 5; right: 0; top: 0;">
    {toasts}
    </div>
    """,
        toasts=toasts,
    )


@register.simple_tag
def form_input_text(form: forms.Form, field: BoundField, input_type: str = "text") -> str:
    """Form group with label, input, and errors when form is bound."""
    form_is_bound = form.is_bound
    errors = field.errors
    has_errors = form_is_bound and errors

    return format_html(
        """
    <div class="form-group">
        <label for="{input_id}">{label}</label>
        <input
            type="{input_type}"
            class="form-control {invalid_class}"
            id="{input_id}"
            name="{name}"
            value="{value}"
            aria-describedby="{input_id}-help {input_id}-feedback"
            required>
        <small id="{input_id}-help" class="form-text text-muted">{help_text}</small>
        <div id="{input_id}-feedback" class="invalid-feedback">{errors}</div>
    </div>
    """,
        input_id=field.id_for_label,
        label=field.label,
        input_type=input_type,
        invalid_class="is-invalid" if has_errors else "",
        has_errors=has_errors,
        name=field.html_name,
        value=field.value() or "",
        help_text=field.help_text,
        errors=" ".join(field.errors),
    )


@register.simple_tag
def form_input_checkbox(
    form: forms.Form,
    field: BoundField,
    tooltip_src: str = "ui/svg/info-circle-fill.svg",
    placement: str = "right",
) -> str:
    """Form group with label, checkbox input, and tooltip."""
    form_is_bound = form.is_bound
    errors = field.errors
    has_errors = form_is_bound and errors

    return format_html(
        """
    <div class="form-group">
        <div class="form-check">
            <input
            type="checkbox"
            class="form-check-input {invalid_class}"
            id="{input_id}" {checked}/>
            <label class="form-check-label" for="{input_id}">{label}</label>
            <img
                id="{input_id}-tooltip"
                src="{static_url}{tooltip_src}"
                data-toggle="tooltip"
                data-placement="{placement}"
                title="{help_text}" />
        </div>
    </div>
    """,
        input_id=field.id_for_label,
        label=field.label,
        invalid_class="is-invalid" if has_errors else "",
        checked="checked" if form_is_bound and field.value() else "",
        static_url=settings.STATIC_URL,
        tooltip_src=tooltip_src,
        placement=placement,
        help_text=field.help_text,
    )
