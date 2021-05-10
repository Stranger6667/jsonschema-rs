use std::{error::Error, fs, path::PathBuf, process};

use jsonschema::JSONSchema;
use structopt::StructOpt;

type BoxErrorResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, StructOpt)]
#[structopt(name = "jsonschema")]
struct Cli {
    /// A path to a JSON instance (i.e. filename.json) to validate (may be specified multiple times).
    #[structopt(short = "i", long = "instance")]
    instances: Option<Vec<PathBuf>>,

    /// The JSON Schema to validate with (i.e. schema.json).
    #[structopt(parse(from_os_str), required_unless("version"))]
    schema: Option<PathBuf>,

    /// Show program's version number and exit.
    #[structopt(short = "v", long = "version")]
    version: bool,
}

pub fn main() -> BoxErrorResult<()> {
    let config = Cli::from_args();

    if config.version {
        println!("Version: {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    let mut success = true;
    if let Some(schema) = config.schema {
        if let Some(instances) = config.instances {
            success = validate_instances(&instances, schema)?;
        }
    }

    if !success {
        process::exit(1);
    }

    Ok(())
}

fn validate_instances(instances: &[PathBuf], schema: PathBuf) -> BoxErrorResult<bool> {
    let mut success = true;

    let schema_json = fs::read_to_string(schema)?;
    let schema_json = serde_json::from_str(&schema_json)?;
    match JSONSchema::compile(&schema_json) {
        Ok(schema) => {
            for instance in instances {
                let instance_path_name = instance.to_str().unwrap();
                let instance_json = fs::read_to_string(&instance)?;
                let instance_json = serde_json::from_str(&instance_json)?;
                let validation = schema.validate(&instance_json);
                match validation {
                    Ok(_) => println!("{} - VALID", instance_path_name),
                    Err(errors) => {
                        success = false;

                        println!("{} - INVALID. Errors:", instance_path_name);
                        for (i, e) in errors.enumerate() {
                            println!("{}. {}", i + 1, e);
                        }
                    }
                }
            }
        }
        Err(error) => {
            println!("Schema is invalid. Error: {}", error);
            success = false;
        }
    }
    Ok(success)
}
