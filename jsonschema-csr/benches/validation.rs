use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jsonschema_csr::JsonSchema;
use serde_json::json;

fn bench_is_valid(c: &mut Criterion) {
    c.bench_function("is_valid maximum", |b| {
        let schema = json!({"maximum": 5});
        let instance = black_box(json!(4));
        let compiled = JsonSchema::new(&schema).unwrap();
        b.iter(|| compiled.is_valid(&instance))
    });
    c.bench_function("is_valid properties", |b| {
        let schema = json!({"properties": {"A": {"maximum": 5}}});
        let instance = black_box(json!({"A": 4}));
        let compiled = JsonSchema::new(&schema).unwrap();
        b.iter(|| compiled.is_valid(&instance))
    });
}

criterion_group!(benches, bench_is_valid);
criterion_main!(benches);
