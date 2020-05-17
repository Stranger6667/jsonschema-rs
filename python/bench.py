import json
import timeit
from textwrap import dedent
import fastjsonschema


def load_json(filename):
    with open(filename) as fd:
        return json.load(fd)


BIG_SCHEMA = load_json("../benches/canada_schema.json")
BIG_INSTANCE = load_json("../benches/canada.json")
SMALL_SCHEMA = load_json("../benches/small_schema.json")
SMALL_INSTANCE_VALID = [9, 'hello', [1, 'a', True], {'a': 'a', 'b': 'b', 'd': 'd'}, 42, 3]

# Compiled fastjsonschema validators
fast_validate_big = fastjsonschema.compile(BIG_SCHEMA)
fast_validate_small = fastjsonschema.compile(SMALL_SCHEMA)


ITERATIONS_BIG = 10
ITERATIONS_SMALL = 10000
setup = dedent("""
    import json
    import jsonschema_rs
    import jsonschema
    from __main__ import (
        BIG_SCHEMA, 
        BIG_INSTANCE, 
        SMALL_SCHEMA,
        SMALL_INSTANCE_VALID,
        fast_validate_small,
        fast_validate_big,
    )
""")


def run(code, name, number):
    result = timeit.timeit(code, setup, number=number)
    print(f"{name} => {result:.5f}")


def bench_object(schema, instance, number, type_):
    code = f"jsonschema_rs.is_valid({schema}, {instance})"
    run(code, f"{type_}: object", number)


def bench_jsonschema(schema, instance, number, type_):
    code = f"jsonschema.validate({instance}, {schema})"
    run(code, f"{type_}: jsonschema", number)


def bench_fastjsonschema_small(instance, number, type_):
    code = f"fast_validate_small({instance})"
    run(code, f"{type_}: fast jsonschema", number)


def bench_fastjsonschema_big(instance, number, type_):
    code = f"fast_validate_big({instance})"
    run(code, f"{type_}: fast jsonschema", number)


def bench_object_noop(schema, instance, number, type_):
    code = f"jsonschema_rs.is_valid_noop({schema}, {instance})"
    run(code, f"{type_}: object noop", number)


if __name__ == '__main__':
    bench_object("SMALL_SCHEMA", "SMALL_INSTANCE_VALID", ITERATIONS_SMALL, "Small")
    bench_object_noop("SMALL_SCHEMA", "SMALL_INSTANCE_VALID", ITERATIONS_SMALL, "Small")
    bench_jsonschema("SMALL_SCHEMA", "SMALL_INSTANCE_VALID", ITERATIONS_SMALL, "Small")
    bench_fastjsonschema_small("SMALL_INSTANCE_VALID", ITERATIONS_SMALL, "Small")
    bench_object("BIG_SCHEMA", "BIG_INSTANCE", ITERATIONS_BIG, "Big  ")
    bench_object_noop("BIG_SCHEMA", "BIG_INSTANCE", ITERATIONS_BIG, "Big  ")
    bench_jsonschema("BIG_SCHEMA", "BIG_INSTANCE", ITERATIONS_BIG, "Big  ")
    bench_fastjsonschema_big("BIG_INSTANCE", ITERATIONS_BIG, "Big  ")
