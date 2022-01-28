use criterion::Criterion;
use serde::Deserialize;
use serde_json::{from_reader, Value};
use std::{
    fs::{read_to_string, File},
    io::BufReader,
};

#[derive(Debug, Deserialize)]
struct Benchmark<'a> {
    name: &'a str,
    schema: Value,
    valid: Option<Vec<Value>>,
    invalid: Vec<Value>,
}

pub fn read_json(filepath: &str) -> Value {
    let file = File::open(filepath).expect("Failed to open file");
    let reader = BufReader::new(file);
    from_reader(reader).expect("Invalid JSON")
}

fn strip_characters(original: &str) -> String {
    original
        .chars()
        .filter(|&c| !"{}:\" ,[]".contains(c))
        .collect()
}

pub fn bench_openapi(bench: &mut dyn FnMut(&str, Value, Value)) {
    let schema = read_json("benches/data/openapi.json");
    let instance = read_json("benches/data/zuora.json");
    bench("openapi", schema, instance)
}

pub fn bench_swagger(bench: &mut dyn FnMut(&str, Value, Value)) {
    let schema = read_json("benches/data/swagger.json");
    let instance = read_json("benches/data/kubernetes.json");
    bench("swagger", schema, instance)
}

pub fn bench_geojson(bench: &mut dyn FnMut(&str, Value, Value)) {
    let schema = read_json("benches/data/geojson.json");
    let instance = read_json("benches/data/canada.json");
    bench("geojson", schema, instance)
}

pub fn bench_citm(bench: &mut dyn FnMut(&str, Value, Value)) {
    let schema = read_json("benches/data/citm_catalog_schema.json");
    let instance = read_json("benches/data/citm_catalog.json");
    bench("CITM", schema, instance)
}

pub fn bench_fast(bench: &mut dyn FnMut(&str, Value, Value, Value)) {
    let schema = read_json("benches/data/fast_schema.json");
    let valid = read_json("benches/data/fast_valid.json");
    let invalid = read_json("benches/data/fast_invalid.json");
    bench("fast", schema, valid, invalid)
}

pub fn bench_keywords(
    c: &mut Criterion,
    is_skipped: &dyn Fn(&str) -> bool,
    is_valid: &dyn Fn(&Value, &Value) -> bool,
    bench_compile: &mut dyn FnMut(&mut Criterion, &str, &Value),
    bench_valid: fn(&mut Criterion, &str, &Value, &Value),
    bench_invalid: fn(&mut Criterion, &str, &Value, &Value),
) {
    let content = read_to_string("benches/data/keywords.json").expect("Can't read file");
    let data: Vec<Benchmark> = serde_json::from_str(&content).expect("Deserialization error");
    for benchmark in data {
        if is_skipped(benchmark.name) {
            eprintln!("Skip {}", benchmark.name);
            continue;
        }
        // Schema compilation
        let suffix = strip_characters(&benchmark.schema.to_string());
        bench_compile(
            c,
            &format!("{} {}", benchmark.name, suffix),
            &benchmark.schema,
        );
        // Valid cases
        if let Some(valid_cases) = benchmark.valid {
            for instance in valid_cases {
                if !is_valid(&benchmark.schema, &instance) {
                    eprintln!("This instance should be VALID: {}", benchmark.name);
                }
                let suffix = strip_characters(&instance.to_string());
                bench_valid(
                    c,
                    &format!("{} {}", benchmark.name, suffix),
                    &benchmark.schema,
                    &instance,
                )
            }
        }
        // Invalid cases
        for instance in benchmark.invalid {
            if is_valid(&benchmark.schema, &instance) {
                eprintln!("This instance should be INVALID: {}", benchmark.name);
            }
            let suffix = strip_characters(&instance.to_string());
            bench_invalid(
                c,
                &format!("{} {}", benchmark.name, suffix),
                &benchmark.schema,
                &instance,
            )
        }
    }
}
