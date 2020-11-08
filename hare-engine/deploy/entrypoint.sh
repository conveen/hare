#!/usr/bin/env sh

envsubst < /etc/nginx/sites-available/hare_engine.conf.template > /etc/nginx/sites-available/hare_engine.conf && \
    rm -f /etc/nginx/sites-available/hare_engine.conf.template && \
    ln -s /etc/nginx/sites-available/hare_engine.conf /etc/nginx/sites-enabled/hare_engine.conf && \
    uwsgi --ini /var/www/hare/hare_engine.ini && \
    nginx -g 'daemon off;'
