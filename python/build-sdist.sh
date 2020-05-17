#!/bin/bash
# `setuptools_rust` and `maturin` don't support some local dependencies as `jsonschema` is (it is in the parent directory)
# As a workaround we create a modified distribution of this library that has `jsonschema` crate as a dependency in
# the same directory, then the sources are copied as declared in MANIFEST.in and the resulting package can be
# installed properly
set -ex

ln -sf ../ jsonschema
# Modify cargo.toml to include this symlink
sed -i 's/\.\.\//jsonschema/' Cargo.toml
# Build the source distribution
python setup.py sdist
# Rollback local changes after a source distribution is ready
rm jsonschema
sed -i 's/"jsonschema"/"\.\.\/"/' Cargo.toml
