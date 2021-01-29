#!/bin/bash
set -ex

# Create a symlink for jsonschema
ln -sf ../../jsonschema jsonschema-lib
# Modify Cargo.toml to include this symlink
cp Cargo.toml Cargo.toml.orig
sed -i 's/\.\.\/\.\.\/jsonschema/\.\/jsonschema-lib/' Cargo.toml
# Build the source distribution
python setup.py sdist
rm jsonschema-lib
mv Cargo.toml.orig Cargo.toml
