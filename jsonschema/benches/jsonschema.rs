use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jsonschema::JSONSchema;
use jsonschema_valid::schemas;
use serde_json::{from_str, json, Value};
use std::{fs::File, io::Read, path::Path};
use valico::json_schema;

fn read_json(filepath: &str) -> Value {
    let path = Path::new(filepath);
    let mut file = File::open(&path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).ok().unwrap();
    let data: Value = from_str(&content).unwrap();
    data
}

fn strip_characters(original: &str) -> String {
    original
        .chars()
        .filter(|&c| !"{}:\" ,[]".contains(c))
        .collect()
}

macro_rules! bench {
    (
      name = $name:tt;
      schema = $schema:tt;
      valid = $( $valid:tt ),* $(,)*;
      invalid = $( $invalid:tt ),* $(,)*;
    ) => {
        paste::item! {
          #[allow(dead_code)]
          fn [<bench_ $name>](c: &mut Criterion) {
              let schema = json!($schema);
              let validator = JSONSchema::compile(&schema).unwrap();
              let suffix = strip_characters(stringify!($schema));
              c.bench_function(format!("jsonschema-rs {} compile {}", $name, suffix).as_str(), |b| b.iter(|| JSONSchema::compile(&schema).unwrap()));
              $(
                   let instance = black_box(json!($valid));
                   assert!(validator.is_valid(&instance));
                   let suffix = strip_characters(stringify!($valid));
                   c.bench_function(format!("jsonschema-rs {} is_valid valid {}", $name, suffix).as_str(), |b| b.iter(|| validator.is_valid(&instance)));
                   c.bench_function(format!("jsonschema-rs {} validate valid {}", $name, suffix).as_str(), |b| b.iter(|| validator.validate(&instance).ok()));
              )*
              $(
                   let instance = black_box(json!($invalid));
                   assert!(!validator.is_valid(&instance));
                   let suffix = strip_characters(stringify!($invalid));
                   c.bench_function(format!("jsonschema-rs {} is_valid invalid {}", $name, suffix).as_str(), |b| b.iter(|| validator.is_valid(&instance)));
                   c.bench_function(format!("jsonschema-rs {} validate invalid {}", $name, suffix).as_str(), |b| b.iter(|| {
                        let _: Vec<_> = validator.validate(&instance).unwrap_err().collect();
                   }));
              )*
          }
        }
    };
    (
      name = $name:tt;
      schema = $schema:tt;
      invalid = $( $invalid:tt ),* $(,)*;
    ) => {
        paste::item! {
          fn [<bench_ $name>](c: &mut Criterion) {
              let schema = json!($schema);
              let validator = JSONSchema::compile(&schema).unwrap();
              let suffix = strip_characters(stringify!($schema));
              c.bench_function(format!("jsonschema-rs {} compile {}", $name, suffix).as_str(), |b| b.iter(|| JSONSchema::compile(&schema).unwrap()));
              $(
                   let instance = black_box(json!($invalid));
                   assert!(!validator.is_valid(&instance));
                   let suffix = strip_characters(stringify!($invalid));
                   c.bench_function(format!("jsonschema-rs {} is_valid invalid {}", $name, suffix).as_str(), |b| b.iter(|| validator.is_valid(&instance)));
                   c.bench_function(format!("jsonschema-rs {} validate invalid {}", $name, suffix).as_str(), |b| b.iter(|| {
                        let _: Vec<_> = validator.validate(&instance).unwrap_err().collect();
                   }));
              )*
          }
        }
    };
}

