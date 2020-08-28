from contextlib import suppress

import pytest
from hypothesis import given
from hypothesis import strategies as st

from jsonschema_rs import JSONSchema, ValidationError, is_valid

json = st.recursive(
    st.none() | st.booleans() | st.floats() | st.integers() | st.text(),
    lambda children: st.lists(children, min_size=1) | st.dictionaries(st.text(), children, min_size=1),
)


@given(instance=json)
def test_instance_processing(instance):
    with suppress(Exception):
        is_valid(True, instance)


@given(instance=json)
def test_schema_processing(instance):
    with suppress(Exception):
        is_valid(instance, True)


def test_invalid_schema():
    with pytest.raises(ValueError):
        is_valid(2 ** 64, True)


def test_invalid_type():
    with pytest.raises(ValueError, match="Unsupported type: 'set'"):
        is_valid(set(), True)


def test_repr():
    assert repr(JSONSchema({"minimum": 5})) == '<JSONSchema: {"minimum":5}>'


def test_validate():
    schema = JSONSchema({"minimum": 5})
    with pytest.raises(ValidationError, match="2 is less than the minimum of 5"):
        schema.validate(2)


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
        ([], None, "Invalid schema"),
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
