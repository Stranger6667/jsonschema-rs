//! Logic for retrieving external resources.
use referencing::{Retrieve, Uri};
use serde_json::Value;

pub(crate) struct DefaultRetriever;

impl Retrieve for DefaultRetriever {
    #[allow(unused)]
    fn retrieve(&self, uri: &Uri<&str>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_arch = "wasm32")]
        {
            Err("External references are not supported in WASM".into())
        }
        #[cfg(not(target_arch = "wasm32"))]
        match uri.scheme().as_str() {
            "http" | "https" => {
                #[cfg(any(feature = "resolve-http", test))]
                {
                    Ok(reqwest::blocking::get(uri.as_str())?.json()?)
                }
                #[cfg(not(any(feature = "resolve-http", test)))]
                Err("`resolve-http` feature or a custom resolver is required to resolve external schemas via HTTP".into())
            }
            "file" => {
                #[cfg(any(feature = "resolve-file", test))]
                {
                    let path = uri.path().as_str();
                    let path = {
                        #[cfg(windows)]
                        {
                            // Remove the leading slash and replace forward slashes with backslashes
                            let path = path.trim_start_matches('/').replace('/', "\\");
                            std::path::PathBuf::from(path)
                        }
                        #[cfg(not(windows))]
                        {
                            std::path::PathBuf::from(path)
                        }
                    };
                    let file = std::fs::File::open(path)?;
                    Ok(serde_json::from_reader(file)?)
                }
                #[cfg(not(any(feature = "resolve-file", test)))]
                {
                    Err("`resolve-file` feature or a custom resolver is required to resolve external schemas via files".into())
                }
            }
            scheme => Err(format!("Unknown scheme {scheme}").into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    #[cfg(not(target_arch = "wasm32"))]
    use std::io::Write;

    #[cfg(not(target_arch = "wasm32"))]
    fn path_to_uri(path: &std::path::Path) -> String {
        use percent_encoding::{percent_encode, AsciiSet, CONTROLS};

        let mut result = "file://".to_owned();
        const SEGMENT: &AsciiSet = &CONTROLS
            .add(b' ')
            .add(b'"')
            .add(b'<')
            .add(b'>')
            .add(b'`')
            .add(b'#')
            .add(b'?')
            .add(b'{')
            .add(b'}')
            .add(b'/')
            .add(b'%');

        #[cfg(not(target_os = "windows"))]
        {
            use std::os::unix::ffi::OsStrExt;

            const CUSTOM_SEGMENT: &AsciiSet = &SEGMENT.add(b'\\');
            for component in path.components().skip(1) {
                result.push('/');
                result.extend(percent_encode(
                    component.as_os_str().as_bytes(),
                    CUSTOM_SEGMENT,
                ));
            }
        }
        #[cfg(target_os = "windows")]
        {
            use std::path::{Component, Prefix};
            let mut components = path.components();

            match components.next() {
                Some(Component::Prefix(ref p)) => match p.kind() {
                    Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => {
                        result.push('/');
                        result.push(letter as char);
                        result.push(':');
                    }
                    _ => panic!("Unexpected path"),
                },
                _ => panic!("Unexpected path"),
            }

            for component in components {
                if component == Component::RootDir {
                    continue;
                }

                let component = component.as_os_str().to_str().expect("Unexpected path");

                result.push('/');
                result.extend(percent_encode(component.as_bytes(), SEGMENT));
            }
        }
        result
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_retrieve_from_file() {
        let mut temp_file = tempfile::NamedTempFile::new().expect("Failed to create temp file");
        let external_schema = json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" }
            },
            "required": ["name"]
        });
        write!(temp_file, "{}", external_schema).expect("Failed to write to temp file");

        let uri = path_to_uri(temp_file.path());

        let schema = json!({
            "type": "object",
            "properties": {
                "user": { "$ref": uri }
            }
        });

        let validator = crate::validator_for(&schema).expect("Schema compilation failed");

        let valid = json!({"user": {"name": "John Doe"}});
        assert!(validator.is_valid(&valid));

        let invalid = json!({"user": {}});
        assert!(!validator.is_valid(&invalid));
    }

    #[test]
    fn test_unknown_scheme() {
        let schema = json!({
            "type": "object",
            "properties": {
                "test": { "$ref": "unknown-schema://test" }
            }
        });

        let result = crate::validator_for(&schema);

        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        #[cfg(not(target_arch = "wasm32"))]
        assert!(error.contains("Unknown scheme"));
        #[cfg(target_arch = "wasm32")]
        assert!(error.contains("External references are not supported in WASM"));
    }
}
