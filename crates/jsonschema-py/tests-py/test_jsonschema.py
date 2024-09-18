import sys
import uuid
from collections import OrderedDict, namedtuple
from contextlib import suppress
from enum import Enum
from functools import partial

import pytest
from hypothesis import given
from hypothesis import strategies as st

from jsonschema_rs import ValidationError, is_valid, iter_errors, validate, validator_for

json = st.recursive(
    st.none()
    | st.booleans()
    | st.floats()
    | st.integers(min_value=-sys.maxsize - 1, max_value=sys.maxsize)
    | st.text(),
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
    try:
        func(instance, True)
    except Exception:
        pass


@pytest.mark.parametrize("func", (is_valid, validate))
def test_invalid_schema(func):
    with pytest.raises(ValueError):
        func(2**64, True)


@pytest.mark.parametrize("func", (is_valid, validate))
def test_invalid_type(func):
    with pytest.raises(ValueError, match="Unsupported type: 'set'"):
        func(set(), True)


def test_repr():
    assert repr(validator_for({"minimum": 5})) == '<Draft7Validator: {"minimum":5}>'


@pytest.mark.parametrize(
    "func",
    (
        validator_for({"minimum": 5}).validate,
        validator_for('{"minimum": 5}').validate,
        partial(validate, {"minimum": 5}),
    ),
)
def test_validate(func):
    with pytest.raises(ValidationError, match="2 is less than the minimum of 5"):
        func(2)


def test_from_str_error():
    with pytest.raises(ValidationError, match='42 is not of types "boolean", "object"'):
        validator_for(42)  # type: ignore


@pytest.mark.parametrize(
    "val",
    (
        ("A", "B", "C"),
        ["A", "B", "C"],
    ),
)
def test_array_tuple(val):
    schema = {"type": "array", "items": {"type": "string"}}
    validate(schema, val)


@pytest.mark.parametrize(
    "val",
    ((1, 2, 3), [1, 2, 3], {"foo": 1}),
)
def test_array_tuple_invalid(val):
    schema = {"type": "array", "items": {"type": "string"}}
    with pytest.raises(ValueError):
        validate(schema, val)


def test_named_tuple():
    Person = namedtuple("Person", "first_name last_name")
    person_a = Person("Joe", "Smith")
    schema = {"type": "array", "items": {"type": "string"}}
    with pytest.raises(ValueError):
        validate(schema, person_a)


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


def test_paths():
    with pytest.raises(ValidationError) as exc:
        validate({"items": [{"type": "string"}]}, [1])
    assert exc.value.schema_path == ["items", 0, "type"]
    assert exc.value.instance_path == [0]
    assert exc.value.message == '1 is not of type "string"'


@given(minimum=st.integers().map(abs))
def test_minimum(minimum):
    with suppress(SystemError, ValueError):
        assert is_valid({"minimum": minimum}, minimum)
        assert is_valid({"minimum": minimum}, minimum - 1) is False


@given(maximum=st.integers().map(abs))
def test_maximum(maximum):
    with suppress(SystemError, ValueError):
        assert is_valid({"maximum": maximum}, maximum)
        assert is_valid({"maximum": maximum}, maximum + 1) is False


@pytest.mark.parametrize("method", ("is_valid", "validate"))
def test_invalid_value(method):
    schema = validator_for({"minimum": 42})
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

Failed validating "type" in schema["properties"]["foo"]

On instance["foo"]:
    null"""
        )


SCHEMA = {"properties": {"foo": {"type": "integer"}, "bar": {"type": "string"}}}


@pytest.mark.parametrize(
    "func",
    (
        validator_for(SCHEMA).iter_errors,
        partial(iter_errors, SCHEMA),
    ),
)
def test_iter_err_message(func):
    errors = func({"foo": None, "bar": None})

    first = next(errors)
    assert first.message == 'null is not of type "string"'

    second = next(errors)
    assert second.message == 'null is not of type "integer"'

    with suppress(StopIteration):
        next(errors)
        pytest.fail("Validation error should happen")


@pytest.mark.parametrize(
    "func",
    (
        validator_for({"properties": {"foo": {"type": "integer"}}}).iter_errors,
        partial(iter_errors, {"properties": {"foo": {"type": "integer"}}}),
    ),
)
def test_iter_err_empty(func):
    instance = {"foo": 1}
    errs = func(instance)
    with suppress(StopIteration):
        next(errs)
        pytest.fail("Validation error should happen")


class StrEnum(Enum):
    bar = "bar"
    foo = "foo"


class IntEnum(Enum):
    bar = 1
    foo = 2


@pytest.mark.parametrize(
    "type_, value, expected",
    (
        ("number", IntEnum.bar, True),
        ("number", StrEnum.bar, False),
        ("string", IntEnum.bar, False),
        ("string", StrEnum.bar, True),
    ),
)
def test_enums(type_, value, expected):
    schema = {"properties": {"foo": {"type": type_}}}
    instance = {"foo": value}
    assert is_valid(schema, instance) is expected


def test_dict_with_non_str_keys():
    schema = {"type": "object"}
    instance = {uuid.uuid4(): "foo"}
    with pytest.raises(ValueError) as exec_info:
        validate(schema, instance)
    assert exec_info.value.args[0] == "Dict key must be str. Got 'UUID'"


class MyDict(dict):
    pass


class MyDict2(MyDict):
    pass


@pytest.mark.parametrize(
    "type_, value, expected",
    (
        (dict, 1, True),
        (dict, "bar", False),
        (OrderedDict, 1, True),
        (OrderedDict, "bar", False),
        (MyDict, 1, True),
        (MyDict, "bar", False),
        (MyDict2, 1, True),
        (MyDict2, "bar", False),
    ),
)
def test_dict_subclasses(type_, value, expected):
    schema = {"type": "object", "properties": {"foo": {"type": "integer"}}}
    document = type_({"foo": value})
    assert is_valid(schema, document) is expected


def test_custom_format():
    def is_currency(value):
        return len(value) == 3 and value.isascii()

    validator = validator_for({"type": "string", "format": "currency"}, formats={"currency": is_currency})
    assert validator.is_valid("USD")
    assert not validator.is_valid(42)
    assert not validator.is_valid("invalid")


def test_custom_format_invalid_callback():
    with pytest.raises(ValueError, match="Format checker for 'currency' must be a callable"):
        validator_for({"type": "string", "format": "currency"}, formats={"currency": 42})


def test_custom_format_with_exception():
    def is_currency(_):
        raise ValueError("Invalid currency")

    schema = {"type": "string", "format": "currency"}
    formats = {"currency": is_currency}
    validator = validator_for(schema, formats=formats)
    with pytest.raises(ValueError, match="Invalid currency"):
        validator.is_valid("USD")
    with pytest.raises(ValueError, match="Invalid currency"):
        validator.validate("USD")
    with pytest.raises(ValueError, match="Invalid currency"):
        for _ in validator.iter_errors("USD"):
            pass
    with pytest.raises(ValueError, match="Invalid currency"):
        is_valid(schema, "USD", formats=formats)
    with pytest.raises(ValueError, match="Invalid currency"):
        validate(schema, "USD", formats=formats)
    with pytest.raises(ValueError, match="Invalid currency"):
        for _ in iter_errors(schema, "USD", formats=formats):
            pass