macro_rules! jsonschema_rs_bench {
    ($c:tt, $name:expr, $schema:ident, $instance:ident) => {{
        let validator = JSONSchema::options()
            .with_meta_schemas()
            .compile(&$schema)
            .expect("Invalid schema");
        assert!(validator.is_valid(&$instance), "Invalid instance");
        assert!(validator.validate(&$instance).is_ok(), "Invalid instance");
        $c.bench_function(&format!("jsonschema-rs {} compile", $name), |b| {
            b.iter(|| JSONSchema::options().with_meta_schemas().compile(&$schema))
        });
        $c.bench_function(&format!("jsonschema-rs {} is_valid", $name), |b| {
            b.iter(|| validator.is_valid(&$instance))
        });
        $c.bench_function(&format!("jsonschema-rs {} validate", $name), |b| {
            b.iter(|| validator.validate(&$instance).ok())
        });
    }};
}
macro_rules! jsonschema_valid_bench {
    ($c:tt, $name:expr, $schema:ident, $instance:ident, $draft:expr) => {{
        let cfg =
            jsonschema_valid::Config::from_schema(&$schema, Some($draft)).expect("Invalid schema");
        assert!(
            jsonschema_valid::validate(&cfg, &$instance).is_ok(),
            "Invalid instance"
        );
        $c.bench_function(&format!("jsonschema-valid {}", $name), |b| {
            // There is no specialized method for fast boolean return value
            b.iter(|| jsonschema_valid::validate(&cfg, &$instance).is_ok())
        });
    }};
}
macro_rules! valico_bench {
    ($c:tt, $name:expr, $schema:ident, $instance:ident) => {{
        let mut scope = json_schema::Scope::new();
        let valico_validator = scope.compile_and_return($schema.clone(), false).unwrap();
        assert!(
            valico_validator.validate(&$instance).is_valid(),
            "Invalid instance"
        );
        $c.bench_function(&format!("valico {}", $name), |b| {
            // There is no specialized method for fast boolean return value
            b.iter(|| valico_validator.validate(&$instance).is_valid())
        });
    }};
}

#[allow(dead_code)]
fn large_schemas(c: &mut Criterion) {
    // Open API JSON Schema
    // Only `jsonschema` works correctly - other libraries do not recognize `zuora` as valid
    let openapi = read_json("benches/openapi.json");
    let zuora = read_json("benches/zuora.json");
    jsonschema_rs_bench!(c, "openapi", openapi, zuora);

    // Swagger JSON Schema
    let swagger = read_json("benches/swagger.json");
    let kubernetes = read_json("benches/kubernetes.json");
    jsonschema_rs_bench!(c, "swagger", swagger, kubernetes);
    valico_bench!(c, "swagger", swagger, kubernetes);

    // Canada borders in GeoJSON
    let geojson = read_json("benches/geojson.json");
    let canada = read_json("benches/canada.json");
    jsonschema_rs_bench!(c, "canada", geojson, canada);
    jsonschema_valid_bench!(c, "canada", geojson, canada, schemas::Draft::Draft7);
    valico_bench!(c, "canada", geojson, canada);

    // CITM catalog
    let citm_catalog_schema = read_json("benches/citm_catalog_schema.json");
    let citm_catalog = read_json("benches/citm_catalog.json");
    jsonschema_rs_bench!(c, "citm_catalog", citm_catalog_schema, citm_catalog);
    jsonschema_valid_bench!(
        c,
        "citm_catalog",
        citm_catalog_schema,
        citm_catalog,
        schemas::Draft::Draft7
    );
    valico_bench!(c, "citm_catalog", citm_catalog_schema, citm_catalog);
}

#[allow(dead_code)]
fn fast_schema(c: &mut Criterion) {
    let schema = read_json("benches/fast_schema.json");
    let valid =
        black_box(json!([9, "hello", [1, "a", true], {"a": "a", "b": "b", "d": "d"}, 42, 3]));
    let invalid =
        black_box(json!([10, "world", [1, "a", true], {"a": "a", "b": "b", "c": "xy"}, "str", 5]));

    // jsonschema
    let validator = JSONSchema::compile(&schema).unwrap();
    assert!(validator.is_valid(&valid));
    assert!(!validator.is_valid(&invalid));
    c.bench_function("compare jsonschema-rs small schema compile", |b| {
        b.iter(|| JSONSchema::compile(&schema).unwrap())
    });
    c.bench_function("compare jsonschema-rs small schema is_valid valid", |b| {
        b.iter(|| validator.is_valid(&valid))
    });
    c.bench_function("compare jsonschema-rs small schema validate valid", |b| {
        b.iter(|| validator.validate(&valid).ok())
    });
    c.bench_function("compare jsonschema-rs small schema is_valid invalid", |b| {
        b.iter(|| validator.is_valid(&invalid))
    });
    c.bench_function("compare jsonschema-rs small schema validate invalid", |b| {
        b.iter(|| {
            let _: Vec<_> = validator.validate(&invalid).unwrap_err().collect();
        })
    });

    // jsonschema_valid
    let cfg = jsonschema_valid::Config::from_schema(&schema, Some(schemas::Draft::Draft7)).unwrap();
    c.bench_function("compare jsonschema_valid small schema compile", |b| {
        b.iter(|| {
            jsonschema_valid::Config::from_schema(&schema, Some(schemas::Draft::Draft7)).unwrap()
        })
    });
    c.bench_function(
        "compare jsonschema_valid small schema validate valid",
        |b| b.iter(|| jsonschema_valid::validate(&cfg, &valid)),
    );
    c.bench_function(
        "compare jsonschema_valid small schema validate invalid",
        |b| b.iter(|| jsonschema_valid::validate(&cfg, &invalid).ok()),
    );

    // valico
    let mut scope = json_schema::Scope::new();
    let compiled = scope.compile_and_return(schema.clone(), false).unwrap();
    c.bench_function("compare valico small schema compile", |b| {
        b.iter(|| {
            let mut scope = json_schema::Scope::new();
            scope.compile_and_return(schema.clone(), false).unwrap();
        })
    });
    c.bench_function("compare valico small schema validate valid", |b| {
        b.iter(|| compiled.validate(&valid).is_valid())
    });
    c.bench_function("compare valico small schema validate invalid", |b| {
        b.iter(|| compiled.validate(&invalid).is_valid())
    });
}

