# civ

Deterministic simulation and policy-driven architecture workspace.

## Repository Structure

- `src/` core implementation
- `docs/` VitePress docs and specification corpus
- `docs/wiki/` concept and architecture knowledge
- `docs/development-guide/` contributor and implementation guides
- `docs/document-index/` generated documentation index entrypoint
- `docs/api/` API and contract documentation
- `docs/roadmap/` planning and sequencing artifacts

## Development

- `cargo test` run Rust tests
- `task quality` run local quality gate
- `cd docs && npm run docs:build` build documentation site

## Documentation Workflow

- `cd docs && npm run docs:index` regenerate `docs/.generated/doc-index.json`
- `cd docs && npm run docs:dev` preview docs locally
