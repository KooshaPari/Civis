# API Stability Contract

## Public API Surface

The SDK public API surface is stable within minor versions.

## Versioning

- Breaking changes require major version bump and changelog notice
- All public types must have XML doc comments
- Deprecated members must have `[Obsolete]` attributes with guidance
- Public method signatures should remain unchanged across minor versions

## Review Process

- API changes in PRs must be flagged explicitly
- SDK public surface changes require docs update
- Consider backwards compatibility for all public API modifications
