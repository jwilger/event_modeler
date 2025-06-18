# Schema Versioning Strategy

**Date**: 2025-06-18

## Status

Accepted

## Context

Event Modeler is adopting a YAML-based format for event models. As this format will evolve over time, we need a clear strategy for versioning the schema to ensure compatibility and clear communication of changes.

Key considerations:
- We are currently pre-1.0 (version 0.3.0)
- The format will likely evolve significantly before stabilization
- Users need to know which version of Event Modeler their files are compatible with
- We need to plan for future compatibility requirements

## Decision

We will tie the .eventmodel schema version directly to the Event Modeler application version.

### Version Field

The YAML format will include an optional `version` field:
```yaml
version: 0.3.0  # Optional, defaults to current app version
workflow: User Account Signup
# ... rest of the file
```

### Semantic Versioning Rules

Following semantic versioning principles:
- **Major version change**: Breaking changes to schema (removing fields, changing types)
- **Minor version change**: Backward-compatible additions (new optional fields, new entity types)
- **Patch version change**: No schema changes, only implementation fixes

### Pre-1.0 Strategy

While we're pre-1.0:
- No backward compatibility guarantees
- Clean breaks are acceptable and expected
- Focus on finding the right design

### Post-1.0 Strategy

After 1.0 release:
- Parser accepts schema versions with same major version
- Warn on minor version differences (newer features might not render)
- Error on major version differences (incompatible schema)
- Provide migration tools for major version changes

### Implementation Details

1. Store version constant in code matching Cargo.toml version
2. Parse version field first to determine parsing strategy
3. Default to current version if not specified
4. Clear error messages for version mismatches

## Consequences

### Positive

- Clear relationship between app and schema versions
- Users always know which version to use
- Simple mental model
- Natural evolution path
- Enables future migration strategies

### Negative

- Schema changes require app version bumps
- Can't update schema independently of app
- Users must update app for new schema features

### Mitigations

- Clear documentation of schema changes in release notes
- Version compatibility matrix in documentation
- Future: schema-only releases if needed

## Alternatives Considered

1. **Independent Schema Versioning**: Separate version numbers for schema and app
   - Rejected: Adds complexity without clear benefit
   - Would require maintaining compatibility matrix

2. **Date-based Versioning**: Use dates like 2025.06.18
   - Rejected: Doesn't communicate breaking vs non-breaking changes
   - Less familiar to developers

3. **No Versioning**: Always use latest schema
   - Rejected: Makes it impossible to maintain compatibility
   - Poor user experience when files stop working