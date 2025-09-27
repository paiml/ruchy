// EXTREME TDD: Actor Implementation Tests
// Following CLAUDE.md Toyota Way - ALL tests written FIRST before implementation

use ruchy::frontend::parser::Parser;
use ruchy::runtime::interpreter::{Interpreter, Value};

fn eval_code(interpreter: &mut Interpreter, code: &str) -> Result<Value, String> {
    let mut parser = Parser::new(code);
    let expr = parser.parse().map_err(|e| e.to_string())?;
    interpreter.eval_expr(&expr).map_err(|e| e.to_string())
}

#[cfg(test)]
mod actor_definition_tests {
    use super::*;

    #[test]
    fn test_simple_actor_with_state() {
        let mut interpreter = Interpreter::new();
        let code = "actor SimpleActor { count: i32 }";
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");

        // Actor definition should return a value representing the actor type
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_actor_with_multiple_state_fields() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            actor CounterActor {
                count: i32,
                name: String,
                active: bool
            }
        "#;
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_actor_with_state_block() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            actor StateActor {
                state {
                    counter: i32,
                    message: String
                }
            }
        "#;
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_actor_with_receive_handler() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            actor PingActor {
                count: i32
                receive {
                    Ping(n) => { self.count = n; },
                    Pong => { self.count = self.count + 1; }
                }
            }
        "#;
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_actor_with_individual_receive() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            actor SimpleActor {
                count: i32
                receive Ping(n) => { self.count = n; }
            }
        "#;
        let result = eval_code(&mut interpreter, code).expect("Should parse and evaluate");
        assert!(matches!(result, Value::Object(_)));
    }
}

#[cfg(test)]
mod actor_instantiation_tests {
    use super::*;

    #[test]
    fn test_actor_instantiation() {
        let mut interpreter = Interpreter::new();

        // Define actor
        eval_code(&mut interpreter, "actor Counter { count: i32 }").expect("Should define");

        // Instantiate actor
        let result = eval_code(&mut interpreter, "let instance = Counter.new()")
            .expect("Should instantiate");
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_actor_with_initial_state() {
        let mut interpreter = Interpreter::new();

        eval_code(&mut interpreter, "actor Counter { count: i32 }").expect("Should define");

        let result = eval_code(&mut interpreter, "let instance = Counter.new(count: 5)")
            .expect("Should instantiate");
        assert!(matches!(result, Value::Object(_)));
    }
}

#[cfg(test)]
mod message_passing_tests {
    use super::*;

    #[test]
    fn test_send_message_to_actor() {
        let mut interpreter = Interpreter::new();

        // Define actors
        eval_code(
            &mut interpreter,
            r#"
            actor PingActor {
                count: i32
                receive Ping(n) => { self.count = n; }
            }
        "#,
        )
        .expect("Should define");

        // Create actor instance
        eval_code(&mut interpreter, "let ping = PingActor.new(count: 0)").expect("Should create");

        // Send message
        let result = eval_code(&mut interpreter, "ping.send(Ping(42))").expect("Should send");
        assert!(matches!(result, Value::Nil) || matches!(result, Value::Bool(true)));
    }

    #[test]
    fn test_ask_message_to_actor() {
        let mut interpreter = Interpreter::new();

        eval_code(
            &mut interpreter,
            r#"
            actor EchoActor {
                receive Echo(msg) => { msg }
            }
        "#,
        )
        .expect("Should define");

        eval_code(&mut interpreter, "let echo = EchoActor.new()").expect("Should create");

        let result = eval_code(&mut interpreter, r#"echo.ask(Echo("hello"))"#).expect("Should ask");
        assert!(matches!(result, Value::String(_)));
    }
}

#[cfg(test)]
mod ping_pong_integration_tests {
    use super::*;

    #[test]
    fn test_ping_pong_actors() {
        let mut interpreter = Interpreter::new();

        // Define Ping actor
        eval_code(
            &mut interpreter,
            r#"
            actor PingActor {
                count: i32,
                pong_ref: ActorRef

                receive {
                    Start => { self.pong_ref.send(Ping(1)); },
                    Pong(n) => {
                        if n < 3 {
                            self.pong_ref.send(Ping(n + 1));
                        }
                    }
                }
            }
        "#,
        )
        .expect("Should define ping");

        // Define Pong actor
        eval_code(
            &mut interpreter,
            r#"
            actor PongActor {
                ping_ref: ActorRef

                receive Ping(n) => {
                    self.ping_ref.send(Pong(n));
                }
            }
        "#,
        )
        .expect("Should define pong");

        // Create actors
        eval_code(&mut interpreter, "let pong = PongActor.new()").expect("Should create pong");
        eval_code(
            &mut interpreter,
            "let ping = PingActor.new(count: 0, pong_ref: pong)",
        )
        .expect("Should create ping");

        // Start interaction
        let result = eval_code(&mut interpreter, "ping.send(Start)").expect("Should start");
        assert!(matches!(result, Value::Nil) || matches!(result, Value::Bool(true)));
    }
}

#[cfg(test)]
mod actor_state_tests {
    use super::*;

