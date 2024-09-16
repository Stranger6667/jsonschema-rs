#![no_main]
use jsonschema::BasicOutput;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: (&[u8], &[u8])| {
    let (schema, instance) = data;
    if let Ok(schema) = serde_json::from_slice(schema) {
        if let Ok(compiled) = jsonschema::compile(&schema) {
            if let Ok(instance) = serde_json::from_slice(instance) {
                let _ = compiled.is_valid(&instance);
                if let Err(errors) = compiled.validate(&instance) {
                    for error in errors {
                        let _ = error.to_string();
                    }
                }
                let output: BasicOutput = compiled.apply(&instance).basic();
                let _ = serde_json::to_value(output).expect("Failed to serialize");
            }
        }
    }
});