bench!(
  name = "additional_items_boolean";
  schema = {"items": [{}, {}, {}], "additionalItems": false};
  valid = [1, 2, 3];
  invalid = [1, 2, 3, 4];
);
bench!(
  name = "additional_items_object";
  schema = {"items": [{}, {}, {}], "additionalItems": {"type": "string"}};
  valid = [1, 2, 3, "foo"];
  invalid = [1, 2, 3, 4];
);
bench!(
  name = "additional_properties_single";
  schema = {"additionalProperties": {"type": "string"}};
  valid = {"foo": "bar"};
  invalid = {"foo": 1};
);
bench!(
  name = "additional_properties_and_properties";
  schema = {"additionalProperties": {"type": "string"}, "properties": {"foo": {}}};
  valid = {"foo": 1};
  invalid = {"foo": 1, "bar": true};
);
bench!(
  name = "additional_properties_and_pattern_properties";
  schema = {"additionalProperties": {"type": "string"}, "patternProperties": {"f.*o": {"type": "integer"}}};
  valid = {"foo": 1};
  invalid = {"foo": 1, "bar": true};
);
bench!(
  name = "additional_properties_and_properties_and_pattern_properties";
  schema = {"additionalProperties": {"type": "string"}, "properties": {"foo": {}}, "patternProperties": {"f.*a": {"type": "integer"}}};
  valid = {"foo": null, "fza": 2};
  invalid = {"foo": null, "fzo": 2, "bar": true};
);
bench!(
  name = "additional_properties_false";
  schema = {"additionalProperties": false};
  valid = {};
  invalid = {"foo": 1};
);
bench!(
  name = "additional_properties_false_and_properties";
  schema = {"additionalProperties": false, "properties": {"foo": {}}};
  valid = {"foo": 1};
  invalid = {"foo": 1, "bar": 2};
);
bench!(
  name = "additional_properties_false_and_pattern_properties";
  schema = {"additionalProperties": false, "patternProperties": {"f.*o": {"type": "integer"}}};
  valid = {"foo": 1};
  invalid = {"foo": 1, "bar": 2};
);
bench!(
  name = "additional_properties_false_and_properties_and_pattern_properties";
  schema = {"additionalProperties": false, "properties": {"foo": {}}, "patternProperties": {"f.*o": {"type": "integer"}}};
  valid = {"foo": 1};
  invalid = {"foo": 1, "fz0": 2, "bar": 2};
);
bench!(
  name = "all_of";
  schema = {"allOf": [{"type": "integer"}, {"minimum": 2}]};
  valid = 4;
  invalid = 1;
);
bench!(
  name = "any_of";
  schema = {"anyOf": [{"type": "integer"}, {"minimum": 2}]};
  valid = 1;
  invalid = 1.5;
);
bench!(
  name = "any_of_multiple_types";
  schema = {"anyOf": [{"type": "integer"}, {"type": "string"}]};
  valid = "foo";
  invalid = null;
);
bench!(
  name = "boolean_false";
  schema = false;
  invalid = 1;
);
bench!(
  name = "const";
  schema = {"const": 1};
  valid = 1;
  invalid = "foo";
);
bench!(
  name = "contains";
  schema = {"contains": {"minimum": 5}};
  valid = [5];
  invalid = [1];
);
bench!(
  name = "enum";
  schema = {"enum": [1, 2, 3, 4]};
  valid = 4;
  invalid = 5, "6";
);
bench!(
  name = "exclusive_maximum";
  schema = {"exclusiveMaximum": 3};
  valid = 2;
  invalid = 3;
);
bench!(
  name = "exclusive_minimum";
  schema = {"exclusiveMinimum": 3};
  valid = 4;
  invalid = 3;
);
bench!(
  name = "format_date";
  schema = {"format": "date"};
  valid = "1963-06-19";
  invalid = "06/19/1963";
);
bench!(
  name = "format_datetime";
  schema = {"format": "date-time"};
  valid = "1963-06-19T08:30:06.283185Z";
  invalid = "1990-02-31T15:59:60.123-08:00";
);
bench!(
  name = "format_email";
  schema = {"format": "email"};
  valid = "test@test.com";
  invalid = "foo";
);
bench!(
  name = "format_hostname";
  schema = {"format": "hostname"};
  valid = "www.example.com";
  invalid = "not_a_valid_host_name";
);
bench!(
  name = "format_ipv4";
  schema = {"format": "ipv4"};
  valid = "127.0.0.1";
  invalid = "127.0.0.999", "foobar", "2001:0db8:85a3:0000:0000:8a2e:0370:7334";
);
bench!(
  name = "format_ipv6";
  schema = {"format": "ipv6"};
  valid = "2001:0db8:85a3:0000:0000:8a2e:0370:7334";
  invalid = "127.0.0.1", "foobar";
);
bench!(
  name = "format_iri";
  schema = {"format": "iri"};
  valid = "http://ƒøø.ßår/?∂éœ=πîx#πîüx";
  invalid = "/abc";
);
bench!(
  name = "format_iri_reference";
  schema = {"format": "iri-reference"};
  valid = "http://ƒøø.ßår/?∂éœ=πîx#πîüx";
  invalid = "#ƒräg\\mênt";
);
bench!(
  name = "format_json_pointer";
  schema = {"format": "json-pointer"};
  valid = "/foo/bar~0/baz~1/%a";
  invalid = "/foo/bar~";
);
bench!(
  name = "format_regex";
  schema = {"format": "regex"};
  valid = r#"([abc])+\s+$"#;
  invalid = "^(abc]";
);
bench!(
  name = "format_relative_json_pointer";
  schema = {"format": "relative-json-pointer"};
  valid = "1";
  invalid = "/foo/bar";
);
bench!(
  name = "format_time";
  schema = {"format": "time"};
  valid = "08:30:06.283185Z";
  invalid = "01:01:01,1111";
);
bench!(
  name = "format_uri_reference";
  schema = {"format": "uri-reference"};
  valid = "http://foo.bar/?baz=qux#quux";
  invalid = "#frag\\ment";
);
bench!(
  name = "format_uri_template";
  schema = {"format": "uri-template"};
  valid = "http://example.com/dictionary/{term:1}/{term}";
  invalid = "http://example.com/dictionary/{term:1}/{term";
);
bench!(
  name = "items";
  schema = {"items": {"type": "integer"}};
  valid = [1, 2, 3];
  invalid = [1, 2, "x"];
);
bench!(
  name = "maximum";
  schema = {"maximum": 3};
  valid = 3;
  invalid = 5;
);
bench!(
  name = "max_items";
  schema = {"maxItems": 1};
  valid = [1];
  invalid = [1, 2];
);
bench!(
  name = "max_length";
  schema = {"maxLength": 3};
  valid = "foo";
  invalid = "foob";
);
bench!(
  name = "max_properties";
  schema = {"maxProperties": 1};
  valid = {"a": 1};
  invalid = {"a": 1, "b": 1};
);
bench!(
  name = "minimum";
  schema = {"minimum": 3};
  valid = 5;
  invalid = 1;
);
bench!(
  name = "min_items";
  schema = {"minItems": 2};
  valid = [1, 2];
  invalid = [1];
);
bench!(
  name = "min_length";
  schema = {"minLength": 3};
  valid = "123";
  invalid = "12";
);
bench!(
  name = "min_properties";
  schema = {"minProperties": 2};
  valid = {"a": 1, "b": 2};
  invalid = {"a": 1};
);
bench!(
  name = "multiple_of_integer";
  schema = {"multipleOf": 5};
  valid = 125;
  invalid = 212, 212.4;
);
bench!(
  name = "multiple_of_number";
  schema = {"multipleOf": 2.5};
  valid = 127.5;
  invalid = 112.2;
);
bench!(
  name = "not";
  schema = {"not": {"type": "null"}};
  valid = 1;
  invalid = null;
);
bench!(
  name = "one_of";
  schema = {"oneOf": [{"type": "integer"}, {"minimum": 2}]};
  valid = 1;
  invalid = 3;
);
bench!(
  name = "pattern";
  schema = {"pattern": "A[0-9]{2}Z"};
  valid = "A11Z";
  invalid = "A119";
);
bench!(
  name = "pattern_properties";
  schema = {"patternProperties": {"f.*o": {"type": "integer"}}};
  valid = {"foo": 1};
  invalid = {"foo": "bar", "fooooo": 2};
);
bench!(
  name = "properties";
  schema = {"properties": {"foo": {"type": "string"}}};
  valid = {"foo": "bar"};
  invalid = {"foo": 1};
);
bench!(
  name = "property_names";
  schema = {"propertyNames": {"maxLength": 3}};
  valid = {"ABC": 1};
  invalid = {"ABCD": 1};
);
bench!(
  name = "ref";
  schema = {"items": [{"type": "integer"},{"$ref": "#/items/0"}]};
  valid = [1, 2];
  invalid = [1, "b"];
);
bench!(
  name = "required";
  schema = {"required": ["a"]};
  valid = {"a": 1};
  invalid = {};
);
bench!(
  name = "type_integer";
  schema = {"type": "integer"};
  valid = 1, 1.0;
  invalid = 1.4, "foo";
);
bench!(
  name = "type_string";
  schema = {"type": "string"};
  valid = "foo";
  invalid = 1;
);
bench!(
  name = "type_multiple";
  schema = {"type": ["integer", "string"]};
  valid = "foo";
  invalid = [];
);
bench!(
  name = "unique_items";
  schema = {"uniqueItems": true};
  valid = [1, "2", [3], {"4": 4}, 5, "6", [7], {"8": 8}, 9, "10", [11], {"12": 12}];
  invalid = [1, 2, 3, 4, 5, 1];
);

