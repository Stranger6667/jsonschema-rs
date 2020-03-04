use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jsonschema::*;
use jsonschema_valid;
use jsonschema_valid::schemas;
use serde_json::{from_str, json, Value};
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn read_json(filepath: &str) -> Value {
    let path = Path::new(filepath);
    let mut file = File::open(&path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).ok().unwrap();
    let data: Value = from_str(&content).unwrap();
    data
}

macro_rules! bench_validate {
    ($b:ident, $name:expr, $schema:tt, $data: tt) => {
        fn $b(c: &mut Criterion) {
            let schema = json!($schema);
            let validator = JSONSchema::compile(&schema, None).unwrap();
            let data = black_box(json!($data));
            c.bench_function($name, |b| b.iter(|| validator.is_valid(&data)));
        }
    };
}

macro_rules! bench_compile {
    ($b:ident, $name:expr, $schema:tt) => {
        fn $b(c: &mut Criterion) {
            let schema = black_box(json!($schema));
            c.bench_function($name, |b| b.iter(|| JSONSchema::compile(&schema, None)));
        }
    };
}

fn canada_benchmark(c: &mut Criterion) {
    let schema = black_box(read_json("benches/canada_schema.json"));
    let data = black_box(read_json("benches/canada.json"));
    let validator = JSONSchema::compile(&schema, None).unwrap();
    c.bench_function("canada bench", |b| b.iter(|| validator.is_valid(&data)));
}

fn canada_benchmark_alternative(c: &mut Criterion) {
    let schema = black_box(read_json("benches/canada_schema.json"));
    let data = black_box(read_json("benches/canada.json"));
    let cfg = jsonschema_valid::Config::from_schema(&schema, Some(schemas::Draft::Draft7)).unwrap();
    c.bench_function("canada bench alternative", |b| {
        b.iter(|| jsonschema_valid::validate(&cfg, &data))
    });
}

fn canada_compile_benchmark(c: &mut Criterion) {
    let schema = black_box(read_json("benches/canada_schema.json"));
    c.bench_function("canada compile", |b| {
        b.iter(|| JSONSchema::compile(&schema, None).unwrap())
    });
}

fn fastjsonschema_compile(c: &mut Criterion) {
    let schema = read_json("benches/fast_schema.json");
    c.bench_function("fastjsonschema compile", |b| {
        b.iter(|| JSONSchema::compile(&schema, None).unwrap())
    });
}
fn fastjsonschema_valid_benchmark(c: &mut Criterion) {
    let schema = black_box(read_json("benches/fast_schema.json"));
    let validator = JSONSchema::compile(&schema, None).unwrap();
    let data =
        black_box(json!([9, "hello", [1, "a", true], {"a": "a", "b": "b", "d": "d"}, 42, 3]));
    c.bench_function("fastjsonschema valid", |b| {
        b.iter(|| validator.is_valid(&data))
    });
}

fn fastjsonschema_invalid_benchmark(c: &mut Criterion) {
    let schema = black_box(read_json("benches/fast_schema.json"));
    let validator = JSONSchema::compile(&schema, None).unwrap();
    let data =
        black_box(json!([10, "world", [1, "a", true], {"a": "a", "b": "b", "c": "xy"}, "str", 5]));
    c.bench_function("fastjsonschema invalid", |b| {
        b.iter(|| validator.is_valid(&data))
    });
}

fn format_time_benchmark(c: &mut Criterion) {
    let schema = black_box(json!({"type": "string", "format": "time"}));
    let validator = JSONSchema::compile(&schema, None).unwrap();
    let data = black_box(json!("10:00:00Z"));
    c.bench_function("format time", |b| b.iter(|| validator.validate(&data)));
}

fn max_length_benchmark(c: &mut Criterion) {
    let schema = json!({"maxLength": 5});
    let validator = JSONSchema::compile(&schema, None).unwrap();
    let data = black_box(json!("abc"));
    c.bench_function("max length", |b| b.iter(|| validator.validate(&data)));
}

