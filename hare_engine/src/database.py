## -*- coding: UTF8 -*-
## database.py
## Copyright (c) 2020 conveen
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

from typing import List, Optional, Tuple
from urllib.parse import quote_plus

from sqlalchemy.exc import InvalidRequestError
from sqlalchemy.ext.declarative import declarative_base, declared_attr
from sqlalchemy.orm import relationship
from sqlalchemy.schema import CheckConstraint, Column, ForeignKey, UniqueConstraint
from sqlalchemy.types import Boolean, Integer, UnicodeText, TIMESTAMP
from lc_sqlalchemy_dbutils.manager import DBManager
from lc_sqlalchemy_dbutils.schema import TimestampDefaultExpression


__author__ = "conveen"


class BaseTableMixin:
    """Mixin class containing base columns that should be present
    on every table:
        1) id: primary key
        2) created_at: timestamp of record creation in database
    Also sets table name as lowercase of each class name.
    """
    __slots__ = ()

    @declared_attr
    def __tablename__(cls):
        return cls.__name__.lower()

    id = Column(Integer, primary_key=True, nullable=False)
    created_at = Column(TIMESTAMP(timezone=True), server_default=TimestampDefaultExpression())

    def __repr__(self):
        # Get class name (not table name)
        cls_name = type(self).__name__
        # Get columns for table
        columns = [(column.name, getattr(self, column.name)) for column in self.__table__.columns]
        # "<class_name>(<column_1>=<value1>, <column_2>=<value2>, ..., <column_N>=<valueN>)"
        return "{}({})".format(cls_name,
                               ", ".join("{}={}".format(column,value) for column, value in columns))


BaseTable = declarative_base(cls=BaseTableMixin)


class Destination(BaseTable):
    """destination table

    A destination represents a unique _URL_ and set of zero or more parameters.
    For example, a destination could be https://www.youtube.com/results?search_query={},
    where '{}' is a single parameter. Destinations could also have no
    parameters, like https://time.is/, or have parameters as part of the URL (not a URL parameter).

    Fallback destinations are URLs with a single parameter, and are used if the specified
    destination does not exist, or there is otherwise an error in redirecting an a destination.
    Common choices could be DuckDuckGo, Wikipedia, or Google.
    """
    url = Column(UnicodeText, nullable=False, index=True, unique=True)
    num_args = Column(Integer, nullable=False)
    is_fallback = Column(Boolean, nullable=False, index=True)
    is_default_fallback = Column(Boolean, nullable=False, server_default=text("false"))
    aliases = relationship("Alias", backref="destination")


class DestinationForeignKeyMixin:
    """Mixin class for tables that have a Foreign Key relationship
    with the destination table. The ondelete and onupdate behavior
    is set to CASCADE so that any changes made through the SQLAlchemy ORM
    to the destination table will be propogated to any table with a relation to it.
    Note that ondelete and onupdate are SQLAlchemy constructs, and do not change
    database (non-ORM) behavior.
    """
    __slots__ = ()

    @declared_attr
    def dest_id(cls):
        return Column(Integer,
                      ForeignKey("destination.id", onupdate="CASCADE", ondelete="CASCADE"),
                      nullable=False,
                      index=True)


class Alias(DestinationForeignKeyMixin, BaseTable):
    """alias table

    Aliases act as references, or pointers, to destinations, and are the shortcuts users type to 
    get redirected to a destination. For example, the shortcuts "ddg", "duckduckgo", and "go" could
    all reference a search on DuckDuckGo.

    There is a one-to-many relationship between destination and alias, and a one-to-one relationship
    between alias and destination. In other words, a destination can have many aliases, but an alias
    must uniquely point to a single destination (enforced by a unique constraint on dest_id and name).
    """
    __table_args__ = (
        UniqueConstraint("dest_id", "name"),
    )

    name = Column(UnicodeText, nullable=False, index=True)


def add_destination_with_aliases(dbm: DBManager,
                                 url: str,
                                 num_args: int,
                                 aliases: List[str],
                                 is_fallback: bool = False,
                                 is_default_fallback: bool = False):
    """
    Args:
        See destination and alias table schemata.
    Description:
        Validate arguments and add destination to database with provided alias(es).
        Some validation is also done on the database. Any exceptions raised by the underlying
        database driver, or any validation errors (ValueError), must be handled by the caller.
    Preconditions:
        Database manager has existing session with database
    Raises:
        ValueError: if arguments don't pass validation
        Consult database driver and DBManager class for other possible errors
    """
    # Ensure num_args >= 0
    if num_args < 0:
        raise ValueError("num_args must be non-negative")
    # Ensure number of aliases >= 1
    # NOTE: Checked here due to lack of runtime type-checking
    if not aliases:
        raise ValueError("must provide at least one alias")
    # Ensure num_args is 1 if is fallback destination
    if is_fallback:
        if num_args > 1:
            raise ValueError("fallback destinations must have one argument")
    # Ensure if destination is default fallback, is also fallback
    elif is_default_fallback and not is_fallback:
        is_fallback = True

    # Flush pending transaction if within one to ensure
    # can rollback destination addition transaction if fails
    try:
        dbm.commit()
    except InvalidRequestError:
        pass

    # If is_default_fallback, determine if existing default fallback(s)
    # Cannot have two default fallbacks, so must atomically add new
    # destination as default and remove flag from existing.
    if is_default_fallback:
        existing_defaults = dbm.query(db.Destination, is_default_fallback=True).all()
        if existing_defaults:
            for destination in existing_defaults:
                destination.is_default_fallback = False

    # Add destination and alias records to session
    destination = db.Destination(url=url,
                                 num_args=num_args,
                                 is_fallback=is_fallback,
                                 is_default_fallback=is_default_fallback))
    db.add(destination)
    for alias in aliases:
        dbm.add(db.Alias(name=quote_plus(alias), destination=destination))

    # Try to commit transaction. If fails, rollback transaction and pass through exception
    try:
        dbm.commit()
    except:
        dbm.rollback()
        raise
