use bench_helpers::read_json;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jsonschema_csr::JsonSchema;
use serde_json::json;

fn bench_is_valid(c: &mut Criterion) {
    let schema = json!({"maximum": 5});
    let instance = black_box(json!(4));
    let compiled = JsonSchema::new(&schema).unwrap();
    c.bench_function("is_valid maximum", |b| {
        b.iter(|| compiled.is_valid(&instance))
    });
}

criterion_group!(benches, bench_is_valid);
criterion_main!(benches);
