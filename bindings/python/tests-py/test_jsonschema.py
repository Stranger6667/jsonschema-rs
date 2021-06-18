from contextlib import suppress
from functools import partial

import pytest
from hypothesis import given
from hypothesis import strategies as st

from jsonschema_rs import JSONSchema, ValidationError, is_valid, validate

json = st.recursive(
    st.none() | st.booleans() | st.floats() | st.integers() | st.text(),
    lambda children: st.lists(children, min_size=1) | st.dictionaries(st.text(), children, min_size=1),
)


@pytest.mark.parametrize("func", (is_valid, validate))
@given(instance=json)
def test_instance_processing(func, instance):
    with suppress(Exception):
        func(True, instance)


@pytest.mark.parametrize("func", (is_valid, validate))
@given(instance=json)
def test_schema_processing(func, instance):
    with suppress(Exception):
        func(instance, True)


@pytest.mark.parametrize("func", (is_valid, validate))
def test_invalid_schema(func):
    with pytest.raises(ValueError):
        func(2 ** 64, True)


@pytest.mark.parametrize("func", (is_valid, validate))
def test_invalid_type(func):
    with pytest.raises(ValueError, match="Unsupported type: 'set'"):
        func(set(), True)


def test_repr():
    assert repr(JSONSchema({"minimum": 5})) == '<JSONSchema: {"minimum":5}>'


@pytest.mark.parametrize("func", (JSONSchema({"minimum": 5}).validate, partial(validate, {"minimum": 5})))
def test_validate(func):
    with pytest.raises(ValidationError, match="2 is less than the minimum of 5"):
        func(2)


def test_recursive_dict():
    instance = {}
    instance["foo"] = instance
    with pytest.raises(ValueError):
        is_valid(True, instance)


def test_recursive_list():
    instance = []
    instance.append(instance)
    with pytest.raises(ValueError):
        is_valid(True, instance)


@pytest.mark.parametrize(
    "schema, draft, error",
    (
        ([], None, r'\[\] is not of types "boolean", "object"'),
        ({}, 5, "Unknown draft: 5"),
    ),
)
def test_initialization_errors(schema, draft, error):
    with pytest.raises(ValueError, match=error):
        JSONSchema(schema, draft)


@given(minimum=st.integers().map(abs))
def test_minimum(minimum):
    with suppress(SystemError):
        assert is_valid({"minimum": minimum}, minimum)
        assert is_valid({"minimum": minimum}, minimum - 1) is False


@given(maximum=st.integers().map(abs))
def test_maximum(maximum):
    with suppress(SystemError):
        assert is_valid({"maximum": maximum}, maximum)
        assert is_valid({"maximum": maximum}, maximum + 1) is False


@pytest.mark.xfail(reason="The underlying Rust crate has not enough precision.")
@given(multiple_of=(st.integers() | st.floats(allow_infinity=False, allow_nan=False)).filter(lambda x: x > 0))
def test_multiple_of(multiple_of):
    with suppress(SystemError):
        assert is_valid({"multipleOf": multiple_of}, multiple_of * 3)


@pytest.mark.parametrize("method", ("is_valid", "validate"))
def test_invalid_value(method):
    schema = JSONSchema({"minimum": 42})
    with pytest.raises(ValueError, match="Unsupported type: 'object'"):
        getattr(schema, method)(object())


def test_error_message():
    schema = {"properties": {"foo": {"type": "integer"}}}
    instance = {"foo": None}
    try:
        validate(schema, instance)
        pytest.fail("Validation error should happen")
    except ValidationError as exc:
        assert (
            str(exc)
            == """null is not of type "integer"

Failed validating "type" in schema["properties"]["foo"]["type"]

On instance["foo"]:
    null"""
        )
