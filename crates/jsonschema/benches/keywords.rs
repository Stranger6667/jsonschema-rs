use benchmark::run_keyword_benchmarks;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use jsonschema::JSONSchema;
use serde_json::Value;

fn bench_keyword_compile(c: &mut Criterion, name: &str, schema: &Value) {
    c.bench_function(&format!("keyword/{}/compile", name), |b| {
        b.iter(|| JSONSchema::compile(schema).expect("Valid schema"))
    });
}

fn bench_keyword_is_valid(c: &mut Criterion, name: &str, schema: &Value, instance: &Value) {
    let compiled = JSONSchema::compile(schema).expect("Valid schema");
    c.bench_with_input(
        BenchmarkId::new(format!("keyword/{}", name), "is_valid"),
        instance,
        |b, instance| {
            b.iter(|| {
                let _ = compiled.is_valid(instance);
            })
        },
    );
}

fn bench_keyword_validate(c: &mut Criterion, name: &str, schema: &Value, instance: &Value) {
    let compiled = JSONSchema::compile(schema).expect("Valid schema");
    c.bench_with_input(
        BenchmarkId::new(format!("keyword/{}", name), "validate"),
        instance,
        |b, instance| {
            b.iter(|| {
                let _ = compiled.validate(instance);
            })
        },
    );
}

fn run_benchmarks(c: &mut Criterion) {
    run_keyword_benchmarks(&mut |name, schema, instances| {
        bench_keyword_compile(c, name, schema);
        for instance in instances {
            let name = format!("jsonschema/{}/{}", name, instance.name);
            bench_keyword_is_valid(c, &name, schema, &instance.data);
            bench_keyword_validate(c, &name, schema, &instance.data);
        }
    });
}

criterion_group!(keywords, run_benchmarks);
criterion_main!(keywords);
