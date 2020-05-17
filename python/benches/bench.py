import json

import fastjsonschema
import jsonschema
import pytest

import jsonschema_rs


def load_json(filename):
    with open(filename) as fd:
        return json.load(fd)


BIG_SCHEMA = load_json("../benches/canada_schema.json")
BIG_INSTANCE = load_json("../benches/canada.json")
SMALL_SCHEMA = load_json("../benches/small_schema.json")
SMALL_INSTANCE_VALID = [9, "hello", [1, "a", True], {"a": "a", "b": "b", "d": "d"}, 42, 3]


@pytest.fixture(params=[True, False])
def is_compiled(request):
    return request.param


@pytest.fixture(params=["rust", "python", "python-fast"])
def args(request, is_compiled):
    schema, instance = request.node.get_closest_marker("data").args
    if request.param == "rust":
        if is_compiled:
            return jsonschema_rs.JSONSchema(schema).is_valid, instance
        else:
            return jsonschema_rs.is_valid, schema, instance
    if request.param == "python":
        if is_compiled:
            return jsonschema.validators.validator_for(schema)(schema).is_valid, instance
        else:
            return jsonschema.validate, instance, schema
    if request.param == "python-fast":
        if is_compiled:
            return fastjsonschema.compile(schema), instance
        else:
            return fastjsonschema.validate, schema, instance


@pytest.mark.data(True, True)
@pytest.mark.benchmark(group="boolean")
def test_boolean(benchmark, args):
    benchmark(*args)


@pytest.mark.data({"minimum": 10}, 10)
@pytest.mark.benchmark(group="minimum")
def test_minimum(benchmark, args):
    benchmark(*args)


@pytest.mark.data(SMALL_SCHEMA, SMALL_INSTANCE_VALID)
@pytest.mark.benchmark(group="small")
def test_small_schema(benchmark, args):
    benchmark(*args)


@pytest.mark.data(BIG_SCHEMA, BIG_INSTANCE)
@pytest.mark.benchmark(group="big")
def test_big_schema(benchmark, args):
    benchmark(*args)
