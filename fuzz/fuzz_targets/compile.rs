#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(schema) = serde_json::from_slice(data) else {
        return;
    };
    let _ = jsonschema::compile(&schema);
});
