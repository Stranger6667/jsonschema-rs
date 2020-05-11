use draft::test_draft;
use jsonschema::JSONSchema;
use serde_json::Value;

test_draft!("tests/suite/tests/draft6/");
test_draft!("tests/suite/tests/draft7/");
