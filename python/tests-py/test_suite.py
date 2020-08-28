import json
import os
import subprocess
import sys

import pytest

import jsonschema_rs


@pytest.fixture(scope="session", autouse=True)
def mock_server():
    try:
        process = subprocess.Popen(args=[sys.executable, "../tests/suite/bin/jsonschema_suite", "serve"])
        yield
    finally:
        process.terminate()


SUPPORTED_DRAFTS = (4, 6, 7)
NOT_SUPPORTED_CASES = {
    4: ("bignum.json",),
    6: ("bignum.json",),
    7: ("bignum.json", "idn-hostname.json"),  # https://github.com/Stranger6667/jsonschema-rs/issues/101
}


def load_file(path):
    with open(path, mode="r", encoding="utf-8") as fd:
        for block in json.load(fd):
            yield block


def maybe_optional(draft, schema, instance, expected, description, filename):
    output = (filename, draft, schema, instance, expected, description)
    if filename in NOT_SUPPORTED_CASES.get(draft, ()):
        output = pytest.param(
            *output, marks=pytest.mark.skip(reason="{filename} is not supported".format(filename=filename))
        )
    return output


def pytest_generate_tests(metafunc):
    cases = [
        maybe_optional(draft, block["schema"], test["data"], test["valid"], test["description"], filename)
        for draft in SUPPORTED_DRAFTS
        for root, dirs, files in os.walk("../tests/suite/tests/draft{draft}/".format(draft=draft))
        for filename in files
        for block in load_file(os.path.join(root, filename))
        for test in block["tests"]
    ]
    metafunc.parametrize("filename, draft, schema, instance, expected, description", cases)


def test_draft(filename, draft, schema, instance, expected, description):
    try:
        result = jsonschema_rs.is_valid(schema, instance, int(draft))
        assert result is expected, "[{filename}] {description}: {schema} | {instance}".format(
            description=description,
            schema=schema,
            instance=instance,
            filename=filename,
        )
    except ValueError:
        pytest.fail(
            "[{filename}] {description}: {schema} | {instance}".format(
                description=description, schema=schema, instance=instance, filename=filename
            )
        )
