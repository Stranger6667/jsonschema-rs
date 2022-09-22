use criterion::{black_box, criterion_group, criterion_main, Criterion};
use jsonschema_csr::JsonSchema;
use serde_json::{json, Value};

fn run_bench(c: &mut Criterion, name: &str, schema: Value, instance: &Value) {
    let compiled = JsonSchema::new(&schema).unwrap();
    let instance = black_box(instance);
    c.bench_function(name, |b| b.iter(|| compiled.is_valid(&instance)));
}

fn bench_is_valid(c: &mut Criterion) {
    run_bench(c, "is_valid maximum", json!({"maximum": 5}), &json!(4));
    run_bench(
        c,
        "is_valid properties",
        json!({"properties": {"A": {"maximum": 5}}}),
        &json!({"A": 4}),
    );
    run_bench(
        c,
        "is_valid many properties",
        json!({
            "properties": {
                "1": {"maximum": 5},
                "2": {"maximum": 5},
                "3": {"maximum": 5},
                "4": {"maximum": 5},
                "5": {"maximum": 5},
                "6": {"maximum": 5},
                "7": {"maximum": 5},
                "8": {"maximum": 5},
                "9": {"maximum": 5},
                "10": {"maximum": 5},
            }
        }),
        &json!({
            "1": 4,
            "2": 4,
            "3": 4,
            "4": 4,
            "5": 4,
            "6": 4,
            "7": 4,
            "8": 4,
            "9": 4,
            "10": 4
        }),
    );
    run_bench(
        c,
        "is_valid ref",
        json!({
            "properties": {
                "1": {"$ref": "#/definitions/1"},
            },
            "definitions": {
                "1": {
                    "maximum": 5
                }
            }
        }),
        &json!({"1": 4}),
    );
}

criterion_group!(benches, bench_is_valid);
criterion_main!(benches);
