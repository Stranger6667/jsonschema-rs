#!/bin/bash
set -ex

yum install openssl-devel -y

curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
export PATH="$HOME/.cargo/bin:$PATH"

for PYBIN in /opt/python/{cp35-cp35m,cp36-cp36m,cp37-cp37m,cp38-cp38}/bin; do
    export PYTHON_SYS_EXECUTABLE="$PYBIN/python"

    "${PYBIN}/pip" install -U setuptools wheel setuptools-rust
    "${PYBIN}/python" setup.py bdist_wheel
    rm build/lib/jsonschema_rs/*.so
done

for whl in dist/*.whl; do
    auditwheel repair "$whl" -w dist/
    rm "$whl"
done
