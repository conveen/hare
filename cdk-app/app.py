#!/usr/bin/env python3
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

import json
from os import environ as ENV
from pathlib import Path
from typing import Any, Dict

from aws_cdk import core

from cdk_app.scaffolding_stack import ScaffoldingStack


__author__ = "conveen"
__version__ = "0.1.0"
_REQUIRED_CONFIG_OPTIONS = (
    "account",
    "region",
    "certificate_arn",
)


def gen_config() -> Dict[Any, Any]:
    if not "CDK_CONFIG_FILE" in ENV:
        raise ValueError("must provide CDK_CONFIG_FILE env variable")
    config_file_path = Path(ENV["CDK_CONFIG_FILE"])
    if not config_file_path.is_file():
        raise FileNotFoundError(f"must provide valid config file path (found {config_file_path})")

    with config_file_path.open("rb") as config_file:
        config = json.load(config_file)
    for option in _REQUIRED_CONFIG_OPTIONS:
        if not option in config or config[option] in {"", None}:
            raise ValueError(f"must provide config option '{option}'")
    return config


def main() -> int:
    config = gen_config()
    env = core.Environment(account=config["account"], region=config["region"])
    app = core.App()

    scaffolding_stack = ScaffoldingStack(app,
                                         "hare-scaffolding",
                                         certificate_arn=config["certificate_arn"],
                                         env=env)

    app.synth()
    return 0


if __name__ == "__main__":
    main()
