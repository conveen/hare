# -*- coding: UTF8 -*-
# database.py
#Copyright (c) 2018 conveen
#
#Permission is hereby granted, free of charge, to any person obtaining a copy
#of this software and associated documentation files (the "Software"), to deal
#in the Software without restriction, including without limitation the rights
#to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
#copies of the Software, and to permit persons to whom the Software is
#furnished to do so, subject to the following conditions:
#
#The above copyright notice and this permission notice shall be included in all
#copies or substantial portions of the Software.
#
#THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
#IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
#FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
#AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
#LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
#OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
#SOFTWARE.

from sqlalchemy import Column
from sqlalchemy.types import BigInteger, Integer, TIMESTAMP
from sqlalchemy.schema import Table, Column, MetaData, DDLElement
from sqlalchemy.ext.declarative import declared_attr
from sqlalchemy.ext.compiler import compiles
from sqlalchemy.sql.expression import ClauseElement, FromClause
from sqlalchemy.event import listen

class TimestampDefaultExpression(ClauseElement):
    ''''
    Class to generate server default timestamp expressions based
    on SQL dialect.
    '''
    pass

@compiles(TimestampDefaultExpression, 'mssql')
def generate_timestamp_expression(element, compiler, **kwargs):
    return 'GETUTCDATE()'
@compiles(TimestampDefaultExpression, 'mysql')
def generate_timestamp_expression(element, compiler, **kwargs):
    return 'UTC_TIMESTAMP()'
@compiles(TimestampDefaultExpression, 'oracle')
def generate_timestamp_expression(element, compiler, **kwargs):
    return 'SYS_EXTRACT_UTC(SYSTIMESTAMP)'
@compiles(TimestampDefaultExpression, 'postgresql')
def generate_timestamp_expression(element, compiler, **kwargs):
    return '(NOW() AT TIME ZONE \'UTC\')'
@compiles(TimestampDefaultExpression, 'sqlite')
def generate_timestamp_expression(element, compiler, **kwargs):
    return 'CURRENT_TIMESTAMP'

class CreateViewExpression(DDLElement):
    '''
    Class to allow easy creation of views 
    (implementation taken from 
    http://www.jeffwidman.com/blog/847/using-sqlalchemy-to-create-and-manage-postgresql-materialized-views/)
    '''
    def __init__(self, name, selectable):
        self.name = name
        self.selectable = selectable

@compiles(CreateViewExpression)
def generate_mview_create_expression(element, compiler, **kwargs):
    return 'CREATE OR REPLACE VIEW %s AS %s'%(\
        element.name,\
        compiler.sql_compiler.process(element.selectable, literal_binds=True))

class CreateMaterializedViewExpression(CreateViewExpression):
    '''
    Class to allow easy creation of materialized views 
    in PostgreSQL (implementation taken from 
    http://www.jeffwidman.com/blog/847/using-sqlalchemy-to-create-and-manage-postgresql-materialized-views/)
    '''
    pass

@compiles(CreateMaterializedViewExpression)
def generate_mview_create_expression(element, compiler, **kwargs):
    return 'CREATE OR REPLACE VIEW %s AS %s'%(\
        element.name,\
        sql_compiler.process(element.selectable, literal_binds=True))
@compiles(CreateMaterializedViewExpression, 'postgresql')
def generate_mview_create_expression(element, compiler, **kwargs):
    return 'CREATE OR REPLACE MATERIALIZED VIEW %s AS %s'%(\
        element.name,\
        sql_compiler.process(element.selectable, literal_binds=True))

class DropViewExpression(DDLElement):
    '''
    Class to allow easy deletion of views
    '''
    def __init__(self, name):
        self.name = name

@compiles(DropViewExpression)
def generate_mview_drop_expression(element, compiler, **kwargs):
    return 'DROP VIEW IF EXISTS %s'%(element.name)

class DropMaterializedViewExpression(DropViewExpression):
    '''
    Class to allow easy deletion of materialized views in PostgreSQL
    '''
    pass

@compiles(DropMaterializedViewExpression)
def generate_mview_drop_expression(element, compiler, **kwargs):
    return 'DROP VIEW IF EXISTS %s'%(element.name)
@compiles(DropMaterializedViewExpression, 'postgresql')
def generate_mview_drop_expression(element, compiler, **kwargs):
    return 'DROP MATERIALIZED VIEW IF EXISTS %s'%(element.name)

def create_view(name, selectable, metadata, materialized=False):
    '''
    Args:
        name: String            => name of materialized view to create
        selectable: FromClause  => query to create view as
        metadata: MetaData      => metadata to listen for events on
        materialized: Boolean   => whether to create standard or materialized view
    Returns:
        Table object bound to temporary MetaData object with columns as
        columns returned from selectable (essentially creates table as view)
        NOTE:
            For non-postgresql backends, creating a materialized view
            will result in a standard view, which cannot be indexed
    Preconditions:
        name is of type String
        selectable is of type FromClause
        metadata is of type Metadata
        materialized is of type Boolean
    '''
    assert isinstance(name, str), 'Name is not of type String'
    assert isinstance(selectable, FromClause), 'Selectable is not of type FromClause'
    assert isinstance(metadata, MetaData), 'Metadata is not of type MetaData'
    assert isinstance(materialized, bool), 'Materialized is not of type Boolean'
    _tmp_mt = MetaData()
    tbl = Table(name, _tmp_mt)
    for c in selectable.c:
        tbl.append_column(Column(c.name, c.type, primary_key=c.primary_key))
    listen(\
        metadata,\
        'after_create',\
        CreateMaterializedViewExpression(name, selectable) if materialized else CreateViewExpression(name, selectable))
    listen(\
        metadata,\
        'before_drop',\
        DropMaterializedViewExpression(name) if materialized else DropViewExpression(name))
    return tbl

class BaseTableTemplate(object):
    '''
    Template for tables in database
    '''
    @declared_attr
    def __tablename__(cls):
        return str(cls.__name__.lower())
    @staticmethod
    def _notnone(value):
        return str(value) if value is not None else 'None'

    id          = Column(BigInteger().with_variant(Integer, 'sqlite'), nullable=False, primary_key=True)
    created_at  = Column(TIMESTAMP(timezone=True), server_default=TimestampDefaultExpression())
    modified_at = Column(TIMESTAMP(timezone=True), server_default=TimestampDefaultExpression())

    def populate_fields(self, data_dict, overwrite=True):
        '''
        Args:
            data_dict: Dict<String, Any>   => dict containing data to map to fields
            overwrite: Boolean              => whether to overwrite values of current instance
        Procedure:
            Populate instance fields in this class with values from data_dict
            where each key in data_dict maps to an instance field.
            For example, to populate id and created_at, data_dict would look
            like:
                {
                    'id': <Integer>,
                    'created_at': datetime.datetime
                }
        Preconditions:
            data_dict is of type Dict<String, Any>
        '''
        assert isinstance(data_dict, dict) and all((
            isinstance(key, str) for key in data_dict)), 'Data_dict is not of type Dict<String, Any>'
        for key in data_dict:
            if hasattr(self, key) and (getattr(self, key) is None or overwrite):
                setattr(self, key, data_dict[key])
        return self

