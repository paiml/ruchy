# Ruchy Class and Struct Specification (Swift Model)
Version 2.0.0

## Abstract

This specification defines the exact Swift model for structs (value types) and classes (reference types) in Ruchy, providing clear semantics that developers already understand while transpiling to idiomatic Rust code.

## Core Principle

**Structs are value types. Classes are reference types.** This fundamental distinction, borrowed directly from Swift, drives all design decisions.

## Sub-spec Index

| Sub-spec | Scope | Link |
|----------|-------|------|
| Core Semantics and Transpilation Rules | Syntax grammar, value/reference semantics, initialization, methods, inheritance, traits, transpilation to Rust | [sub/class-sugar-semantics-transpilation.md](sub/class-sugar-semantics-transpilation.md) |
| Advanced Features, Migration, and Examples | Properties, operator overloading, generics, restrictions, compatibility mode, error messages, migration from Python, shape hierarchy example, open questions | [sub/class-sugar-advanced-migration.md](sub/class-sugar-advanced-migration.md) |

## Motivation

- Adopt proven semantics from Swift that millions of developers understand
- Clear mental model: value vs reference semantics
- Performance by default with structs (stack allocation, no ARC)
- Flexibility when needed with classes (inheritance, shared state)
- Zero surprises for developers coming from Swift/modern languages

## Transpilation Summary

| Ruchy Construct | Generated Rust |
|----------------|----------------|
| `struct Point { x: f64 }` | `#[derive(Clone, Copy, Debug)] struct Point { pub x: f64 }` |
| `class Person { ... }` | `struct PersonData { ... }` + `struct Person(Rc<RefCell<PersonData>>)` |
| `class Dog : Animal { ... }` | `trait Animal` + `struct Dog { _base: AnimalBase, ... }` |
| `mutating fun` | `fn(&mut self, ...)` |
| `init(...)` | `fn new(...) -> Self` |

## Performance Guarantees

- Zero vtable overhead unless `dyn` trait objects used
- Methods inline by default
- No hidden allocations
- Field access compiles to direct memory access
- Inheritance via composition has no runtime cost

---

*This specification is subject to change during implementation based on technical constraints and user feedback.*