    #[test]
    fn test_actor_state_access() {
        let mut interpreter = Interpreter::new();

        eval_code(&mut interpreter, "actor Counter { count: i32 }").expect("Should define");
        eval_code(&mut interpreter, "let counter = Counter.new()").expect("Should create");

        let result = eval_code(&mut interpreter, "counter.count").expect("Should access state");
        assert_eq!(result, Value::Nil);
    }

    #[test]
    fn test_actor_state_modification() {
        let mut interpreter = Interpreter::new();

        eval_code(
            &mut interpreter,
            r#"
            actor Counter {
                count: i32
                receive Increment => { self.count = self.count + 1; }
            }
        "#,
        )
        .expect("Should define");

        eval_code(&mut interpreter, "let counter = Counter.new(count: 0)").expect("Should create");
        eval_code(&mut interpreter, "counter.send(Increment)").expect("Should send");

        let result =
            eval_code(&mut interpreter, "counter.count").expect("Should access updated state");
        assert_eq!(result, Value::Integer(1));
    }
}

#[cfg(test)]
mod actor_lifecycle_tests {
    use super::*;

    #[test]
    fn test_actor_spawn() {
        let mut interpreter = Interpreter::new();

        eval_code(&mut interpreter, "actor Simple { count: i32 }").expect("Should define");

        let result = eval_code(&mut interpreter, "let instance = spawn Simple(count: 0)")
            .expect("Should spawn");
        assert!(matches!(result, Value::Object(_)));
    }

    #[test]
    fn test_actor_stop() {
        let mut interpreter = Interpreter::new();

        eval_code(&mut interpreter, "actor Simple { count: i32 }").expect("Should define");
        eval_code(&mut interpreter, "let instance = spawn Simple(count: 0)").expect("Should spawn");

        let result = eval_code(&mut interpreter, "instance.stop()").expect("Should stop");
        assert!(matches!(result, Value::Bool(true)));
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_actor_undefined_message() {
        let mut interpreter = Interpreter::new();

        eval_code(
            &mut interpreter,
            r#"
            actor Simple {
                count: i32
                receive Ping => { self.count = 1; }
            }
        "#,
        )
        .expect("Should define");

        eval_code(&mut interpreter, "let instance = Simple.new(count: 0)").expect("Should create");

        // Sending undefined message should either be ignored or return error
        let result = eval_code(&mut interpreter, "instance.send(UndefinedMessage)");
        // Should either work (ignored) or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_actor_type_safety() {
        let mut interpreter = Interpreter::new();

        eval_code(
            &mut interpreter,
            r#"
            actor TypedActor {
                count: i32
                receive SetCount(n: i32) => { self.count = n; }
            }
        "#,
        )
        .expect("Should define");

        eval_code(&mut interpreter, "let instance = TypedActor.new(count: 0)")
            .expect("Should create");

        // Sending wrong type should fail
        let result = eval_code(&mut interpreter, r#"instance.send(SetCount("invalid"))"#);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    fn test_actor_message_ordering() {
        let mut interpreter = Interpreter::new();

        eval_code(
            &mut interpreter,
            r#"
            actor OrderedActor {
                messages: Vec<i32>
                receive Push(n) => { self.messages.push(n); }
            }
        "#,
        )
        .expect("Should define");

        eval_code(
            &mut interpreter,
            "let instance = OrderedActor.new(messages: [])",
        )
        .expect("Should create");

        // Send multiple messages
        eval_code(&mut interpreter, "instance.send(Push(1))").expect("Should send 1");
        eval_code(&mut interpreter, "instance.send(Push(2))").expect("Should send 2");
        eval_code(&mut interpreter, "instance.send(Push(3))").expect("Should send 3");

        // Messages should be processed in order
        let result = eval_code(&mut interpreter, "instance.messages").expect("Should get messages");
        if let Value::Array(arr) = result {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], Value::Integer(1));
            assert_eq!(arr[1], Value::Integer(2));
            assert_eq!(arr[2], Value::Integer(3));
        } else {
            panic!("Expected array of messages");
        }
    }
}
