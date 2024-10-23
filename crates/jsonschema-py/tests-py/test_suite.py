import json
import os
import random
import socket
import subprocess
import sys
from pathlib import Path
from time import sleep
from urllib.parse import urlparse

import pytest

import jsonschema_rs

TEST_SUITE_PATH = Path(__file__).parent.parent.parent / "jsonschema/tests/suite"
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
    process = subprocess.Popen(args=[sys.executable, TEST_SUITE_PATH / "bin/jsonschema_suite", "serve"])
    wait_until_responsive("http://127.0.0.1:1234")
    try:
        yield
    finally:
        process.terminate()


SUPPORTED_DRAFTS = ("4", "6", "7", "2019-09", "2020-12")
NOT_SUPPORTED_CASES = {
    "4": ("bignum.json",),
    "6": ("bignum.json",),
    "7": ("bignum.json",),
    "2019-09": ("bignum.json",),
    "2020-12": ("bignum.json",),
}


def load_file(path):
    with open(path, mode="r", encoding="utf-8") as fd:
        raw = fd.read().replace("https://localhost:1234", "http://127.0.0.1:1234")
        for block in json.loads(raw):
            yield block


def maybe_optional(draft, schema, instance, expected, description, filename, is_optional):
    output = (filename, draft, schema, instance, expected, description, is_optional)
    if filename in NOT_SUPPORTED_CASES.get(draft, ()):
        output = pytest.param(*output, marks=pytest.mark.skip(reason=f"{filename} is not supported"))
    return output


def pytest_generate_tests(metafunc):
    cases = [
        maybe_optional(
            draft, block["schema"], test["data"], test["valid"], test["description"], filename, "optional" in str(root)
        )
        for draft in SUPPORTED_DRAFTS
        for root, _, files in os.walk(TEST_SUITE_PATH / f"tests/draft{draft}/")
        for filename in files
        for block in load_file(os.path.join(root, filename))
        for test in block["tests"]
    ]
    metafunc.parametrize("filename, draft, schema, instance, expected, description, is_optional", cases)


def test_draft(filename, draft, schema, instance, expected, description, is_optional):
    error_message = f"[{filename}] {description}: {schema} | {instance}"
    try:
        cls = {
            "4": jsonschema_rs.Draft4Validator,
            "6": jsonschema_rs.Draft6Validator,
            "7": jsonschema_rs.Draft7Validator,
            "2019-09": jsonschema_rs.Draft201909Validator,
            "2020-12": jsonschema_rs.Draft202012Validator,
        }[draft]
        kwargs = {}
        if is_optional:
            kwargs["validate_formats"] = True
        result = cls(schema, **kwargs).is_valid(instance)
        assert result is expected, error_message
    except ValueError:
        pytest.fail(error_message)
