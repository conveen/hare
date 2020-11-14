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

### Local Development

To run a local development version of Hare, use the Flask command line:

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
$ cd hare_engine
$ . venv/bin/activate
$ ./hare_engine/scripts/bootstrap.py <database_url>
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
