import json
import sys
from contextlib import suppress
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


def load_json_str(filename):
    with open(filename) as fd:
        return fd.read()


def load_from_benches(filename, loader=load_json):
    return loader(f"../../jsonschema/benches/data/{filename}")


OPENAPI = load_from_benches("openapi.json")
ZUORA = load_from_benches("zuora.json")
SWAGGER = load_from_benches("swagger.json")
KUBERNETES = load_from_benches("kubernetes.json")
GEOJSON = load_from_benches("geojson.json")
CANADA = load_from_benches("canada.json")
CITM_CATALOG_SCHEMA = load_from_benches("citm_catalog_schema.json")
CITM_CATALOG = load_from_benches("citm_catalog.json")
FAST_SCHEMA = load_from_benches("fast_schema.json")
FAST_INSTANCE_VALID = [
    9,
    "hello",
    [1, "a", True],
    {"a": "a", "b": "b", "d": "d"},
    42,
    3,
]
FAST_INSTANCE_INVALID = [
    10,
    "world",
    [1, "a", True],
    {"a": "a", "b": "b", "c": "xy"},
    "str",
    5,
]


@pytest.fixture(params=[True, False], ids=("compiled", "raw"))
def is_compiled(request):
    return request.param


if jsonschema_rs is not None:
    variants = [
        "jsonschema-rs-is-valid",
        "jsonschema-rs-validate",
        "jsonschema",
        "fastjsonschema",
    ]
else:
    variants = ["jsonschema", "fastjsonschema"]


DEFAULT_BENCHMARK_CONFIG = {"iterations": 10, "rounds": 10, "warmup_rounds": 10}


@pytest.fixture(params=variants)
def variant(request):
    return request.param


@pytest.fixture
def args(request, variant, is_compiled):
    schema, instance = request.node.get_closest_marker("data").args
    if variant == "jsonschema-rs-is-valid":
        if is_compiled:
            return jsonschema_rs.JSONSchema(schema, with_meta_schemas=True).is_valid, instance
        else:
            return (
                partial(jsonschema_rs.is_valid, with_meta_schemas=True),
                schema,
                instance,
            )
    if variant == "jsonschema-rs-validate":
        if is_compiled:
            return jsonschema_rs.JSONSchema(schema, with_meta_schemas=True).validate, instance
        else:
            return (
                partial(jsonschema_rs.validate, with_meta_schemas=True),
                schema,
                instance,
            )
    if variant == "jsonschema":
        if is_compiled:
            return jsonschema.validators.validator_for(schema)(schema).is_valid, instance
        else:
            return jsonschema.validate, instance, schema
    if variant == "fastjsonschema":
        if is_compiled:
            return fastjsonschema.compile(schema, use_default=False), instance
        else:
            return partial(fastjsonschema.validate, use_default=False), schema, instance


@pytest.mark.parametrize(
    "name",
    (
        "openapi.json",
        "swagger.json",
        "geojson.json",
        "citm_catalog_schema.json",
        "fast_schema.json",
    ),
)
@pytest.mark.parametrize(
    "func",
    (
        lambda x: jsonschema_rs.JSONSchema(json.loads(x)),
        jsonschema_rs.JSONSchema.from_str,
    ),
    ids=["py-parse", "rs-parse"],
)
@pytest.mark.benchmark(group="create schema")
def test_create_schema(benchmark, func, name):
    benchmark.group = f"{name}: {benchmark.group}"
    schema = load_from_benches(name, loader=load_json_str)
    benchmark(func, schema)


# Small schemas


@pytest.mark.data(True, True)
@pytest.mark.benchmark(group="boolean")
def test_boolean(benchmark, args):
    benchmark(*args)


@pytest.mark.data({"minimum": 10}, 10)
@pytest.mark.benchmark(group="minimum")
def test_minimum(benchmark, args):
    benchmark(*args)


@pytest.mark.data(FAST_SCHEMA, FAST_INSTANCE_VALID)
@pytest.mark.benchmark(group="fast-valid")
def test_fast_valid(benchmark, args):
    benchmark(*args)


@pytest.mark.data(FAST_SCHEMA, FAST_INSTANCE_VALID)
@pytest.mark.benchmark(group="fast-invalid")
def test_fast_invalid(benchmark, args):
    def func():
        with suppress(Exception):
            args[0](*args[1:])

    benchmark(func)


# Large schemas


@pytest.mark.data(OPENAPI, ZUORA)
@pytest.mark.benchmark(group="openapi")
def test_openapi(benchmark, args):
    benchmark(*args)


@pytest.mark.data(SWAGGER, KUBERNETES)
@pytest.mark.benchmark(group="swagger")
def test_swagger(benchmark, args):
    benchmark(*args)


@pytest.mark.data(GEOJSON, CANADA)
@pytest.mark.benchmark(group="canada")
def test_canada(benchmark, args):
    benchmark(*args)


@pytest.mark.data(CITM_CATALOG_SCHEMA, CITM_CATALOG)
@pytest.mark.benchmark(group="citm_catalog")
def test_citm_catalog(benchmark, args):
    benchmark(*args)
