import json
import sys
from functools import partial

import fastjsonschema
import jsonschema
import pytest

if sys.implementation.name != "pypy":
    import jsonschema_rs
else:
    jsonschema_rs = None


def load_json(filename):
    with open(filename) as fd:
        return json.load(fd)


BIG_SCHEMA = load_json("../../jsonschema/benches/swagger.json")
BIG_INSTANCE = load_json("../../jsonschema/benches/kubernetes.json")
SMALL_SCHEMA = load_json("../../jsonschema/benches/small_schema.json")
SMALL_INSTANCE_VALID = [9, "hello", [1, "a", True], {"a": "a", "b": "b", "d": "d"}, 42, 3]


@pytest.fixture(params=[True, False], ids=("compiled", "raw"))
def is_compiled(request):
    return request.param


if jsonschema_rs is not None:
    variants = ["jsonschema-rs", "jsonschema", "fastjsonschema"]
else:
    variants = ["jsonschema", "fastjsonschema"]


@pytest.fixture(params=variants)
def args(request, is_compiled):
    schema, instance = request.node.get_closest_marker("data").args
    if request.param == "jsonschema-rs":
        if is_compiled:
            return jsonschema_rs.JSONSchema(schema, with_meta_schemas=True).is_valid, instance
        else:
            return partial(jsonschema_rs.is_valid, with_meta_schemas=True), schema, instance
    if request.param == "jsonschema":
        if is_compiled:
            return jsonschema.validators.validator_for(schema)(schema).is_valid, instance
        else:
            return jsonschema.validate, instance, schema
    if request.param == "fastjsonschema":
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
