## -*- coding: UTF8 -*-
## setup.py
##
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


import os
import setuptools

if os.path.isfile("README.md"):
    with open("README.md", "r") as readme:
        long_description = readme.read()
else:
    long_description = ""


setuptools.setup(
    name="hare_engine",
    version="PACKAGE_VERSION",
    author="conveen",
    author_email="conveen@protonmail.com",
    description="Hare Smart Bookmarks: search engine",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/conveen/hare",
    project_urls={
        "Issue Tracker": "https://github.com/conveen/hare/issues",
        "Releases": "https://github.com/conveen/hare/releases"
    },
    packages="hare_engine",
    install_requires=[
        "Flask>=1.1",
        "lc-flask-routes[all]",
        "lc-sqlalchemy-dbutils",
    ],
    classifiers=[
        "Intended Audience :: Developers",
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.6",
)