import json
import os
import random
import socket
import subprocess
import sys
from time import sleep
from urllib.parse import urlparse

import pytest

import jsonschema_rs

TEST_SUITE_PATH = "../../jsonschema/tests/suite"
EXPONENTIAL_BASE = 2
JITTER = (0.0, 0.5)
INITIAL_RETRY_DELAY = 0.05
MAX_WAITING_RETRIES = 10


def is_available(url: str) -> bool:
    """Whether the `url` is available for connection or not."""
    parsed = urlparse(url)
    try:
        with socket.create_connection((parsed.hostname, parsed.port or 80)):
            return True
    except ConnectionError:
        return False


def wait_until_responsive(url: str, retries: int = MAX_WAITING_RETRIES, delay: float = INITIAL_RETRY_DELAY) -> None:
    while retries > 0:
        if is_available(url):
            return
        retries -= 1
        delay *= EXPONENTIAL_BASE
        delay += random.uniform(*JITTER)
        sleep(delay)
    raise RuntimeError(f"{url} is not available")


@pytest.fixture(scope="session", autouse=True)
def mock_server():
    process = subprocess.Popen(args=[sys.executable, f"{TEST_SUITE_PATH}/bin/jsonschema_suite", "serve"])
    wait_until_responsive("http://127.0.0.1:1234")
    try:
        yield
    finally:
        process.terminate()


SUPPORTED_DRAFTS = (4, 6, 7)
NOT_SUPPORTED_CASES = {
    4: ("bignum.json", "email.json"),
    6: ("bignum.json", "email.json"),
    7: ("bignum.json", "email.json", "idn-hostname.json"),
}


def load_file(path):
    with open(path, mode="r", encoding="utf-8") as fd:
        for block in json.load(fd):
            yield block


def maybe_optional(draft, schema, instance, expected, description, filename):
    output = (filename, draft, schema, instance, expected, description)
    if filename in NOT_SUPPORTED_CASES.get(draft, ()):
        output = pytest.param(*output, marks=pytest.mark.skip(reason=f"{filename} is not supported"))
    return output


def pytest_generate_tests(metafunc):
    cases = [
        maybe_optional(
            draft,
            block["schema"],
            test["data"],
            test["valid"],
            test["description"],
            filename,
        )
        for draft in SUPPORTED_DRAFTS
        for root, _, files in os.walk(f"{TEST_SUITE_PATH}/tests/draft{draft}/")
        for filename in files
        for block in load_file(os.path.join(root, filename))
        for test in block["tests"]
    ]
    metafunc.parametrize("filename, draft, schema, instance, expected, description", cases)


def test_draft(filename, draft, schema, instance, expected, description):
    error_message = f"[{filename}] {description}: {schema} | {instance}"
    try:
        result = jsonschema_rs.is_valid(schema, instance, int(draft), with_meta_schemas=True)
        assert result is expected, error_message
    except ValueError:
        pytest.fail(error_message)
