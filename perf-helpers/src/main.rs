use jsonschema::JSONSchema;
use serde_json::{from_str, Error, Value};

/// This executable is supposed to be used via ../run sh scrpt
/// The script does trigger the build of the binary and is responsible
/// injects for injecting the needed environmental variables
fn main() -> Result<(), Error> {
    let schema: Value = from_str(include_str!(env!("SCHEMA_PATH")))?;
    let instance: Value = from_str(include_str!(env!("INSTANCE_PATH")))?;
    let number_of_iterations: usize = env!("NUMBER_OF_ITERATIONS")
        .parse()
        .expect("NUMBER_OF_ITERATIONS is expected to be a positive integer");

    eprintln!("Schema {}", schema);
    eprintln!("Instance {}", instance);
    eprintln!("Number of Iterations {}", number_of_iterations);

    let compiled = JSONSchema::compile(&schema).unwrap();
    for _ in 0..number_of_iterations {
        compiled.is_valid(&instance);
    }

    Ok(())
}
