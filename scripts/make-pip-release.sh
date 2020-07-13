#!/usr/bin/env sh
## make-pip-release.sh
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

if [ $# -lt 2 ]
then
    echo "usage: make-pip-release.sh app version"
    echo
    echo "Prepare source files for release and generate distribution files."
    echo "NOTE: Should _not_ be run in a virtual environment"
    echo
    echo "positional arguments:"
    echo "app       Name of the app to release (without 'hare_' prefix)"
    echo "version   Release version (should follow semver)"
    echo
    exit 1
fi

# Set package name, release version, and release tag name options
APP_NAME="${1}"
PACKAGE_NAME="hare_${APP_NAME}"
PACKAGE_VERSION="${2}"
TAG_NAME="v${PACKAGE_VERSION}"

# Parse branch name
BRANCH_NAME=$(git rev-parse --abbrev-ref HEAD)

echo "::: INFO: Creating release for ${PACKAGE_NAME} version ${PACKAGE_VERSION}"

# Ensure package directory exists and has README.md and LICENSE files
if ! [ -d "${PACKAGE_NAME}" ]
then
    echo "::: ERROR: Package directory ${PACKAGE_NAME} does not exist"
    exit 1
fi
# The following section isn't necessary for now, as the repo
# only contains the engine source.
## for FILENAME in "README.md" "LICENSE"
## do
##     if ! [ -f "${PACKAGE_NAME}/${FILENAME}" ]
##     then
##         echo "::: ERROR: Package directory must contain ${FILENAME} file"
##         exit 1
##     else
##         echo "::: INFO: Moving ${FILENAME} to root directory and re-adding to source control"
##         mv "${PACKAGE_NAME}/${FILENAME}" .
##         git add "${FILENAME}"
##     fi
## done

# Remove all other app directories
# NOTE: All app directories must be named 'hare_<app_name>' and be defined at top level
echo "::: INFO: Removing all other app directories" 
find -E . -type d -regex "./hare_[a-z]+" \! -regex "./${PACKAGE_NAME}" -delete

# Remove tests from source file(s)
find "${PACKAGE_NAME}" -type f -iregex '.*\.py' -exec ./scripts/remove-tests.sh {} \;

echo "::: INFO: Generating Setuptools setup.py file and removing template from app directory"
cat "${PACKAGE_NAME}/scripts/setup.py" | sed -e "s/PACKAGE_VERSION/${PACKAGE_VERSION}/g" > setup.py
chmod 744 setup.py
git rm "${PACKAGE_NAME}/scripts/setup.py"

echo "::: INFO: Generating source distribution to dist directory"
python3 setup.py sdist

echo "::: INFO: Generating 'pure' wheel file to dist directory"
python3 setup.py bdist_wheel

# Remove development files from root and app-specific directories
./scripts/remove-dev-files.sh
cd "${PACKAGE_NAME}"
../scripts/remove-dev-files.sh
cd ../

# Add the distribution files to source control
echo "::: INFO: Adding distribution files to source control"
git add setup.py dist/

# Commit changes
echo "::: INFO: Committing release"
git commit -am "Made ${PACKAGE_NAME} release ${PACKAGE_VERSION}"

# Create local tag for release
echo "::: INFO: Creating tag for release (${TAG_NAME})"
git tag -m "${PACKAGE_NAME} release ${TAG_NAME}" -a "${TAG_NAME}"

# Push commit and tag to remote
echo "::: INFO: Pushing release to remote"
git push origin "${BRANCH_NAME}"
git push origin "${TAG_NAME}"
