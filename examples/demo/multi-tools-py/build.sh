#!/bin/bash
set -e
mkdir -p dist
/opt/homebrew/bin/python3.11 -m venv venv
. venv/bin/activate
pip install -r requirements.txt
componentize-py -w spin-http componentize app -o dist/multi-tools-py.wasm