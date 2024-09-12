use benchmark::Benchmark;
use codspeed_criterion_compat::{criterion_group, criterion_main, BenchmarkId, Criterion};
use jsonschema::JSONSchema;
use serde_json::Value;

fn bench_compile(c: &mut Criterion, name: &str, schema: &Value) {
    c.bench_function(&format!("{}/compile", name), |b| {
        b.iter(|| JSONSchema::compile(schema).expect("Valid schema"))
    });
}

fn bench_is_valid(c: &mut Criterion, name: &str, schema: &Value, instance: &Value) {
    let compiled = JSONSchema::compile(schema).expect("Valid schema");
    c.bench_with_input(
        BenchmarkId::new(name, "is_valid"),
        instance,
        |b, instance| {
            b.iter(|| {
                let _ = compiled.is_valid(instance);
            })
        },
    );
}

fn bench_validate(c: &mut Criterion, name: &str, schema: &Value, instance: &Value) {
    let compiled = JSONSchema::compile(schema).expect("Valid schema");
    c.bench_with_input(
        BenchmarkId::new(name, "validate"),
        instance,
        |b, instance| {
            b.iter(|| {
                let _ = compiled.validate(instance);
            })
        },
    );
}

fn run_benchmarks(c: &mut Criterion) {
    for benchmark in Benchmark::iter() {
        benchmark.run(&mut |name, schema, instances| {
            bench_compile(c, name, schema);
            for instance in instances {
                let name = format!("{}/{}", name, instance.name);
                bench_is_valid(c, &name, schema, &instance.data);
                bench_validate(c, &name, schema, &instance.data);
            }
        });
    }
}

criterion_group!(jsonschema, run_benchmarks);
criterion_main!(jsonschema);
