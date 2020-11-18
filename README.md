# Hare: Smart Shortcut Engine

## Overview

Hare is a miminal rewrite of the open source "smart bookmark" project by Facebook called bunny1 (http://www.bunny1.org/).
Hare allows for the creation of parameterized bookmarks so that, instead of having to type in https://google.com/?q=monkeys,
you can simply type "g monkeys" in your browser and it'll automagically redirect you to the right URL.
The impetus for the rewrite was that Bunny1 requires you to define your destinations and aliases before deploying the application,
and having multiple aliases for a single destination is cumbersome. Hare's solution is to keep destinations, aliases, and arguments in a database,
allowing for dynamic additions of destinations and aliases.

## Installation

### Git

Clone the repo from GitHub and run the setup in a virtual environment:

```bash
$ git clone git@github.com:conveen/hare.git
$ cd hare-engine
$ python3 -m venv venv
$ . venv/bin/activate
$ pip install .
```

## Dependencies

All dependencies are specified in the [setup.py](hare-engine/setup.py) file.

## Getting Started

### Docker

Docker images are available on [Docker Hub](https://hub.docker.com/r/conveen/hare-engine).
Use the `:latest` tag for the most up-to-date image.

```bash
$ docker pull conveen/hare-engine:latest
$ docker run --rm --name hare-engine -e HARE_SECRET_KEY='secret key here' -p 8080:80 conveen/hare-engine:latest
```

#### Environment Variables

The hare-engine image (entrypoint) has a number of environment variables to help configure the server and database connections.

`HARE_SECRET_KEY`

**Required.** Sets the Flask secret key, which is required for creating sessions. In production, this should be set via something like a secrets manager.
See [Flask documentation](https://flask.palletsprojects.com/en/1.1.x/quickstart/#sessions) for more information. 

`HARE_DATABASE_URL`

**Required.** Must be [RFC-1738](http://rfc.net/rfc1738.html)-compliant URL pointing to the database.
See [SQLAlchemy documentation](https://docs.sqlalchemy.org/en/13/core/engines.html#database-urls) for more information.

`HARE_DOMAIN`

Domain at which the hare-engine is hosted, with the format `http[s]://<domain.tld>[:<port>]`.  This is used for the [OpenSearch config file](#firefox-via-opensearch), which will automatically be populated if this variable is provided. If not, the OpenSearch file is removed from the container.

`DEFAULT_SEARCH_ENGINE`

Alias of the default search engine to use in the OpenSearch config file.  Must correspond to existing alias in the database, such as `google` or `duckduckgo`.

`HARE_DATABASE_DRIVER`

Name of the Python package to install for the database driver. See the section for your database in the [SQLAlchemy documentation](https://docs.sqlalchemy.org/en/13/core/engines.html#database-urls) for more information.

`HARE_DATABASE_BOOTSTRAP_ON_STARTUP`

If provided, the Hare engine will attempt to create all the necessary tables and indices in the database when it starts.  If the tables and indices already exist, it will not erase data or re-create them, so this option should probably be enabled most of the time.

`PORT`

Port that the container will listen on, defaults to `80`.  In most cases this option shouldn't be changed, unless you're using Docker `host` network mode (or `awsvpc` in AWS) and you need to explicitly set the port.  Instead, prefer publishing the port you need when running the container (`-p $PUBLISH_PORT:80`).

### Docker Compose

A [docker compose](hare-engine/deploy/docker-compose.yml) file is provided that uses [Postgres](https://hub.docker.com/_/postgres) for the database and [PGWeb](https://sosedoff.github.io/pgweb/) for running SQL queries on the database.

#### Environment Variables

The compose file handles the required variables for the Hare engine Docker image, though in production environments you should override the default `HARE_SECRET_KEY` value.  There are a few additional variables to configure the compose stack:

`POSTGRES_PASSWORD`

**Required.** Postgres database password. See the [Postgres image documentation](https://hub.docker.com/_/postgres) for more information.

`NETWORK`

Docker networking mode, default is `bridge`.

`PGWEB_PORT`

Published port for the PGWeb container, default is `8081`.

`TAG`

Hare engine Docker image tag to use, default is `latest`.

`HARE_PORT`

Published port for the Hare engine container, default is `8080`.

#### Creating the database

To run the stack, you must create the Hare database before running PGWeb and Hare.  Run the Postgres container and use `createdb` to create the database, then run the whole stack.  You only have to do this once per stack.

```bash
$ cd hare-engine/deploy
$ POSTGRES_PASSWORD='<pg_password>' docker-compose run -d postgres-01 # prints the container name
$ docker exec -it <pg_container_name> /bin/bash # enter the container
$ createdb -U postgres hare 'Hare datacbase' # runs inside the container
$ exit # exit the container
$ docker kill <pg_container_name>
$ docker up # run the stack
```
After creating the database and running the stack, you can (optionally) [bootstrap the database](#bootstrapping-the-database) with a number of common shortcuts.  Since Docker Compose injects the database URL, you can run the script with the `HARE_DATABASE_URL` env variable.

### Local Development

To run a local development (non-Docker) version of Hare, use the Flask command line:

```bash
$ cd hare-engine
$ . venv/bin/activate
$ FLASK_APP=hare_engine.app:gen_app FLASK_ENV=development flask run -p 8080
```

### Bootstrapping the Database

Hare comes with a [script](hare-engine/hare_engine/scripts/bootstrap.py) to bootstrap the database with a number of common websites,
such as DuckDuckGo, Google, Bing, Wikipedia, Reddit, Twitter, Facebook, Youtube, etc.
To run the script, simply point it at the database using an [RFC-1738](http://rfc.net/rfc1738.html)-compliant URL:

```bash
$ ./hare-engine/hare_engine/scripts/bootstrap.py <database_url>
```

## Installing Hare in the Browser

### Chrome

1. Open Chrome and navigate to chrome://settings/searchEngines
2. To the right of `Other search engines` click the `Add` button
3. Enter the following information:
    * `Search engine`: Hare
    * `Keyword`: hare
    * `URL with %s in place of query`: http(s)://<hare_domain_name>:<port>?query=%s&fallback=g
        * Note that the fallback parameter could be any search engine, like DuckDuckGo, Wikipedia, Bing, etc.

### Firefox (via OpenSearch)

Firefox supports the [OpenSearch description format](https://developer.mozilla.org/en-US/docs/Web/OpenSearch) for specifying
search engines.  The Hare OpenSearch template is in the [deploy/opensearch](hare-engine/deploy/opensearch) directory,
and is automatically included in the Docker container if the `HARE_DOMAIN` env variable is provided.

## Contributing/Suggestions

Contributions and suggestions are welcome! To make a feature request, report a bug, or otherwise comment on existing
functionality, please file an issue. For contributions please submit a PR, but make sure to lint, type-check, and test
your code before doing so. Thanks in advance!
