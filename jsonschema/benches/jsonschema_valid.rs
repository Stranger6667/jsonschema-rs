use bench_helpers::{bench_citm, bench_fast, bench_geojson, bench_keywords};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use jsonschema_valid::schemas;
use serde_json::Value;

macro_rules! jsonschema_valid_bench {
    ($c:tt, $name:expr, $schema:ident, $instance:ident, $draft:expr) => {{
        let cfg =
            jsonschema_valid::Config::from_schema(&$schema, Some($draft)).expect("Invalid schema");
        assert!(
            jsonschema_valid::validate(&cfg, &$instance).is_ok(),
            "Invalid instance"
        );
        $c.bench_function(&format!("{} jsonschema_valid/validate/valid", $name), |b| {
            // There is no specialized method for fast boolean return value
            b.iter(|| jsonschema_valid::validate(&cfg, &$instance).is_ok())
        });
    }};
}

fn large_schemas(c: &mut Criterion) {
    // Canada borders in GeoJSON
    bench_geojson(&mut |name, schema, instance| {
        jsonschema_valid_bench!(c, name, schema, instance, schemas::Draft::Draft7)
    });
    // CITM catalog
    bench_citm(&mut |name, schema, instance| {
        jsonschema_valid_bench!(c, name, schema, instance, schemas::Draft::Draft7)
    });
}

fn fast_schema(c: &mut Criterion) {
    bench_fast(&mut |name, schema, valid, invalid| {
        let cfg = jsonschema_valid::Config::from_schema(&schema, Some(schemas::Draft::Draft7))
            .expect("Valid schema");
        c.bench_function(&format!("{} jsonschema_valid/compile", name), |b| {
            b.iter(|| {
                jsonschema_valid::Config::from_schema(&schema, Some(schemas::Draft::Draft7))
                    .expect("Valid schema")
            })
        });
        c.bench_function(&format!("{} jsonschema_valid/validate/valid", name), |b| {
            b.iter(|| jsonschema_valid::validate(&cfg, &valid))
        });
        c.bench_function(
            &format!("{} jsonschema_valid/validate/invalid", name),
            |b| b.iter(|| jsonschema_valid::validate(&cfg, &invalid).ok()),
        );
    });
}

fn keywords(c: &mut Criterion) {
    bench_keywords(
        c,
        &|name: &str| {
            // Bug in `jsonschema_valid`
            // `Option::unwrap()` on a `None` value'
            // https://github.com/mdboom/jsonschema-valid/blob/de1da64fb624085eccde290e036a2ed592656f38/src/validators.rs#L531
            name == "multiple_of_integer"
        },
        &|schema: &Value, instance: &Value| {
            let compiled =
                jsonschema_valid::Config::from_schema(schema, Some(schemas::Draft::Draft7))
                    .expect("Valid schema");
            let result = jsonschema_valid::validate(&compiled, instance).is_ok();
            result
        },
        &mut |c: &mut Criterion, name: &str, schema: &Value| {
            c.bench_with_input(
                BenchmarkId::new(name, "jsonschema_valid/compile"),
                schema,
                |b, schema| {
                    b.iter(|| {
                        jsonschema_valid::Config::from_schema(schema, Some(schemas::Draft::Draft7))
                            .expect("Valid schema")
                    })
                },
            );
        },
        validate,
        validate,
    )
}

fn validate(c: &mut Criterion, name: &str, schema: &Value, instance: &Value) {
    let compiled = jsonschema_valid::Config::from_schema(schema, Some(schemas::Draft::Draft7))
        .expect("Valid schema");
    c.bench_with_input(
        BenchmarkId::new(name, "jsonschema_valid"),
        &(compiled, instance),
        |b, (compiled, instance)| {
            b.iter(|| {
                let _ = jsonschema_valid::validate(compiled, instance);
            })
        },
    );
}

criterion_group!(jsonschema_valid, large_schemas, fast_schema, keywords);
criterion_main!(jsonschema_valid);
