//! Strategic TDD tests for parser/actors.rs - Target: 3.08% → 60%+ coverage  
//! Focus: Test implemented functionality to maximize coverage gains

#[cfg(test)]
mod tests {
    use ruchy::frontend::parser::Parser;
    use ruchy::frontend::ast::{Expr, ExprKind, StructField};
    
    // Helper function (complexity: 3)
    fn parse_actor_str(input: &str) -> Result<Expr, Box<dyn std::error::Error>> {
        let mut parser = Parser::new(input);
        let expr = parser.parse()?;
        Ok(expr)
    }
    
    // Helper to extract actor details (complexity: 4) 
    fn extract_actor_info(expr: &Expr) -> Option<(&str, &[StructField])> {
        if let ExprKind::Actor { name, state, handlers: _ } = &expr.kind {
            Some((name, state))
        } else {
            None
        }
    }
    
    // Basic actor parsing tests (complexity: 2 each)
    #[test]
    fn test_parse_empty_actor() {
        let input = r#"
            actor EmptyActor {
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse empty actor");
        
        let expr = result.unwrap();
        let (name, state) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(name, "EmptyActor");
        assert!(state.is_empty());
    }
    
    #[test]
    fn test_parse_actor_with_simple_state_field() {
        let input = r#"
            actor CounterActor {
                counter: i32
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse actor with state field");
        
        let expr = result.unwrap();
        let (name, state, _handlers) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(name, "CounterActor");
        assert_eq!(state.len(), 1);
        assert_eq!(state[0].name, "counter");
        assert!(!state[0].is_pub);
    }
    
    #[test]
    fn test_parse_actor_with_multiple_state_fields() {
        let input = r#"
            actor MultiStateActor {
                name: String,
                age: i32;
                active: bool
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse actor with multiple state fields");
        
        let expr = result.unwrap();
        let (name, state, _handlers) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(name, "MultiStateActor");
        assert_eq!(state.len(), 3);
        
        assert_eq!(state[0].name, "name");
        assert_eq!(state[1].name, "age"); 
        assert_eq!(state[2].name, "active");
    }
    
    #[test]
    fn test_parse_actor_with_state_block() {
        let input = r#"
            actor StateBlockActor {
                state {
                    counter: i32,
                    name: String;
                    active: bool
                }
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse actor with state block");
        
        let expr = result.unwrap();
        let (name, state, _handlers) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(name, "StateBlockActor");
        assert_eq!(state.len(), 3);
        
        assert_eq!(state[0].name, "counter");
        assert_eq!(state[1].name, "name");
        assert_eq!(state[2].name, "active");
    }
    
    // Handler parsing tests (complexity: 3 each)
    #[test]
    fn test_parse_actor_with_simple_receive_handler() {
        let input = r#"
            actor HandlerActor {
                counter: i32
                receive Increment() {
                    self.counter += 1
                }
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse actor with receive handler");
        
        let expr = result.unwrap();
        let (_name, _state, handlers) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].message_type, "Increment");
        assert!(handlers[0].params.is_empty());
    }
    
    #[test]
    fn test_parse_actor_with_parameterized_handler() {
        let input = r#"
            actor ParamActor {
                value: i32
                receive Update(new_val: i32) {
                    self.value = new_val
                }
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse actor with parameterized handler");
        
        let expr = result.unwrap();
        let (_name, _state, handlers) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].message_type, "Update");
        assert_eq!(handlers[0].params.len(), 1);
        assert_eq!(handlers[0].params[0].name(), "new_val");
    }
    
    #[test]
    fn test_parse_actor_with_multiple_handlers() {
        let input = r#"
            actor MultiHandlerActor {
                counter: i32
                receive Increment() {
                    self.counter += 1
                }
                receive Decrement() {
                    self.counter -= 1
                }
                receive Reset() {
                    self.counter = 0
                }
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse actor with multiple handlers");
        
        let expr = result.unwrap();
        let (_name, _state, handlers) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(handlers.len(), 3);
        
        assert_eq!(handlers[0].message_type, "Increment");
        assert_eq!(handlers[1].message_type, "Decrement");
        assert_eq!(handlers[2].message_type, "Reset");
    }
    
    #[test]
    fn test_parse_actor_with_receive_block() {
        let input = r#"
            actor ReceiveBlockActor {
                value: String
                receive {
                    SetValue(v: String) => {
                        self.value = v
                    },
                    GetValue() => {
                        self.value.clone()
                    }
                }
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse actor with receive block");
        
        let expr = result.unwrap();
        let (_name, _state, handlers) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(handlers.len(), 2);
        
        assert_eq!(handlers[0].message_type, "SetValue");
        assert_eq!(handlers[1].message_type, "GetValue");
    }
    
    #[test]
    fn test_parse_actor_with_handler_return_type() {
        let input = r#"
            actor ReturnTypeActor {
                data: Vec<i32>
                receive GetLength() -> i32 {
                    self.data.len() as i32
                }
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse actor with handler return type");
        
        let expr = result.unwrap();
        let (_name, _state, handlers) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].message_type, "GetLength");
    }
    
    // Complex actor tests (complexity: 4 each)
    #[test]
    fn test_parse_complex_actor_with_state_and_handlers() {
        let input = r#"
            actor ComplexActor {
                state {
                    name: String,
                    counter: i32,
                    active: bool
                }
                
                receive SetName(new_name: String) {
                    self.name = new_name
                }
                
                receive {
                    Increment(amount: i32) => {
                        self.counter += amount
                    },
                    Activate() => {
                        self.active = true
                    },
                    Deactivate() => {
                        self.active = false
                    }
                }
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse complex actor");
        
        let expr = result.unwrap();
        let (name, state, handlers) = extract_actor_info(&expr).expect("Expected actor");
        
        assert_eq!(name, "ComplexActor");
        assert_eq!(state.len(), 3);
        assert_eq!(handlers.len(), 4);
        
        // Verify state fields
        assert_eq!(state[0].name, "name");
        assert_eq!(state[1].name, "counter");
        assert_eq!(state[2].name, "active");
        
        // Verify handlers
        assert_eq!(handlers[0].message_type, "SetName");
        assert_eq!(handlers[1].message_type, "Increment");
        assert_eq!(handlers[2].message_type, "Activate");
        assert_eq!(handlers[3].message_type, "Deactivate");
    }
    
    #[test]
    fn test_parse_actor_with_default_values() {
        let input = r#"
            actor DefaultValueActor {
                counter: i32 = 0,
                name: String = "default";
                active: bool = true
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse actor with default values");
        
        let expr = result.unwrap();
        let (_name, state, _handlers) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(state.len(), 3);
        
        assert_eq!(state[0].name, "counter");
        assert_eq!(state[1].name, "name");
        assert_eq!(state[2].name, "active");
    }
    
    // Error condition tests (complexity: 2 each)
    #[test]
    fn test_parse_actor_missing_name() {
        let input = r#"
            actor {
                counter: i32
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_err(), "Should fail on missing actor name");
    }
    
    #[test]
    fn test_parse_actor_missing_braces() {
        let input = r#"
            actor MissingBraces
                counter: i32
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_err(), "Should fail on missing braces");
    }
    
    #[test]
    fn test_parse_actor_missing_field_type() {
        let input = r#"
            actor MissingTypeActor {
                field_name
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_err(), "Should fail on missing field type");
    }
    
    #[test]
    fn test_parse_actor_invalid_receive_syntax() {
        let input = r#"
            actor InvalidReceiveActor {
                counter: i32
                receive 
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_err(), "Should fail on invalid receive syntax");
    }
    
    #[test]
    fn test_parse_actor_missing_handler_arrow() {
        let input = r#"
            actor MissingArrowActor {
                receive {
                    Message() {
                        // Missing => arrow
                    }
                }
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_err(), "Should fail on missing fat arrow in receive block");
    }
    
    // Edge case tests (complexity: 3 each)
    #[test]
    fn test_parse_actor_with_complex_types() {
        let input = r#"
            actor ComplexTypeActor {
                data: HashMap<String, Vec<i32>>,
                callback: fn(i32) -> bool;
                optional: Option<String>
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse actor with complex types");
        
        let expr = result.unwrap();
        let (_name, state, _handlers) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(state.len(), 3);
        
        assert_eq!(state[0].name, "data");
        assert_eq!(state[1].name, "callback");
        assert_eq!(state[2].name, "optional");
    }
    
    #[test]
    fn test_parse_actor_with_nested_handler_expressions() {
        let input = r#"
            actor NestedExprActor {
                items: Vec<i32>
                receive ProcessItems() {
                    for item in self.items {
                        if item > 0 {
                            println!("Processing: {}", item);
                        }
                    }
                }
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse actor with nested expressions");
        
        let expr = result.unwrap();
        let (_name, _state, handlers) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].message_type, "ProcessItems");
    }
    
    #[test]
    fn test_parse_actor_with_mixed_syntax() {
        let input = r#"
            actor MixedSyntaxActor {
                // Direct field
                counter: i32,
                
                // State block
                state {
                    name: String,
                    active: bool
                }
                
                // Individual handler
                receive Increment() {
                    self.counter += 1
                }
                
                // Receive block
                receive {
                    SetName(n: String) => {
                        self.name = n
                    }
                }
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse actor with mixed syntax");
        
        let expr = result.unwrap();
        let (_name, state, handlers) = extract_actor_info(&expr).expect("Expected actor");
        assert_eq!(state.len(), 3); // counter + name + active
        assert_eq!(handlers.len(), 2); // Increment + SetName
    }
    
    // Integration test (complexity: 5)
    #[test]
    fn test_parse_complete_actor_system() {
        let input = r#"
            actor WorkerActor {
                state {
                    worker_id: i32,
                    task_queue: Vec<String>,
                    is_busy: bool,
                    completed_tasks: i32
                }
                
                // Individual handler with return type
                receive GetStatus() -> String {
                    format!("Worker {}: {} tasks completed, busy: {}", 
                           self.worker_id, self.completed_tasks, self.is_busy)
                }
                
                // Receive block with multiple handlers
                receive {
                    AddTask(task: String) => {
                        self.task_queue.push(task);
                        if !self.is_busy {
                            self.process_next_task()
                        }
                    },
                    
                    ProcessNext() => {
                        if let Some(task) = self.task_queue.pop() {
                            self.is_busy = true;
                            self.process_task(task);
                            self.completed_tasks += 1;
                            self.is_busy = false
                        }
                    },
                    
                    Reset() => {
                        self.task_queue.clear();
                        self.completed_tasks = 0;
                        self.is_busy = false
                    }
                }
            }
        "#;
        
        let result = parse_actor_str(input);
        assert!(result.is_ok(), "Failed to parse complete actor system");
        
        let expr = result.unwrap();
        let (name, state, handlers) = extract_actor_info(&expr).expect("Expected actor");
        
        // Verify complete structure
        assert_eq!(name, "WorkerActor");
        assert_eq!(state.len(), 4);
        assert_eq!(handlers.len(), 4);
        
        // Verify state fields
        let field_names: Vec<&str> = state.iter().map(|f| f.name.as_str()).collect();
        assert!(field_names.contains(&"worker_id"));
        assert!(field_names.contains(&"task_queue"));
        assert!(field_names.contains(&"is_busy"));
        assert!(field_names.contains(&"completed_tasks"));
        
        // Verify handlers
        let handler_types: Vec<&str> = handlers.iter().map(|h| h.message_type.as_str()).collect();
        assert!(handler_types.contains(&"GetStatus"));
        assert!(handler_types.contains(&"AddTask"));
        assert!(handler_types.contains(&"ProcessNext"));
        assert!(handler_types.contains(&"Reset"));
    }
}