## Hare
----

### Purpose

Hare is a miminal rewrite of the open source "smart bookmark" project by Facebook called bunny1 (http://www.bunny1.org/).  Hare allows for the creation of parameterized bookmarks so that, instead of having to type in https://google.com/?q=monkeys you can simply type "g monkeys" in your browser and it'll automagically redirect you to the right URL.  The impetus for the rewrite was that Bunny1 requires you to define your destinations and aliases before deploying the application, and having multiple aliases for a single destination is cumbersome.  Hare's solution is to keep destinations, aliases, and arguments in a database, allowing for dynamic updating of destinations, aliases, and arguments.

### Installation

```bash
$ git clone git@github.com:conveen/hare.git
$ virtualenv -p $(which python3) venv
$ source venv/bin/activate
$ pip install -r requirements.txt
```

Once the dependencies are installed, you can bootstrap the Hare database with a base set of data using the following command:

```bash
$ ./bootstrap.py --connect 'sqlite:///./apps/engine/src/hare.db'
```

### Databases Supported

Theoretically, all databases supported by SQLAlchemy should be supported by Hare, though it has only been tested with SQLite.  See below for the list of database systems and information about each as they relate to SQLAlchemy:

| RDBMS Name | SQLAlchemy Link |
|------------|-----------------|
| SQLite | <a href="http://docs.sqlalchemy.org/en/latest/dialects/sqlite.html" target="_blank">http://docs.sqlalchemy.org/en/latest/dialects/sqlite.html</a> |
| PostgreSQL | <a href="http://docs.sqlalchemy.org/en/latest/dialects/postgresql.html" target="_blank">http://docs.sqlalchemy.org/en/latest/dialects/postgresql.html</a> |
| MySQL | <a href="http://docs.sqlalchemy.org/en/latest/dialects/mysql.html" target="_blank">http://docs.sqlalchemy.org/en/latest/dialects/mysql.html</a> |
| MSSQL | <a href="http://docs.sqlalchemy.org/en/latest/dialects/mssql.html" target="_blank">http://docs.sqlalchemy.org/en/latest/dialects/mssql.html</a> |
| Oracle | <a href="http://docs.sqlalchemy.org/en/latest/dialects/oracle.html" target="_blank">http://docs.sqlalchemy.org/en/latest/dialects/oracle.html</a> |

### Getting Started

To run Hare locally for testing, run the following commands:

```bash
$ source venv/bin/activate
$ ./engine.py --port <insert port here - default is 5000>
```

Once Hare is running, open Chrome or Internet Explorer and try browsing to http://localhost:port/?query=wp+hare.

### Installing Hare In the Browser

#### Chrome

1. Open Chrome and go to chrome://settings/searchEngines.
2. Under __Default search engines__ on the right click __ADD__.
3. In the __Search engine__ field enter "Hare"
4. In the __Keyword__ field enter "hr"
5. In the __URL with %s in place of query__ field enter "http://localhost:port?query=%s"
    * If you want to set a fallback/default search engine, enter "http://localhost:port?fallback=ddg&query=%s"

#### Firefox, Edge, IE

Create a bookmark with the following as the URL:

```javascript
javascript:hareurl='http://localhost:port?query=';cmd=prompt('Hare%20Search');if(cmd){window.location=hareurl+escape(cmd);}else{void(0);}
```
