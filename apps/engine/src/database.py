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

from sqlalchemy.orm import relationship
from sqlalchemy import Column, ForeignKey
from sqlalchemy.schema import UniqueConstraint, CheckConstraint, DDL
from sqlalchemy.types import BigInteger, Integer, Boolean, String, Text, TIMESTAMP
from sqlalchemy.event import listen
from sqlalchemy.ext.declarative import declarative_base, declared_attr

from apps.common.database import BaseTableTemplate, TimestampDefaultExpression

BaseTable = declarative_base(cls=BaseTableTemplate)

class DestinationMixin(object):
    @declared_attr
    def dest_id(cls):
        return Column(\
            BigInteger().with_variant(Integer, 'sqlite'), 
            ForeignKey('destination.id', ondelete='CASCADE', onupdate='CASCADE'), 
            nullable=False, 
            index=True\
        )

class Destination(BaseTable):
    url         = Column(String().with_variant(Text, 'postgresql'), nullable=False, unique=True, index=True)
    is_fallback = Column(Boolean, nullable=False, index=True)
    priority    = Column(Integer, index=True)
    aliases     = relationship('Alias', backref='destination')
    args        = relationship('Argument', backref='destination')

    def __repr__(self):
        return 'Destination(id=%s, url=%s, is_fallback=%s, priority=%s)'%(\
            self._notnone(self.id),
            self._notnone(self.url),
            self._notnone(self.is_fallback),
            self._notnone(self.priority)\
        )

class Alias(DestinationMixin, BaseTable):
    name        = Column(String().with_variant(Text, 'postgresql'), nullable=False, index=True)
    __table_args__ = (
        UniqueConstraint('dest_id', 'name'),
    )

    def __repr__(self):
        return 'Alias(id=%s, dest_id=%s, name=%s)'%(\
            self._notnone(self.id),
            self._notnone(self.dest_id),
            self._notnone(self.name)\
        )

class Argument(DestinationMixin, BaseTable):
    name        = Column(String().with_variant(Text, 'postgresql'), nullable=False, index=True)
    type        = Column(String().with_variant(Text, 'postgresql'), nullable=False, index=True)
    default     = Column(String().with_variant(Text, 'postgresql'))
    __table_args__ = (\
        UniqueConstraint('dest_id', 'name'),
        CheckConstraint("type IN ('string', 'nargs')")
    )

    def __repr__(self):
        return 'Argument(id=%s, dest_id=%s, name=%s, type=%s, default=%s)'%(\
            self._notnone(self.id),
            self._notnone(self.dest_id),
            self._notnone(self.name),
            self._notnone(self.type),
            self._notnone(self.default)\
        )
