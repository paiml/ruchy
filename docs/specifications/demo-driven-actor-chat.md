# Demo-Driven Actor Chat Specification
## Ruchy Actor Concurrency for Agentic AI

### Executive Summary

This specification drives the next phase of Ruchy development through a concrete chatbot demo that showcases:
- **Actor-based concurrency** with supervision trees
- **MCP integration** for LLM communication
- **EXTREME-TDD** with 100% test-first development
- **Production quality** from day one

The demo features a multi-agent chat system where specialized AI agents (Architect, SecurityAuditor, TestEngineer, Refactorer) discuss topics autonomously under a supervised actor hierarchy.

## Sub-spec Index

| Sub-spec | Scope | Link |
|----------|-------|------|
| Demo and EXTREME-TDD Specification | Demo code, language features, TDD phases 1-5 (parser, types, transpiler, integration, property tests) | [sub/actor-chat-demo-tdd.md](sub/actor-chat-demo-tdd.md) |
| Quality, Implementation Phases, and Appendices | Coverage/performance/complexity requirements, week-by-week plan, acceptance criteria, MCP integration, supervision patterns | [sub/actor-chat-quality-phases.md](sub/actor-chat-quality-phases.md) |

## Key Language Features

- **`spawn Actor::new()`** - Creates unsupervised actor
- **`spawn_supervised Actor::new()`** - Creates actor with implicit supervisor
- **`supervisor.spawn_child(Actor::new())`** - Creates actor under explicit supervisor
- **`actor ! message`** - Fire-and-forget send
- **`actor ? message`** - Request-reply (ask)
- **`actor !> op() |> handler`** - Pipeline operator for async message flows

## Acceptance Criteria Summary

### Functional
- Parse all actor syntax correctly
- Type check message passing
- Generate valid Rust code with Tokio runtime
- Supervision tree restarts failed actors
- Chat demo runs for 1 hour without crashes

### Quality
- 95% test coverage, 90% mutation score
- Performance meets p99 targets (spawn <100us, message <1us)
- Zero clippy warnings, zero SATD comments
