import json
import os

import pytest

import jsonschema_rs

SUPPORTED_DRAFTS = (4, 6, 7)
NOT_SUPPORTED_CASES = {4: ("bignum.json",), 6: ("bignum.json",), 7: ("bignum.json",)}


def load_file(path):
    with open(path) as fd:
        for block in json.load(fd):
            yield block


def maybe_optional(draft, schema, instance, expected, description, filename):
    output = (draft, schema, instance, expected, description)
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
    metafunc.parametrize("draft, schema, instance, expected, description", cases)


def test_draft(draft, schema, instance, expected, description):
    try:
        result = jsonschema_rs.is_valid(schema, instance, int(draft))
        assert result is expected, "{description}: {schema} | {instance}".format(
            description=description, schema=schema, instance=instance
        )
    except ValueError:
        pytest.fail(
            "{description}: {schema} | {instance}".format(description=description, schema=schema, instance=instance)
        )
