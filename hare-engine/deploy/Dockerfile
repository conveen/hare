# Conveen
# 2020-11-07

FROM ubuntu:focal

ENV DEBIAN_FRONTEND=noninteractive

WORKDIR /var/www/hare

RUN apt-get update && apt-get upgrade -y && \
    apt-get install -y apt-utils build-essential gettext-base nginx python3 python3-pip python3-venv && \
    pip3 install --no-cache-dir uwsgi && \
    rm -f /etc/nginx/sites-enabled/default /etc/nginx/sites-available/default

COPY hare-engine/deploy/nginx/hare_engine.conf.template /etc/nginx/sites-available/
COPY hare-engine/deploy/wsgi/hare_engine.ini .
COPY hare-engine/deploy/entrypoint.sh .
COPY hare-engine/setup.py .
COPY hare-engine/hare_engine hare_engine
COPY hare-engine/deploy/opensearch/opensearch.xml.template hare_engine/static/

RUN python3 -m venv venv && \
    . venv/bin/activate && \
    pip install --no-cache-dir -U pip wheel && \
    pip install --no-cache-dir -e . && \
    deactivate

RUN chmod +x entrypoint.sh && \
    mkdir /var/log/hare && \
    chown -R www-data:www-data /var/www/hare /var/log/hare

ENV PORT=80

ENTRYPOINT ["./entrypoint.sh"]
