# -*- coding: UTF8 -*-
# manager.py
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

import os.path
from sqlalchemy import create_engine, MetaData
from sqlalchemy.engine import Engine
from sqlalchemy.orm import sessionmaker, scoped_session, Session
from flask import Flask, current_app

class DBManager(object):
    '''
    Class for managing state of database connection
    '''
    def __init__(self, conn_string=None, metadata=None, scoped=False):
        self.conn_string = conn_string
        self.metadata = metadata
        self.scoped_sessions = scoped
        self.engine = None
        self.session_factory = None
        self.session = None
    def __flask_teardown_appcontext(self, err):
        '''
        '''
        if current_app.config.get('HARE_DATABASE_COMMIT_ON_TEARDOWN'):
            if err is None:
                self.commit()
        self.session.remove()
        return err
    @property
    def conn_string(self):
        '''
        @conn_string.getter
        '''
        return self.__conn_string
    @conn_string.setter
    def conn_string(self, value):
        '''
        @conn_string.setter
        Preconditions:
            conn_string is of type String
        '''
        assert value is None or isinstance(value, str)
        self.__conn_string = value
    @property
    def engine(self):
        '''
        @engine.getter
        '''
        return self.__engine
    @engine.setter
    def engine(self, value):
        '''
        @engine.setter
        Preconditions:
            value is of type Engine
        '''
        assert value is None or isinstance(value, Engine)
        self.__engine = value
    def create_engine(self, conn_string=None, persist=True):
        '''
        Args:
            conn_string: String     => database connection string
            persist: True           => whether to persist the database engine
        Returns:
            Engine
            New database engine using either provided conn_string
            or self.conn_string
        Preconditions:
            conn_string is of type String
            persist is of type Boolean
        '''
        assert isinstance(persist, bool)
        if conn_string is not None:
            self.conn_string = conn_string
        if self.conn_string is not None:
            engine = create_engine(self.conn_string)
            if persist:
                self.engine = engine
            return engine
    @property
    def metadata(self):
        '''
        @metadata.getter
        '''
        return self.__metadata
    @metadata.setter
    def metadata(self, value):
        '''
        @metadata.setter
        Preconditions:
            value is of type MetaData
        '''
        assert value is None or isinstance(value, MetaData)
        self.__metadata = value
    @property
    def session_factory(self):
        '''
        @session_factory.getter
        '''
        return self.__session_factory
    @session_factory.setter
    def session_factory(self, value):
        '''
        @session_factory.setter
        Preconditions:
            value is of type Callable
        '''
        assert value is None or callable(value)
        self.__session_factory = value
    @property
    def session(self):
        '''
        @_session.getter
        '''
        if self.scoped_sessions:
            return self.session_factory
        return self.__session
    @session.setter
    def session(self, value):
        '''
        @_session.setter
        '''
        assert value is None or isinstance(value, Session)
        self.__session = value
    @property
    def scoped_sessions(self):
        '''
        Getter for scoped_sessions
        '''
        return self.__scoped_sessions
    @scoped_sessions.setter
    def scoped_sessions(self, value):
        '''
        Setter for scoped_sessions
        '''
        assert isinstance(value, bool)
        self.__scoped_sessions = value
    def create_session(self, persist=True):
        '''
        Args:
            persist: Boolean    => whether to persist the session
        Returns:
            Session
            Either new session object or pre-existing session
            NOTE:
                If _session_factory is None, this will throw an error
        Preconditions:
            persist is of type Boolean
        '''
        assert isinstance(persist, bool), 'Persist is not of type Boolean'
        if self.scoped_sessions:
            return self.session_factory
        if persist:
            if self.session is None:
                self.session = self.session_factory()
            return self.session
        return self.session_factory()
    def close_session(self, session=None):
        '''
        Args:
            session: Session    => session to close if not self.session
        Procedure:
            Closes either the provided session or the current session
            at self.session
        Preconditions:
            session is of type Session
        '''
        assert session is None or isinstance(session, Session)
        if session is not None:
            session.close()
        elif self.scoped_sessions and self.session_factory is not None:
            self.session_factory.remove()
        elif self.session is not None:
            self.session.close()
            self.session = None
    def bootstrap(self, engine=None):
        '''
        Args:
            engine: Engine  => the connection engine to use
        Procedure:
            Use a connection engine to bootstrap a database
            with the necessary tables, indexes, and (materialized) view
        Preconditions:
            engine is of type Engine
        '''
        if engine is not None:
            self.engine = engine
        if self.engine is not None and self.metadata is not None:
            self.metadata.create_all(self.engine)
    def initialize(self, app, bootstrap=False):
        '''
        Args:
            app: Flask              => flask app using this DB manager
            bootstrap: Boolean      => whether to bootstrap database with table, index, and view information
        Procedure:
            Initialize a database connection using database connection string 
            from Flask app config and perform setup tasks if necessary
        Preconditions:
            app is of type Flask
            bootstrap is of type Boolean
        '''
        assert app is None or isinstance(app, Flask)
        assert isinstance(bootstrap, bool)
        if app is not None:
            app.teardown_appcontext_funcs.append(self.__flask_teardown_appcontext)
            app.dbm = self
            self.conn_string = app.config.get('HARE_DATABASE_URI')
            self.scoped_sessions = True
        try:
            self.create_engine()
            if self.engine is not None:
                if bootstrap:
                    self.bootstrap()
                self.session_factory = sessionmaker(bind=self.engine, autoflush=False)
                if self.scoped_sessions:
                    self.session_factory = scoped_session(self.session_factory)
        except Exception as e:
            raise Exception('Failed to initialize DBManager (%s)'%str(e))
    def query(self, model, **kwargs):
        '''
        Args:
            model: BaseTable    => model of table to query
            **kwargs: Any       => field to filter on
        Returns:
            Query
            Query object if no error thrown, None otherwise
        Preconditions:
            model is base class of BaseTable (assumed True)
        '''
        try:
            query = self.session.query(model)
            for arg in kwargs:
                query = query.filter(getattr(model, arg) == kwargs[arg])
            return query
        except:
            return None
    def add(self, record, session=None, commit=False):
        '''
        Args:
            record: BaseTable   => record to add to current session
            session: Session    => session to add record to
            commit: Boolean     => whether to commit and end the transaction block
        Procedure:
            DBManager
            Add record to either provided or current session and potentially commit
        Preconditions:
            record is instance of BaseTable
            session is of type Session
            commit is of type Boolean
        '''
        if session is None:
            session = self.session
        session.add(record)
        if commit:
            self.commit(session)
        return self
    def delete(self, record, session=None, commit=False):
        '''
        Args:
            record: BaseTable   => record to add to current session
            session: Session    => session to add record to
            commit: Boolean     => whether to commit and end the transaction block
        Procedure:
            DBManager
            Delete record using either provided session or current session 
            and potentially commit
        Preconditions:
            record is instance of BaseTable
            session is of type Session
            commit is of type Boolean
        '''
        if session is None:
            session = self.session
        session.delete(record)
        if commit:
            self.commit(session)
        return self
    def commit(self, session=None):
        '''
        Args:
            session: Session    => session to add record to
        Procedure:
            Commit either provided or current session
        Preconditions:
            session is of type Session
        '''
        if session is None:
            session = self.session
        if session is not None:
            session.commit()
        return self
    def rollback(self, session=None):
        '''
        Args:
            session: Session    => session to add record to
        Procedure:
            Rollback either provided or current session
        Preconditions:
            session is of type Session
        '''
        if session is None:
            session = self.session
        if session is not None:
            session.rollback()
        return self
