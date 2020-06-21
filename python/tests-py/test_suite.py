import json
import os
import sys

import pytest

import jsonschema_rs

IS_WINDOWS = sys.platform == "win32"
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
    output = (draft, schema, instance, expected, description)
    if filename in NOT_SUPPORTED_CASES.get(draft, ()):
        output = pytest.param(
            *output, marks=pytest.mark.skip(reason="{filename} is not supported".format(filename=filename))
        )
    if IS_WINDOWS and "$ref" in repr(schema):
        # TODO: Try to fix https://github.com/json-schema-org/JSON-Schema-Test-Suite.git to allow tests to
        # correctly run on Windows. The current state seems that flask has issues if there is a path separator
        # while running on Windows (as it would expect `\` instead of `/`)
        # We know that this is not ideal, but it should be good enough for now
        output = pytest.param(
            *output,
            marks=pytest.mark.skip(
                reason="{schema} contains $ref and is not yet supported for test on windows".format(schema=schema)
            ),
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
