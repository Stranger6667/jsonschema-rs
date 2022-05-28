use bench_helpers::read_json;
use criterion::{criterion_group, criterion_main, Criterion};
use jsonschema_csr::{scope_of, Resolver};

fn bench_build(c: &mut Criterion) {
    for name in [
        "openapi",
        "swagger",
        "geojson",
        "citm_catalog_schema",
        "fast_schema",
    ] {
        let schema = read_json(&format!("../jsonschema/benches/data/{}.json", name));
        let scope = scope_of(&schema);
        c.bench_function(&format!("build {}", name), |b| {
            b.iter(|| Resolver::new(&schema, scope.clone()))
        });
    }
}

fn bench_resolve(c: &mut Criterion) {
    for (name, reference) in [
        ("openapi", "#/definitions/HTTPSecurityScheme/properties/type/enum/0"),
        ("swagger", "#/definitions/pathParameterSubSchema/properties/description/description"),
        ("geojson", "#/properties/features/items/properties/geometry/oneOf/7/properties/geometries/items/oneOf/5/properties/coordinates/items/items/items/items/type"),
        ("citm_catalog_schema", "#/properties/performances/items/properties/seatCategories/items/properties/areas/items/properties/blockIds/items/type"),
        ("fast_schema", "#/items/3/properties/c/type/1"),
    ] {
        let schema = read_json(&format!("../jsonschema/benches/data/{}.json", name));
        let resolver = Resolver::new(&schema, scope_of(&schema));
        assert!(resolver.resolve(reference).is_some());
        c.bench_function(&format!("resolve {}", name), |b| {
            b.iter(|| resolver.resolve(reference))
        });
    }
}

criterion_group!(benches, bench_build, bench_resolve);
criterion_main!(benches);