bench_validate!(
    additional_items_valid,
    "additional items valid",
    {"items": [{}, {}, {}], "additionalItems": false},
    [1, 2, 3]
);
bench_validate!(
    additional_items_invalid,
    "additional items invalid",
    {"items": [{}, {}, {}], "additionalItems": false},
    [1, 2, 3, 4]
);
bench_validate!(
    additional_properties_valid1,
    "additional properties valid 1",
    {
        "properties": {"foo": {}, "bar": {}},
        "additionalProperties": {"type": "boolean"}
    },
    {"foo" : 1, "bar" : 2, "quux" : true}
);
bench_validate!(
    additional_properties_invalid1,
    "additional properties invalid 1",
    {
        "properties": {"foo": {}, "bar": {}},
        "additionalProperties": {"type": "boolean"}
    },
    {"foo" : 1, "bar" : 2, "quux" : 12}
);
bench_validate!(
    additional_properties_valid2,
    "additional properties valid 2",
    {"additionalProperties": {"type": "boolean"}},
    {"foo" : true}
);
bench_validate!(
    additional_properties_invalid2,
    "additional properties invalid 2",
    {"additionalProperties": {"type": "boolean"}},
    {"foo" : 1}
);
bench_validate!(
    additional_properties_valid3,
    "additional properties valid 3",
    {"additionalProperties": false},
    {}
);
bench_validate!(
    additional_properties_invalid3,
    "additional properties invalid 3",
    {"additionalProperties": false},
    {"foo" : 1}
);
bench_validate!(
    additional_properties_valid4,
    "additional properties valid 4",
    {
        "properties": {"foo": {}, "bar": {}},
        "additionalProperties": false
    },
    {"foo" : 1, "bar" : 2}
);
bench_validate!(
    additional_properties_invalid4,
    "additional properties invalid 4",
    {
        "properties": {"foo": {}, "bar": {}},
        "additionalProperties": false
    },
    {"foo" : 1, "bar" : 2, "quux" : 12}
);
bench_validate!(
    additional_properties_valid5,
    "additional properties valid 5",
    {
        "properties": {"foo": {}, "bar": {}},
        "patternProperties": { "^v": {} },
        "additionalProperties": false
    },
    {"foo": 1}
);
bench_validate!(
    additional_properties_invalid5,
    "additional properties invalid 5",
    {
        "properties": {"foo": {}, "bar": {}},
        "patternProperties": { "^v": {} },
        "additionalProperties": false
    },
    {"foo" : 1, "bar" : 2, "quux" : "boom"}
);
bench_validate!(
    additional_properties_valid6,
    "additional properties valid 6",
    {
        "properties": {"foo": {}, "bar": {}},
        "patternProperties": { "^v": {} },
        "additionalProperties": {"type": "integer"}
    },
    {"foo": 1}
);
bench_validate!(
    additional_properties_invalid6,
    "additional properties invalid 6",
    {
        "properties": {"foo": {}, "bar": {}},
        "patternProperties": { "^v": {} },
        "additionalProperties": {"type": "integer"}
    },
    {"foo" : 1, "bar" : 2, "quux" : "boom"}
);
bench_validate!(all_of_valid, "allOf valid", {"allOf": [{"type": "integer"}, {"minimum": 2}]}, 4);
bench_validate!(all_of_invalid, "allOf invalid", {"allOf": [{"type": "integer"}, {"minimum": 2}]}, 1);
bench_validate!(any_of_valid, "anyOf valid", {"anyOf": [{"type": "integer"}, {"minimum": 2}]}, 1);
bench_validate!(any_of_invalid, "anyOf invalid", {"anyOf": [{"type": "integer"}, {"minimum": 2}]}, 1.5);
bench_validate!(one_of_valid, "oneOf valid", {"oneOf": [{"type": "integer"}, {"minimum": 2}]}, 1);
bench_validate!(one_of_invalid1, "oneOf invalid1", {"oneOf": [{"type": "integer"}, {"minimum": 2}]}, 3);
bench_validate!(enum_valid, "enum invalid", {"enum": [1, 2, 3, 4]}, 4);
bench_validate!(enum_invalid, "enum invalid", {"enum": [1, 2, 3, 4]}, 5);
bench_validate!(contains_valid, "contains valid", {"contains": {"minimum": 5}}, [5]);
bench_validate!(contains_invalid, "contains invalid", {"contains": {"minimum": 5}}, [1]);
bench_validate!(const_valid, "const valid", {"const": 1}, 1);
bench_validate!(const_invalid1, "const invalid1", {"const": 1}, 2);
bench_validate!(const_invalid2, "const invalid1", {"const": 1}, "2");
bench_validate!(false_schema, "false schema", false, 1);
bench_validate!(format_ipv4_valid, "format ipv4 valid", {"format": "ipv4"}, "127.0.0.1");
bench_validate!(format_ipv4_invalid, "format ipv4 valid", {"format": "ipv4"}, "127.0.0.999");
bench_validate!(not_valid, "not valid", {"not": {"type": "null"}}, 1);
bench_validate!(not_invalid, "not invalid", {"not": {"type": "null"}}, null);
bench_validate!(min_properties_valid, "min properties valid", {"minProperties": 2}, {"a": 1, "b": 2});
bench_validate!(min_properties_invalid, "min properties invalid", {"minProperties": 2}, {"a": 1});
bench_validate!(max_properties_valid, "max properties valid", {"maxProperties": 1}, {"a": 1});
bench_validate!(max_properties_invalid, "max properties invalid", {"maxProperties": 1}, {"a": 1, "b": 2});
bench_validate!(min_items_valid, "min items valid", {"minItems": 2}, [1, 2]);
bench_validate!(min_items_invalid, "min items invalid", {"minItems": 2}, [1]);
bench_validate!(max_items_valid, "max items valid", {"maxItems": 1}, [1]);
bench_validate!(max_items_invalid, "max items invalid", {"maxItems": 1}, [1, 2]);
bench_validate!(max_length_valid, "max length valid", {"maxLength": 3}, "123");
bench_validate!(max_length_invalid, "max length invalid", {"maxLength": 3}, "1234");
bench_validate!(min_length_valid, "min length valid", {"minLength": 3}, "123");
bench_validate!(min_length_invalid, "min length invalid", {"minLength": 3}, "12");
bench_validate!(exclusive_maximum_valid, "exclusive maximum valid", {"exclusiveMaximum": 3}, 2);
bench_validate!(exclusive_maximum_invalid, "exclusive maximum invalid", {"exclusiveMaximum": 3}, 3);
bench_validate!(exclusive_minimum_valid, "exclusive minimum valid", {"exclusiveMinimum": 3}, 5);
bench_validate!(exclusive_minimum_invalid, "exclusive minimum invalid", {"exclusiveMinimum": 3}, 3);
bench_validate!(maximum_valid, "maximum valid", {"maximum": 3}, 2);
bench_validate!(maximum_invalid, "maximum invalid", {"maximum": 3}, 5);
bench_validate!(minimum_valid, "minimum valid", {"minimum": 3}, 5);
bench_validate!(minimum_invalid, "minimum invalid", {"minimum": 3}, 1);
bench_validate!(type_string_valid, "type string valid", {"type": "string"}, "1");
bench_validate!(type_string_invalid, "type string invalid", {"type": "string"}, 1);
bench_validate!(type_integer_valid1, "type integer valid 1", {"type": "integer"}, 1);
bench_validate!(type_integer_invalid1, "type integer invalid 1", {"type": "integer"}, 1.4);
bench_validate!(type_integer_valid2, "type integer valid 2", {"type": "integer"}, 1.0);
bench_validate!(type_integer_invalid2, "type integer invalid 2", {"type": "integer"}, "foo");
bench_validate!(type_multiple_valid3, "type multiple valid 3", {"type": ["integer", "string"]}, "foo");
bench_validate!(type_multiple_invalid3, "type multiple invalid 3", {"type": ["integer", "string"]}, []);
bench_validate!(unique_items_valid, "unique items valid", {"uniqueItems": true}, [1, 2, 3, 4, 5]);
bench_validate!(unique_items_invalid, "unique items invalid", {"uniqueItems": true}, [1, 2, 3, 4, 5, 1]);
bench_validate!(multiple_of_integer_valid, "multipleOf integer valid", {"multipleOf": 5}, 125);
bench_validate!(multiple_of_integer_invalid1, "multipleOf integer invalid", {"multipleOf": 5}, 212);
bench_validate!(multiple_of_integer_invalid2, "multipleOf integer invalid", {"multipleOf": 5}, 212.4);
bench_validate!(multiple_of_float_valid, "multipleOf float valid", {"multipleOf": 2.5}, 127.5);
bench_validate!(multiple_of_float_invalid, "multipleOf float invalid", {"multipleOf": 2.5}, 112.2);
bench_validate!(property_names_valid, "propertyNames valid", {"propertyNames": {"maxLength": 3}}, {"ABC": 1});
bench_validate!(property_names_invalid1, "propertyNames invalid 1", {"propertyNames": {"maxLength": 3}}, {"ABCD": 1});
bench_validate!(property_names_invalid2, "propertyNames invalid 2", {"propertyNames": false}, {"ABCD": 1});
bench_validate!(pattern_valid, "pattern valid", {"pattern": "A[0-9]{2}Z"}, "A11Z");
bench_validate!(pattern_invalid, "pattern invalid", {"pattern": "A[0-9]{2}Z"}, "A119");
bench_validate!(properties_valid, "properties valid", {"properties": {"foo": {"type": "string"}}}, {"foo": "bar"});
bench_validate!(properties_invalid, "properties invalid", {"properties": {"foo": {"type": "string"}}}, {"foo": 1});
bench_validate!(required_valid, "required valid", {"required": ["a"]}, {"a": 1});
bench_validate!(required_invalid, "required invalid", {"required": ["a"]}, {});
bench_validate!(ref_valid, "ref valid", {"items": [{"type": "integer"},{"$ref": "#/items/0"}]}, [1, 2]);
bench_compile!(c_required, "compile required", {"required": ["a", "b", "c"]});
bench_compile!(c_properties, "compile properties", {"properties": {"a": true, "b": true, "c": true}});
bench_compile!(c_dependencies, "compile dependencies", {"dependencies": {"bar": ["foo"]}});
bench_compile!(c_enum, "compile enum", {"enum": [1, 2, "3"]});
bench_compile!(c_aproperties1, "compile additional properties 1", {"additionalProperties": {"type": "boolean"}});
bench_compile!(c_aproperties2, "compile additional properties 2", {"properties": {"foo": {}, "bar": {}}, "additionalProperties": {"type": "boolean"}});
bench_compile!(c_aproperties3, "compile additional properties 3", {"properties": {"foo": {}, "bar": {}}, "patternProperties": { "^v": {} }, "additionalProperties": false});
bench_compile!(c_aproperties4, "compile additional properties 4", {"patternProperties": {"^á": {}}, "additionalProperties": false});
bench_compile!(c_aproperties5, "compile additional properties 5", {"additionalProperties": false});
bench_compile!(c_aproperties6, "compile additional properties 6", {"properties": {"foo": {}, "bar": {}}, "additionalProperties": false});

