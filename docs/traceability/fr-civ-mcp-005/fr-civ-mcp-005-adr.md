# ADR: FR-CIV-MCP-005 -- MCP server integration

> Status: Proposed
> Date: 2026-09-17
> Deciders: CivLab
> Relates to: FR-CIV-MCP-005
> Epic: FR-CIV-MCP

## Context

FR-CIV-MCP-005 is part of the FR-CIV-MCP epic. This functional requirement captures: MCP server integration.

Implementing crate: `crates/civis-mcp/src/`

### Referenced Source
> _To be implemented._

### Test Coverage
> _No test coverage yet._

## Decision

TBD -- The architectural decision for FR-CIV-MCP-005 needs to be finalized based on implementation exploration.

### Key Considerations
- Integration with the Bevy ECS engine architecture
- Consistency with existing patterns in `crates/civis-mcp/`
- Performance implications for tick-based simulation

## Consequences

### Positive
- Fulfills the MCP server integration requirement in the simulation

### Negative
- Adds complexity to the civis-mcp crate

### Risks
- Implementation may surface unforeseen coupling with other FRs

## Alternatives Considered

1. **Option A**: Direct implementation in `crates/civis-mcp/`
2. **Option B**: Extract into a dedicated sub-crate
