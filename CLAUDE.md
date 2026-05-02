# Dino — Operating Notes

## Language & Toolchain

- **TypeScript** — strict mode, ES2023+ target
- **Vite** for bundling, **Vitest** for unit tests, **Playwright** for E2E
- Package manager: `pnpm` (detect via `pnpm-lock.yaml`)

## Architecture

- Game/interaction engine: component-entity pattern
- Renderer abstraction over WebGL/Canvas
- Event-driven interaction system

## Quality Gates

Run before PR:
- `pnpm lint`
- `pnpm typecheck`
- `pnpm test`
- `pnpm build`

## Git Protocol

- Conventional commits: `feat:`, `fix:`, `chore:`, `docs:`, `refactor:`
- Squash-merge to main
- No force-push to main

## Phenotype Org Policy

- Part of `phenotype-org` — see `~/.claude/CLAUDE.md` for cross-repo reuse,
  OSS-first, and Rust-default scripting hierarchy rules