criterion_group!(
    benches,
    //    canada_benchmark,
    //    canada_benchmark_alternative,
    //    canada_compile_benchmark,
    //    fastjsonschema_compile,
    //    fastjsonschema_valid_benchmark,
    //    fastjsonschema_invalid_benchmark,
    //    type_string_valid,
    //    type_string_invalid,
    //    false_schema,
    //    additional_items_valid,
    //    additional_items_invalid,
    //    not_valid,
    //    not_invalid,
    //    enum_valid,
    //    enum_invalid,
    //    all_of_valid,
    //    all_of_invalid,
    //    any_of_valid,
    //    any_of_invalid,
    //    one_of_valid,
    //    one_of_invalid1,
    //    format_ipv4_valid,
    //    format_ipv4_invalid,
    //    contains_valid,
    //    contains_invalid,
    //    const_valid,
    //    const_invalid1,
    //    const_invalid2,
    //    min_items_valid,
    //    min_items_invalid,
    //    max_items_valid,
    //    max_items_invalid,
    //    min_properties_valid,
    //    min_properties_invalid,
    //    max_properties_valid,
    //    max_properties_invalid,
    //    max_length_valid,
    //    max_length_invalid,
    //    min_length_valid,
    //    min_length_invalid,
    //    exclusive_maximum_valid,
    //    exclusive_maximum_invalid,
    //    exclusive_minimum_valid,
    //    exclusive_minimum_invalid,
    //    maximum_valid,
    //    maximum_invalid,
    //    minimum_valid,
    //    minimum_invalid,
    //    additional_properties_valid1,
    //    additional_properties_invalid1,
    //    additional_properties_valid2,
    //    additional_properties_invalid2,
    //    additional_properties_valid3,
    //    additional_properties_invalid3,
    //    additional_properties_valid4,
    //    additional_properties_invalid4,
    //    additional_properties_valid5,
    //    additional_properties_invalid5,
    //    additional_properties_valid6,
    //    additional_properties_invalid6,
    //    type_integer_valid1,
    //    type_integer_invalid1,
    type_integer_valid2,
    type_integer_invalid2,
    type_multiple_valid3,
    type_multiple_invalid3,
    //    unique_items_valid,
    //    unique_items_invalid,
    //    multiple_of_integer_valid,
    //    multiple_of_integer_invalid1,
    //    multiple_of_integer_invalid2,
    //    multiple_of_float_valid,
    //    multiple_of_float_invalid,
    //    pattern_valid,
    //    pattern_invalid,
    //    property_names_valid,
    //    property_names_invalid1,
    //    property_names_invalid2,
    //    required_valid,
    //    required_invalid,
    //    format_time_benchmark,
    //    max_length_benchmark,
    //    ref_valid,
    //    c_required,
    //    properties_valid,
    //    properties_invalid,
    //    c_properties,
    //    c_dependencies,
    //    c_enum,
    //    c_aproperties1,
    //    c_aproperties2,
    //    c_aproperties3,
    //    c_aproperties4,
    //    c_aproperties5,
    //    c_aproperties6,
);
criterion_main!(benches);
