use bench_helpers::{
    bench_citm, bench_fast, bench_geojson, bench_keywords, bench_openapi, bench_swagger,
};
use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
use jsonschema::{paths::JsonPointerNode, JSONSchema};
use serde_json::Value;

macro_rules! jsonschema_rs_bench {
    ($c:tt, $name:expr, $schema:ident, $instance:ident) => {{
        let compiled = JSONSchema::options()
            .with_meta_schemas()
            .compile(&$schema)
            .expect("Invalid schema");
        assert!(compiled.is_valid(&$instance), "Invalid instance");
        assert!(compiled.validate(&$instance).is_ok(), "Invalid instance");
        $c.bench_function(&format!("{} jsonschema_rs/compile", $name), |b| {
            b.iter(|| JSONSchema::options().with_meta_schemas().compile(&$schema))
        });
        $c.bench_function(&format!("{} jsonschema_rs/is_valid", $name), |b| {
            b.iter(|| compiled.is_valid(&$instance))
        });
        $c.bench_function(&format!("{} jsonschema_rs/validate", $name), |b| {
            b.iter(|| compiled.validate(&$instance).ok())
        });
    }};
}

fn large_schemas(c: &mut Criterion) {
    // Open API JSON Schema
    // Only `jsonschema` works correctly - other libraries do not recognize `zuora` as valid
    bench_openapi(&mut |name, schema, instance| jsonschema_rs_bench!(c, name, schema, instance));
    // Swagger JSON Schema
    bench_swagger(&mut |name, schema, instance| jsonschema_rs_bench!(c, name, schema, instance));
    // Canada borders in GeoJSON
    bench_geojson(&mut |name, schema, instance| jsonschema_rs_bench!(c, name, schema, instance));
    // CITM catalog
    bench_citm(&mut |name, schema, instance| jsonschema_rs_bench!(c, name, schema, instance));
}

fn fast_schema(c: &mut Criterion) {
    bench_fast(&mut |name, schema, valid, invalid| {
        let compiled = JSONSchema::compile(&schema).expect("Valid schema");
        assert!(compiled.is_valid(&valid));
        assert!(!compiled.is_valid(&invalid));
        c.bench_function(&format!("{} jsonschema_rs/compile", name), |b| {
            b.iter(|| JSONSchema::compile(&schema).expect("Valid schema"))
        });
        c.bench_function(&format!("{} jsonschema_rs/is_valid/valid", name), |b| {
            b.iter(|| compiled.is_valid(&valid))
        });
        c.bench_function(&format!("{} jsonschema_rs/validate/valid", name), |b| {
            b.iter(|| compiled.validate(&valid).ok())
        });
        c.bench_function(&format!("{} jsonschema_rs/is_valid/invalid", name), |b| {
            b.iter(|| compiled.is_valid(&invalid))
        });
        c.bench_function(&format!("{} jsonschema_rs/validate/invalid", name), |b| {
            b.iter(|| {
                let _: Vec<_> = compiled
                    .validate(&invalid)
                    .expect_err("There should be errors")
                    .collect();
            })
        });
    });
}

fn keywords(c: &mut Criterion) {
    bench_keywords(
        c,
        &|_: &str| false,
        &|schema: &Value, instance: &Value| {
            let compiled = JSONSchema::compile(schema).expect("Valid schema");
            compiled.is_valid(instance)
        },
        &mut |c: &mut Criterion, name: &str, schema: &Value| {
            c.bench_with_input(
                BenchmarkId::new(name, "jsonschema_rs/compile"),
                schema,
                |b, schema| {
                    b.iter(|| {
                        JSONSchema::compile(schema).expect("Valid schema");
                    })
                },
            );
        },
        validate_valid,
        validate_invalid,
    )
}

fn validate_valid(c: &mut Criterion, name: &str, schema: &Value, instance: &Value) {
    let compiled = JSONSchema::compile(schema).expect("Valid schema");
    c.bench_with_input(
        BenchmarkId::new(name, "jsonschema_rs/is_valid/valid"),
        instance,
        |b, instance| {
            b.iter(|| {
                let _ = compiled.is_valid(instance);
            })
        },
    );
    c.bench_with_input(
        BenchmarkId::new(name, "jsonschema_rs/validate/valid"),
        instance,
        |b, instance| {
            b.iter(|| {
                compiled.validate(instance).ok();
            })
        },
    );
}

fn validate_invalid(c: &mut Criterion, name: &str, schema: &Value, instance: &Value) {
    let compiled = JSONSchema::compile(schema).expect("Valid schema");
    c.bench_with_input(
        BenchmarkId::new(name, "jsonschema_rs/is_valid/invalid"),
        instance,
        |b, instance| {
            b.iter(|| {
                let _ = compiled.is_valid(instance);
            })
        },
    );
    c.bench_with_input(
        BenchmarkId::new(name, "jsonschema_rs/validate/invalid"),
        instance,
        |b, instance| {
            b.iter(|| {
                let _: Vec<_> = compiled
                    .validate(instance)
                    .expect_err("There should be errors")
                    .collect();
            })
        },
    );
}

fn json_pointer_node(c: &mut Criterion) {
    fn bench(b: &mut Bencher, pointer: &JsonPointerNode) {
        b.iter(|| {
            let _ = pointer.to_vec();
        })
    }
    let empty = JsonPointerNode::new();
    c.bench_with_input(BenchmarkId::new("jsonpointer", "empty"), &empty, bench);
    let root = JsonPointerNode::new();
    let node = root.push("entry");
    let node = node.push("entry");
    let node = node.push("entry");
    c.bench_with_input(BenchmarkId::new("jsonpointer", "small"), &node, bench);
    let root = JsonPointerNode::new();
    let node = root.push("entry");
    let node = node.push("entry");
    let node = node.push("entry");
    let node = node.push("entry");
    let node = node.push("entry");
    let node = node.push("entry");
    let node = node.push("entry");
    let node = node.push("entry");
    let node = node.push("entry");
    c.bench_with_input(BenchmarkId::new("jsonpointer", "big"), &node, bench);
}

criterion_group!(
    arbitrary,
    large_schemas,
    fast_schema,
    keywords,
    json_pointer_node
);
criterion_main!(arbitrary);
