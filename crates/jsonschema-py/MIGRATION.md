# Migration Guide

## Upgrading from 0.19.x to 0.20.0


Draft-specific validators are now available:

```python
# Old (0.19.x)
validator = jsonschema_rs.JSONSchema(schema, draft=jsonschema_rs.Draft202012)

# New (0.20.0)
validator = jsonschema_rs.Draft202012Validator(schema)
```

Automatic draft detection:

```python
# Old (0.19.x)
validator = jsonschema_rs.JSONSchema(schema)

# New (0.20.0)
validator = jsonschema_rs.validator_for(schema)
```

