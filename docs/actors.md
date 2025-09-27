# Ruchy Actor System Documentation

## Status: Basic Actor Support Available (v3.48.0+)

The Ruchy actor system provides basic actor definition, instantiation, and state access. This is suitable for introductory course usage with message passing coming in the next release.

## Current Working Features ✅

### 1. Actor Definition
Define actors with state fields and message handlers:

```ruchy
// Basic actor with state
actor Counter {
    count: i32
}

// Actor with multiple state fields
actor Person {
    name: String,
    age: i32,
    active: bool
}

// Actor with state block (alternative syntax)
actor User {
    state {
        id: String,
        balance: f64
    }
}

// Actor with message handlers
actor PingActor {
    count: i32
    receive {
        Ping(n) => { self.count = n; },
        Pong => { self.count = self.count + 1; }
    }
}

// Actor with individual message handler
actor SimpleActor {
    count: i32
    receive Ping(n) => { self.count = n; }
}
```

### 2. Actor Instantiation
Create actor instances using the `.new()` method:

```ruchy
// Define an actor type
actor Counter { count: i32 }

// Create an instance
let instance = Counter.new()
```

The instance will have:
- All state fields initialized to `nil`
- Actor type metadata stored internally
- Access to state via property syntax

### 3. State Access
Access actor state fields using dot notation:

```ruchy
actor Counter { count: i32 }
let instance = Counter.new()

// Access state (returns nil initially)
let count_value = instance.count
println(count_value)  // prints: nil
```

## Planned Features (Coming Soon) 🚧

### Message Passing (Next Release)
```ruchy
// Send messages to actors
actor PingActor {
    count: i32
    receive Ping(n) => { self.count = n; }
}

let ping = PingActor.new()
ping.send(Ping(42))  // Will be supported soon

// Ask pattern for responses
let result = ping.ask(GetCount)  // Will be supported soon
```

### Parameterized Constructors (Next Release)
```ruchy
// Initialize with specific values
let counter = Counter.new(count: 5)  // Will be supported soon
```

### Actor Lifecycle (Next Release)
```ruchy
// Spawn actors in their own execution context
let actor = spawn Counter(count: 0)  // Will be supported soon

// Stop actors gracefully
actor.stop()  // Will be supported soon
```

## Course Usage Examples

### Example 1: Basic Actor Setup
```ruchy
// Define a simple counter actor
actor Counter {
    count: i32,
    name: String
}

// Create an instance
let my_counter = Counter.new()

// Check initial state
println("Count:", my_counter.count)  // prints: Count: nil
println("Name:", my_counter.name)    // prints: Name: nil
```

### Example 2: Actor with Message Handlers (Definition Only)
```ruchy
// Define an actor that will handle messages
actor Calculator {
    result: f64

    receive {
        Add(x) => { self.result = self.result + x; },
        Multiply(x) => { self.result = self.result * x; },
        Clear => { self.result = 0.0; }
    }
}

// Create the actor
let calc = Calculator.new()
println("Calculator created:", calc.result)  // prints: Calculator created: nil

// Message sending will be available in next release
```

### Example 3: Multiple Actors
```ruchy
// Define different actor types
actor Player {
    name: String,
    score: i32,
    level: i32
}

actor Game {
    players: Vec<Player>,
    status: String
}

// Create instances
let player1 = Player.new()
let player2 = Player.new()
let game = Game.new()

// Access their state
println("Game status:", game.status)        // prints: Game status: nil
println("Player1 score:", player1.score)   // prints: Player1 score: nil
```

## Implementation Notes

The current implementation provides:
- ✅ Full actor definition parsing and validation
- ✅ Actor type registration in the interpreter environment
- ✅ Basic actor instantiation with `.new()` method
- ✅ State field access via dot notation
- ✅ Support for all Ruchy data types as state fields
- ✅ Message handler definition (syntax validated)

This foundation enables teaching:
- Object-oriented concepts with actors
- State encapsulation
- Type definitions and instances
- Preparation for message-passing concepts

## Error Handling

Current error reporting includes:
- Parse errors for invalid actor syntax
- Type validation for actor definitions
- Method not found errors for unsupported operations
- Clear error messages for debugging

## Next Steps

The next release will add:
1. Message passing with `.send()` and `.ask()` methods
2. Parameterized constructors with `new(field: value)` syntax
3. Actor lifecycle management with `spawn` and `.stop()`
4. Actor-to-actor communication patterns
5. Example implementations of common actor patterns

This will complete the actor system for full course usage including the ping-pong demo requested in the original specification.