use bench_helpers::{bench_citm, bench_fast, bench_geojson, bench_keywords, bench_swagger};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use serde_json::Value;
use valico::json_schema;

macro_rules! valico_bench {
    ($c:tt, $name:expr, $schema:ident, $instance:ident) => {{
        let mut scope = json_schema::Scope::new();
        let compiled = scope
            .compile_and_return($schema.clone(), false)
            .expect("Valid schema");
        assert!(compiled.validate(&$instance).is_valid(), "Invalid instance");
        $c.bench_function(&format!("valico {}", $name), |b| {
            // There is no specialized method for fast boolean return value
            b.iter(|| compiled.validate(&$instance).is_valid())
        });
    }};
}

fn large_schemas(c: &mut Criterion) {
    // Swagger JSON Schema
    bench_swagger(&mut |name, schema, instance| valico_bench!(c, name, schema, instance));
    // Canada borders in GeoJSON
    bench_geojson(&mut |name, schema, instance| valico_bench!(c, name, schema, instance));
    // CITM catalog
    bench_citm(&mut |name, schema, instance| valico_bench!(c, name, schema, instance));
}

fn fast_schema(c: &mut Criterion) {
    bench_fast(&mut |name, schema, valid, invalid| {
        let mut scope = json_schema::Scope::new();
        let compiled = scope
            .compile_and_return(schema.clone(), false)
            .expect("Valid schema");
        c.bench_function(&format!("{} valico/compile", name), |b| {
            b.iter(|| {
                let mut scope = json_schema::Scope::new();
                scope
                    .compile_and_return(schema.clone(), false)
                    .expect("Valid schema");
            })
        });
        c.bench_function(&format!("{} valico/validate/valid", name), |b| {
            b.iter(|| compiled.validate(&valid).is_valid())
        });
        c.bench_function(&format!("{} valico/validate/invalid", name), |b| {
            b.iter(|| compiled.validate(&invalid).is_valid())
        });
    });
}

fn keywords(c: &mut Criterion) {
    bench_keywords(
        c,
        &|_: &str| false,
        &|schema: &Value, instance: &Value| {
            let mut scope = json_schema::Scope::new();
            let compiled = scope
                .compile_and_return(schema.clone(), false)
                .expect("Valid schema");
            compiled.validate(instance).is_valid()
        },
        &mut |c: &mut Criterion, name: &str, schema: &Value| {
            c.bench_with_input(
                BenchmarkId::new(name, "valico/compile"),
                schema,
                |b, schema| {
                    b.iter_with_setup(
                        || schema.clone(),
                        |schema| {
                            let mut scope = json_schema::Scope::new();
                            scope
                                .compile_and_return(schema, false)
                                .expect("Valid schema");
                        },
                    )
                },
            );
        },
        validate,
        validate,
    )
}

fn validate(c: &mut Criterion, name: &str, schema: &Value, instance: &Value) {
    let mut scope = json_schema::Scope::new();
    let compiled = scope
        .compile_and_return(schema.clone(), false)
        .expect("Valid schema");
    c.bench_with_input(BenchmarkId::new(name, "valico"), instance, |b, instance| {
        b.iter(|| {
            compiled.validate(instance).is_valid();
        })
    });
}

criterion_group!(valico, large_schemas, fast_schema, keywords);
criterion_main!(valico);
