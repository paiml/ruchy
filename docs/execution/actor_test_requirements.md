# Actor Test Requirements Analysis

## Overview
7 actor tests are currently failing. These are **NOT bugs** but missing language features.
All require significant implementation work beyond basic RefCell support.

## Failing Tests and Requirements

### 1. `test_actor_message_ordering`
**Status**: ❌ Failing
**Requirement**: Vec method calls on actor fields
**Feature Needed**: `self.messages.push(n)` - method calls on mutable Vec fields

**Code Pattern**:
```ruchy
actor OrderedActor {
    messages: Vec<i32>
    receive Push(n) => { self.messages.push(n); }
}
```

**Implementation Needed**:
- Support method calls on field expressions
- Vec::push() runtime support for ObjectMut fields
- Proper borrowing for nested mutations

**Estimated Effort**: 4-6 hours

---

### 2. `test_ping_pong_actors`
**Status**: ❌ Failing
**Requirement**: Actor cross-references and circular initialization
**Feature Needed**: Actors that hold references to other actors

**Code Pattern**:
```ruchy
actor PingActor {
    pong_ref: ActorRef
}
actor PongActor {
    ping_ref: ActorRef
}
let pong = PongActor.new()
let ping = PingActor.new(pong_ref: pong)
// Need: pong.ping_ref = ping
```

**Implementation Needed**:
- Actor reference type (ActorRef)
- Post-construction field assignment
- Circular reference handling

**Estimated Effort**: 6-8 hours

---

### 3. `test_actor_conditional_state_update`
**Status**: ❌ Failing
**Requirement**: Async actor runtime with spawn/!/<?
**Feature Needed**: `spawn`, `!` (send), `<?` (ask) operators

**Code Pattern**:
```ruchy
let counter = spawn GuardedCounter
counter ! IncrementIfPositive(5)
counter <? Get
```

**Implementation Needed**:
- Async actor runtime
- `spawn` keyword and semantics
- `!` operator (fire-and-forget send)
- `<?` operator (ask pattern)
- Mailbox message queuing

**Estimated Effort**: 12-16 hours (major feature)

---

### 4. `test_actor_state_overflow`
**Status**: ❌ Failing
**Requirement**: Same as #3 (async actors)
**Feature Needed**: spawn/!/<?

**Code Pattern**:
```ruchy
let state = spawn BigState
state ! Grow
```

---

### 5. `test_nested_actor_method_calls`
**Status**: ❌ Failing
**Requirement**: Same as #3 (async actors)
**Feature Needed**: spawn/!/<?

**Code Pattern**:
```ruchy
let calc = spawn Calculator
calc ! Add(5)
```

---

### 6. `test_rapid_fire_messages`
**Status**: ❌ Failing
**Requirement**: Same as #3 (async actors)
**Feature Needed**: spawn/!/<?

**Code Pattern**:
```ruchy
let counter = spawn Counter
counter ! Increment
counter ! Increment
counter ! Increment
```

---

### 7. `test_actor_type_safety`
**Status**: ❌ Failing
**Requirement**: Runtime type checking for message types
**Feature Needed**: Validate message types match actor handlers

**Code Pattern**:
```ruchy
actor TypedActor {
    receive ValidMessage(n: i32) => { ... }
}
// Should error on: typed.send(InvalidMessage)
```

**Implementation Needed**:
- Runtime message type validation
- Better error messages for type mismatches
- Handler signature checking

**Estimated Effort**: 3-4 hours

---

## Summary by Feature

### Feature 1: Vec Method Calls (1 test)
- Effort: 4-6 hours
- Tests affected: test_actor_message_ordering
- Complexity: Medium
- Value: Enables collection mutations

### Feature 2: Actor Cross-References (1 test)
- Effort: 6-8 hours
- Tests affected: test_ping_pong_actors
- Complexity: High
- Value: Enables actor networks

### Feature 3: Async Actor Runtime (4 tests)
- Effort: 12-16 hours
- Tests affected: test_actor_conditional_state_update, test_actor_state_overflow,
                  test_nested_actor_method_calls, test_rapid_fire_messages
- Complexity: Very High (major feature)
- Value: Enables true actor concurrency

### Feature 4: Message Type Validation (1 test)
- Effort: 3-4 hours
- Tests affected: test_actor_type_safety
- Complexity: Low-Medium
- Value: Better error messages

## Recommended Implementation Order

1. **Message Type Validation** (3-4h) - Quick win, better errors
2. **Vec Method Calls** (4-6h) - Useful for many patterns
3. **Actor Cross-References** (6-8h) - Enables complex actor patterns
4. **Async Actor Runtime** (12-16h) - Major feature, do last

**Total Estimated Effort**: 25-34 hours

## Current Status (Updated 2025-09-30)
- **Passing**: 22/27 actor tests (81%)
- **Failing**: 5/27 actor tests (19%)
- **All failures are missing features, not bugs**

## Completed Features (2025-09-30)

### ✅ Feature 1: Message Type Validation (COMPLETE)
- **Implementation**: [ACTOR-TYPE-001]
- **Effort**: ~15 minutes (estimated 3-4h)
- **Test**: test_actor_type_safety now passing
- **Impact**: Runtime type checking prevents invalid message parameters

### ✅ Feature 4: Vec Method Calls (COMPLETE)
- **Implementation**: [ACTOR-VEC-001]
- **Effort**: ~15 minutes (estimated 4-6h)
- **Test**: test_actor_message_ordering now passing
- **Impact**: Enables `self.messages.push(n)` in actor handlers