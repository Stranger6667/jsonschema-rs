#[cfg(not(feature = "cli"))]
fn main() {
    eprintln!("`jsonschema` CLI is only available with the `cli` feature");
    std::process::exit(1);
}

#[cfg(feature = "cli")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::{
        fs::File,
        io::BufReader,
        path::{Path, PathBuf},
        process,
    };

    use clap::Parser;
    use jsonschema::JSONSchema;

    #[derive(Parser)]
    #[command(name = "jsonschema")]
    struct Cli {
        /// A path to a JSON instance (i.e. filename.json) to validate (may be specified multiple times).
        #[arg(short = 'i', long = "instance")]
        instances: Option<Vec<PathBuf>>,

        /// The JSON Schema to validate with (i.e. schema.json).
        #[arg(value_parser, required_unless_present("version"))]
        schema: Option<PathBuf>,

        /// Show program's version number and exit.
        #[arg(short = 'v', long = "version")]
        version: bool,
    }

    fn read_json(path: &Path) -> serde_json::Result<serde_json::Value> {
        let file = File::open(path).expect("Failed to open file");
        let reader = BufReader::new(file);
        serde_json::from_reader(reader)
    }

    fn validate_instances(
        instances: &[PathBuf],
        schema_path: PathBuf,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let mut success = true;

        let schema_json = read_json(&schema_path)?;
        match JSONSchema::compile(&schema_json) {
            Ok(schema) => {
                for instance in instances {
                    let instance_json = read_json(instance)?;
                    let validation = schema.validate(&instance_json);
                    let filename = instance.to_string_lossy();
                    match validation {
                        Ok(_) => println!("{} - VALID", filename),
                        Err(errors) => {
                            success = false;

                            println!("{} - INVALID. Errors:", filename);
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
    let config = Cli::parse();

    if config.version {
        println!(concat!("Version: ", env!("CARGO_PKG_VERSION")));
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
