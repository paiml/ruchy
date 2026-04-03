# Ruchy Scoring Specification

**Document Status**: Specification Index (split into sub-specs for maintainability)

---

## Executive Summary

The `ruchy score` command provides a single, deterministic quality metric (0.0-1.0) for Ruchy code. This unified score enables objective quality gates, MCP integration, and continuous improvement tracking while maintaining computational efficiency through incremental analysis.

## Critical Design Constraints

### Scope Management
The specification acknowledges the "astronomical scope" risk identified in review. Mitigation strategy:
- **Phase 0 enforcement**: No scoring features beyond base metrics until parser/type system complete
- **Tool priority**: Debugger first (4 months), then incremental tool delivery
- **Feature freeze**: Score algorithm locked at v1.0 to prevent scope creep

### Resource Reality
Given the "decades not years" implementation horizon:
- **Incremental value delivery**: Each tool provides standalone utility
- **Progressive enhancement**: Tools function at reduced fidelity initially
- **Expertise scaling**: Core team implements frameworks; community implements analyzers

---

## Sub-spec Index

| Sub-spec | File | Sections | Description |
|----------|------|----------|-------------|
| MCP Integration and Core Architecture | [scoring-mcp-architecture.md](sub/scoring-mcp-architecture.md) | 1-5 | MCP Server Integration via PMCP, MCP Server Integration, Executive Summary, Critical Design Constraints, Core Architecture |
| Metrics, CLI, and Extended Toolchain | [scoring-metrics-toolchain.md](sub/scoring-metrics-toolchain.md) | 6-12 | Metric Definitions, Addressing Implementation Challenges, CLI Interface, Grade Boundaries, Integration Points, Success Metrics, Future Enhancements, Extended Toolchain Integration (items 1-4) |
| Unified Score Architecture and Interactive Debugger | [scoring-unified-debugger.md](sub/scoring-unified-debugger.md) | 13-17 | Interactive Prover, Unified Score Architecture, Implementation Timeline, Success Metrics, Ruchy Interactive Debugger (`ridb`) |

---

## Conclusion

The debugger integrates with all other tools:

- **Score**: Debuggability metrics feed maintainability score
- **Observatory**: Share actor telemetry infrastructure
- **Dataflow Debugger**: Unified breakpoint system
- **Prover**: Import counterexamples as debug scenarios
- **Mechanical Sympathy**: Show assembly alongside source

This completes the developer experience toolchain, providing the essential capability of understanding program behavior through controlled execution.