criterion_group!(
    keywords,
    bench_additional_items_boolean,
    bench_additional_items_object,
    bench_additional_properties_single,
    bench_additional_properties_and_properties,
    bench_additional_properties_and_pattern_properties,
    bench_additional_properties_and_properties_and_pattern_properties,
    bench_additional_properties_false,
    bench_additional_properties_false_and_properties,
    bench_additional_properties_false_and_pattern_properties,
    bench_additional_properties_false_and_properties_and_pattern_properties,
    bench_all_of,
    bench_any_of,
    bench_any_of_multiple_types,
    bench_boolean_false,
    bench_const,
    bench_contains,
    bench_enum,
    bench_exclusive_maximum,
    bench_exclusive_minimum,
    bench_format_date,
    bench_format_datetime,
    bench_format_email,
    bench_format_hostname,
    bench_format_ipv4,
    bench_format_ipv6,
    bench_format_iri,
    bench_format_iri_reference,
    bench_format_json_pointer,
    bench_format_regex,
    bench_format_relative_json_pointer,
    bench_format_time,
    bench_format_uri_reference,
    bench_format_uri_template,
    bench_items,
    bench_maximum,
    bench_max_items,
    bench_max_length,
    bench_max_properties,
    bench_minimum,
    bench_min_items,
    bench_min_length,
    bench_min_properties,
    bench_multiple_of_integer,
    bench_multiple_of_number,
    bench_not,
    bench_one_of,
    bench_pattern,
    bench_pattern_properties,
    bench_properties,
    bench_property_names,
    bench_ref,
    bench_required,
    bench_type_integer,
    bench_type_string,
    bench_type_multiple,
    bench_unique_items,
);
criterion_group!(arbitrary, large_schemas, fast_schema);
criterion_main!(arbitrary, keywords);
