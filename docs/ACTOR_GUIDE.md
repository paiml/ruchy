# Ruchy Actor System Guide

## Overview

The Ruchy actor system provides a concurrent, message-passing model for building scalable and fault-tolerant applications. Actors are isolated units of computation that communicate exclusively through messages, eliminating shared state and race conditions.

## Current Implementation Status (v3.59.0)

### ✅ What's Working

1. **Actor Definition**: Define actors with state fields
2. **Spawn Syntax**: Create actor instances with `spawn`
3. **Message Passing**: Send messages via `.send()` method
4. **Receive Blocks**: Handle messages with pattern matching
5. **Concurrent Execution**: Actors run in separate threads
6. **Supervision Trees**: Parent-child relationships with restart strategies
7. **Lifecycle Management**: Start, stop, restart actors

### 🚧 In Progress

1. **Async Runtime**: Tokio integration for async message handling
2. **Message Operators**: `!` for send, `?` for ask pattern
3. **Complex Handlers**: Full interpreter integration for message processing
4. **Distributed Actors**: Network-based actor communication

## Basic Syntax

### Defining an Actor

```ruchy
actor Counter {
    // State fields
    count: i32 = 0

    // Message handlers
    receive Increment => {
        self.count = self.count + 1
    }

    receive GetCount => {
        self.count
    }
}
```

### Spawning an Actor

```ruchy
fn main() {
    // Spawn a new actor instance
    let counter = spawn Counter {}

    // Or with initial state
    let counter = spawn Counter { count: 10 }
}
```

### Sending Messages

```ruchy
// Send a message (fire and forget)
counter.send(Increment)

// Future: Ask pattern (send and wait for response)
// let result = counter.ask(GetCount)
// let result = counter ? GetCount  // Operator syntax
```

## Architecture

### Thread Model

Each actor runs in its own OS thread with:
- Dedicated message queue (MPSC channel)
- Event loop for message processing
- Isolated state (no shared memory)

### Message Flow

1. Messages sent to actor's mailbox
2. Actor processes messages sequentially
3. State updates are isolated within actor
4. Responses sent back to sender (if applicable)

### Supervision

Actors can supervise child actors with strategies:
- **OneForOne**: Restart only the failed child
- **AllForOne**: Restart all children when one fails
- **RestForOne**: Restart failed child and those started after it

## Examples

### Counter Actor

```ruchy
actor Counter {
    count: i32 = 0

    receive Increment => {
        self.count = self.count + 1
    }

    receive Decrement => {
        self.count = self.count - 1
    }

    receive GetCount => {
        self.count
    }
}

fn main() {
    let counter = spawn Counter {}
    counter.send(Increment)
    counter.send(Increment)
    counter.send(Decrement)
    // Count is now 1
}
```

### Ping-Pong Actors

```ruchy
actor Ping {
    pong_ref: ActorRef<Pong>

    receive Start => {
        self.pong_ref.send(Ping)
    }

    receive Pong => {
        println("Received Pong")
        self.pong_ref.send(Ping)
    }
}

actor Pong {
    ping_ref: ActorRef<Ping>

    receive Ping => {
        println("Received Ping")
        self.ping_ref.send(Pong)
    }
}
```

### Supervised Workers

```ruchy
actor Supervisor {
    workers: Vec<ActorRef<Worker>> = vec![]

    receive StartWorker => {
        let worker = spawn Worker {}
        self.workers.push(worker)
    }

    receive WorkerFailed(id: String) => {
        // Restart strategy
        println("Restarting worker: {id}")
        // Restart logic here
    }
}

actor Worker {
    tasks: i32 = 0

    receive DoWork => {
        self.tasks = self.tasks + 1
        // Simulate work
    }
}
```

## Best Practices

1. **Keep Actors Small**: Each actor should have a single responsibility
2. **Avoid Blocking**: Use async operations when possible
3. **Design for Failure**: Implement supervision strategies
4. **Message Immutability**: Messages should be immutable data
5. **Avoid Shared State**: Actors should not share mutable state

## Limitations

Current implementation limitations:

1. **No Distributed Actors**: Actors only work within a single process
2. **Limited Pattern Matching**: Simple message types only
3. **No Persistence**: Actor state is not persisted
4. **No Backpressure**: Unbounded message queues
5. **No Clustering**: No built-in cluster support

## Future Roadmap

1. **Full Async Runtime**: Complete Tokio integration
2. **Distributed Actors**: Network transparency
3. **Persistence**: Event sourcing and snapshots
4. **Advanced Patterns**: Routers, pools, streams
5. **Monitoring**: Built-in metrics and tracing

## Migration from v3.58.0

No breaking changes. New features:
- Rust 2021 edition support for async compilation
- Improved error messages for actor spawn failures
- Documentation and examples

## Resources

- [Actor Model Theory](https://en.wikipedia.org/wiki/Actor_model)
- [Erlang/OTP Design Principles](https://www.erlang.org/doc/design_principles/des_princ.html)
- [Akka Documentation](https://doc.akka.io/docs/akka/current/)