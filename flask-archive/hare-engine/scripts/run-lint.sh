#!/usr/bin/env sh
## run-lint.sh
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


if [ $# -lt 1 ]
then
    echo "::: ERROR: Must supply at least one file to run tests on"
    exit 1
fi

if [ ! -d "venv" ]
then
    echo "::: ERROR: Virtual environment does not exist"
    exit 1
fi

echo "::: INFO: Entering virtual environment"
. venv/bin/activate

echo "::: INFO: Running Pylint" && \
    pylint --rcfile=pylintrc "${@}" && \
    echo "::: INFO: Running Mypy" && \
    mypy "${@}"

deactivate
