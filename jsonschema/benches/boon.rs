use bench_helpers::{bench_citm, bench_fast, bench_geojson, bench_openapi, bench_swagger};
use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion};

macro_rules! boon_bench {
    ($c:tt, $name:expr, $schema:ident, $instance:ident) => {{
        let mut schemas = boon::Schemas::new();
        let mut compiler = boon::Compiler::new();
        compiler.add_resource("schema.json", $schema).unwrap();
        let id = compiler.compile("schema.json", &mut schemas).unwrap();
        assert!(schemas.validate(&$instance, id).is_ok(), "Invalid instance");
        $c.bench_function(&format!("{} boon/validate", $name), |b| {
            b.iter(|| {
                let _ = schemas.validate(&$instance, id).is_ok();
            });
        });
    }};
}

fn large_schemas(c: &mut Criterion) {
    // Open API JSON Schema
    // Only `jsonschema` works correctly - other libraries do not recognize `zuora` as valid
    bench_openapi(&mut |name, schema, instance| boon_bench!(c, name, schema, instance));
    // Swagger JSON Schema
    bench_swagger(&mut |name, schema, instance| boon_bench!(c, name, schema, instance));
    // Canada borders in GeoJSON
    bench_geojson(&mut |name, schema, instance| boon_bench!(c, name, schema, instance));
    // CITM catalog
    bench_citm(&mut |name, schema, instance| boon_bench!(c, name, schema, instance));
}

fn fast_schema(c: &mut Criterion) {
    bench_fast(&mut |name, schema, valid, invalid| {
        let mut schemas = boon::Schemas::new();
        let mut compiler = boon::Compiler::new();
        compiler.add_resource("schema.json", schema).unwrap();
        let id = compiler.compile("schema.json", &mut schemas).unwrap();
        assert!(schemas.validate(&valid, id).is_ok(), "Invalid instance");
        assert!(schemas.validate(&invalid, id).is_err(), "Invalid instance");
        c.bench_function(&format!("{} boon/is_valid/valid", name), |b| {
            b.iter(|| {
                let _ = schemas.validate(&valid, id).is_ok();
            });
        });
        c.bench_function(&format!("{} boon/is_valid/invalid", name), |b| {
            b.iter(|| {
                let _ = schemas.validate(&invalid, id).is_ok();
            });
        });
    });
}

criterion_group!(arbitrary, large_schemas, fast_schema);
criterion_main!(arbitrary);
