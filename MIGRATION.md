# Migration Guide

## Upgrading from 0.19.x to 0.20.0

### New Features

1. Draft-specific modules are now available:

   ```rust
   // Old (0.19.x)
   let validator = jsonschema::JSONSchema::options()
       .with_draft(jsonschema::Draft2012)
       .compile(&schema)
       .expect("Invalid schema");

   // New (0.20.0)
   let validator = jsonschema::draft202012::new(&schema)
       .expect("Invalid schema");
   ```

   Available modules: `draft4`, `draft6`, `draft7`, `draft201909`, `draft202012`

2. Use the new `options()` function for easier customization:

   ```rust
   // Old (0.19.x)
   let options = jsonschema::JSONSchema::options();

   // New (0.20.0)
   let options = jsonschema::options();
   ```

### Deprecations and Renames

The following items have been renamed. While the old names are still supported in 0.20.0 for backward compatibility, it's recommended to update to the new names:

| Old Name (0.19.x) | New Name (0.20.0) |
|-------------------|-------------------|
| `CompilationOptions` | `ValidationOptions` |
| `JSONSchema` | `Validator` |
| `JSONPointer` | `JsonPointer` |
| `jsonschema::compile` | `jsonschema::validator_for` |
| `CompilationOptions::compile` | `ValidationOptions::build` |

