#!/usr/bin/env sh

# If HARE_DOMAIN provided, fill in template and remove it.
if ! [ -z "${HARE_DOMAIN}" ]
then
    if [ -z "${DEFAULT_SEARCH_ENGINE}" ]; then DEFAULT_SEARCH_ENGINE="google"; fi
    envsubst < hare_engine/static/opensearch.xml.template > hare_engine/static/opensearch.xml
fi
rm hare_engine/static/opensearch.xml.template

envsubst < /etc/nginx/sites-available/hare_engine.conf.template > /etc/nginx/sites-available/hare_engine.conf && \
    rm -f /etc/nginx/sites-available/hare_engine.conf.template && \
    ln -s /etc/nginx/sites-available/hare_engine.conf /etc/nginx/sites-enabled/hare_engine.conf && \
    uwsgi --ini /var/www/hare/hare_engine.ini && \
    nginx -g 'daemon off;'
