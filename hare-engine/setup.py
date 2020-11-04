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

from hare_engine import __version__

if os.path.isfile("README.md"):
    with open("README.md", "r") as readme:
        long_description = readme.read()
else:
    long_description = ""


setuptools.setup(
    name="hare_engine",
    version=__version__,
    author="conveen",
    author_email="conveen@protonmail.com",
    description="Hare: Smart Shortcut Engine",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/conveen/hare",
    project_urls={
        "Issue Tracker": "https://github.com/conveen/hare/issues",
        "Releases": "https://github.com/conveen/hare/releases"
    },
    packages=setuptools.find_packages(include=["hare_engine", "hare_engine.*"]),
    package_data={
        "hare_engine": [
            "static/*.ico",
            "templates/*.html",
            "static/js/*.js",
            "static/css/*.css"
        ],
    },
    install_requires=[
        "Flask>=1.1",
        "lc-flask-routes[all]",
        "lc-sqlalchemy-dbutils",
    ],
    entry_points={"console_scripts": [
        "hare-bootstrap=hare_engine.scripts.bootstrap:main",
        "hare-newdest=hare_engine.scripts.newdest:main",
    ]},
    classifiers=[
        "Framework :: Flask",
        "Intended Audience :: Developers",
        "Programming Language :: Python :: 3",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
    ],
    python_requires=">=3.6",
)
