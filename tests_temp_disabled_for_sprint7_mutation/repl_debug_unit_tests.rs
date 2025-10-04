//! Unit tests for the debug module
//! Target: 80% coverage of debug and introspection functionality

#[cfg(test)]
mod debug_tests {
    use ruchy::frontend::ast::{Expr, ExprKind, Literal};
    use ruchy::runtime::repl::debug::{
        Breakpoint, BreakpointCondition, DebugConfig, DebugEvent, DebugManager, MemoryTracker,
        ProfileData, Profiler, TraceContext,
    };
    use ruchy::runtime::repl::Value;
    use std::collections::HashMap;
    use std::time::{Duration, Instant};

    fn create_test_config() -> DebugConfig {
        DebugConfig {
            trace_enabled: false,
            break_on_error: false,
            profile_enabled: false,
            memory_tracking: false,
            max_trace_depth: 100,
            event_buffer_size: 1000,
        }
    }

    fn create_test_expr() -> Expr {
        Expr {
            kind: ExprKind::Literal(Literal::Int(42)),
            span: Default::default(),
        }
    }

    #[test]
    fn test_debug_manager_creation() {
        let config = create_test_config();
        let manager = DebugManager::new(config);

        assert!(!manager.is_tracing());
        assert!(!manager.is_profiling());
        assert_eq!(manager.event_count(), 0);
        assert_eq!(manager.breakpoint_count(), 0);
    }

    #[test]
    fn test_trace_enabling() {
        let mut config = create_test_config();
        config.trace_enabled = true;
        let mut manager = DebugManager::new(config);

        assert!(manager.is_tracing());

        manager.disable_tracing();
        assert!(!manager.is_tracing());

        manager.enable_tracing();
        assert!(manager.is_tracing());
    }

    #[test]
    fn test_trace_eval() {
        let mut config = create_test_config();
        config.trace_enabled = true;
        let mut manager = DebugManager::new(config);

        let expr = create_test_expr();
        let ctx = manager.trace_eval_start(&expr, 0);

        assert!(ctx.is_some());
        let ctx = ctx.unwrap();
        assert_eq!(ctx.depth, 0);

        let value = Value::Int(42);
        manager.trace_eval_end(ctx, &value);

        assert_eq!(manager.event_count(), 2); // Start and end events
    }

    #[test]
    fn test_trace_depth_limit() {
        let mut config = create_test_config();
        config.trace_enabled = true;
        config.max_trace_depth = 3;
        let mut manager = DebugManager::new(config);

        let expr = create_test_expr();

        // Within limit
        let ctx = manager.trace_eval_start(&expr, 2);
        assert!(ctx.is_some());

        // At limit
        let ctx = manager.trace_eval_start(&expr, 3);
        assert!(ctx.is_none());

        // Over limit
        let ctx = manager.trace_eval_start(&expr, 5);
        assert!(ctx.is_none());
    }

    #[test]
    fn test_breakpoint_management() {
        let config = create_test_config();
        let mut manager = DebugManager::new(config);

        // Add breakpoints
        let bp1 = manager.add_breakpoint(10, None);
        let bp2 = manager.add_breakpoint(20, Some("x > 5".to_string()));

        assert_eq!(manager.breakpoint_count(), 2);
        assert!(manager.has_breakpoint(10));
        assert!(manager.has_breakpoint(20));
        assert!(!manager.has_breakpoint(30));

        // Remove breakpoint
        manager.remove_breakpoint(bp1);
        assert_eq!(manager.breakpoint_count(), 1);
        assert!(!manager.has_breakpoint(10));
        assert!(manager.has_breakpoint(20));
    }

    #[test]
    fn test_conditional_breakpoint() {
        let config = create_test_config();
        let mut manager = DebugManager::new(config);

        let bp_id = manager.add_breakpoint(10, Some("counter == 5".to_string()));
        let bp = manager.get_breakpoint(bp_id);

        assert!(bp.is_some());
        let bp = bp.unwrap();
        assert_eq!(bp.line, 10);
        assert!(bp.condition.is_some());
        assert_eq!(bp.condition.unwrap(), "counter == 5");
        assert!(bp.enabled);
    }

    #[test]
    fn test_breakpoint_enable_disable() {
        let config = create_test_config();
        let mut manager = DebugManager::new(config);

        let bp_id = manager.add_breakpoint(10, None);

        assert!(manager.is_breakpoint_enabled(bp_id));

        manager.disable_breakpoint(bp_id);
        assert!(!manager.is_breakpoint_enabled(bp_id));

        manager.enable_breakpoint(bp_id);
        assert!(manager.is_breakpoint_enabled(bp_id));
    }

    #[test]
    fn test_clear_breakpoints() {
        let config = create_test_config();
        let mut manager = DebugManager::new(config);

        manager.add_breakpoint(10, None);
        manager.add_breakpoint(20, None);
        manager.add_breakpoint(30, None);

        assert_eq!(manager.breakpoint_count(), 3);

        manager.clear_breakpoints();
        assert_eq!(manager.breakpoint_count(), 0);
    }

    #[test]
    fn test_profiling() {
        let mut config = create_test_config();
        config.profile_enabled = true;
        let mut manager = DebugManager::new(config);

        assert!(manager.is_profiling());

        manager.profile_start("test_function");
        std::thread::sleep(Duration::from_millis(10));
        manager.profile_end("test_function");

        let profile = manager.get_profile_data("test_function");
        assert!(profile.is_some());

        let profile = profile.unwrap();
        assert_eq!(profile.name, "test_function");
        assert_eq!(profile.call_count, 1);
        assert!(profile.total_time > Duration::ZERO);
    }

    #[test]
    fn test_profile_multiple_calls() {
        let mut config = create_test_config();
        config.profile_enabled = true;
        let mut manager = DebugManager::new(config);

        for _ in 0..5 {
            manager.profile_start("repeated_func");
            std::thread::sleep(Duration::from_millis(5));
            manager.profile_end("repeated_func");
        }

        let profile = manager.get_profile_data("repeated_func");
        assert!(profile.is_some());

        let profile = profile.unwrap();
        assert_eq!(profile.call_count, 5);
        assert!(profile.average_time > Duration::ZERO);
    }

    #[test]
    fn test_profile_report() {
        let mut config = create_test_config();
        config.profile_enabled = true;
        let mut manager = DebugManager::new(config);

        manager.profile_start("func1");
        manager.profile_end("func1");

        manager.profile_start("func2");
        manager.profile_end("func2");

        let report = manager.get_profile_report();
        assert!(report.contains("func1"));
        assert!(report.contains("func2"));
        assert!(report.contains("calls"));
    }

    #[test]
    fn test_memory_tracking() {
        let mut config = create_test_config();
        config.memory_tracking = true;
        let mut manager = DebugManager::new(config);

        manager.track_allocation("test_var", 1024);
        manager.track_allocation("test_array", 4096);

        let usage = manager.get_memory_usage();
        assert_eq!(usage, 5120);

        manager.track_deallocation("test_var");
        let usage = manager.get_memory_usage();
        assert_eq!(usage, 4096);
    }

    #[test]
    fn test_memory_report() {
        let mut config = create_test_config();
        config.memory_tracking = true;
        let mut manager = DebugManager::new(config);

        manager.track_allocation("var1", 512);
        manager.track_allocation("var2", 1024);
        manager.track_allocation("var3", 2048);

        let report = manager.get_memory_report();
        assert!(report.contains("var1"));
        assert!(report.contains("512"));
        assert!(report.contains("Total"));
        assert!(report.contains("3584"));
    }

    #[test]
    fn test_event_logging() {
        let mut config = create_test_config();
        config.trace_enabled = true;
        let mut manager = DebugManager::new(config);

        manager.log_event(DebugEvent::Info("Test info".to_string()));
        manager.log_event(DebugEvent::Warning("Test warning".to_string()));
        manager.log_event(DebugEvent::Error("Test error".to_string()));

        assert_eq!(manager.event_count(), 3);

        let events = manager.get_events(10);
        assert_eq!(events.len(), 3);
    }

    #[test]
    fn test_event_buffer_limit() {
        let mut config = create_test_config();
        config.trace_enabled = true;
        config.event_buffer_size = 5;
        let mut manager = DebugManager::new(config);

        for i in 0..10 {
            manager.log_event(DebugEvent::Info(format!("Event {}", i)));
        }

        // Should only keep last 5 events
        assert_eq!(manager.event_count(), 5);

        let events = manager.get_events(10);
        assert_eq!(events.len(), 5);
        assert!(events[0].contains("Event 5"));
        assert!(events[4].contains("Event 9"));
    }

    #[test]
    fn test_get_trace() {
        let mut config = create_test_config();
        config.trace_enabled = true;
        let mut manager = DebugManager::new(config);

        let expr = create_test_expr();

        for i in 0..3 {
            if let Some(ctx) = manager.trace_eval_start(&expr, i) {
                let value = Value::Int(i as i64);
                manager.trace_eval_end(ctx, &value);
            }
        }

        let trace = manager.get_trace(None);
        assert_eq!(trace.len(), 6); // 3 starts + 3 ends

        let trace = manager.get_trace(Some(2));
        assert_eq!(trace.len(), 2);
    }

    #[test]
    fn test_clear_debug_data() {
        let mut config = create_test_config();
        config.trace_enabled = true;
        config.profile_enabled = true;
        config.memory_tracking = true;
        let mut manager = DebugManager::new(config);

        // Add various debug data
        manager.log_event(DebugEvent::Info("Test".to_string()));
        manager.add_breakpoint(10, None);
        manager.profile_start("test");
        manager.profile_end("test");
        manager.track_allocation("test", 1024);

        // Clear all
        manager.clear();

        assert_eq!(manager.event_count(), 0);
        assert_eq!(manager.breakpoint_count(), 0);
        assert!(manager.get_profile_data("test").is_none());
        assert_eq!(manager.get_memory_usage(), 0);
    }

    #[test]
    fn test_watch_expressions() {
        let config = create_test_config();
        let mut manager = DebugManager::new(config);

        manager.add_watch("x + y");
        manager.add_watch("result * 2");

        assert_eq!(manager.watch_count(), 2);

        let watches = manager.get_watches();
        assert!(watches.contains(&"x + y".to_string()));
        assert!(watches.contains(&"result * 2".to_string()));

        manager.remove_watch("x + y");
        assert_eq!(manager.watch_count(), 1);
    }

    #[test]
    fn test_step_execution() {
        let config = create_test_config();
        let mut manager = DebugManager::new(config);

        manager.set_step_mode(StepMode::StepOver);
        assert_eq!(manager.get_step_mode(), StepMode::StepOver);

        manager.set_step_mode(StepMode::StepInto);
        assert_eq!(manager.get_step_mode(), StepMode::StepInto);

        manager.set_step_mode(StepMode::StepOut);
        assert_eq!(manager.get_step_mode(), StepMode::StepOut);

        manager.set_step_mode(StepMode::Continue);
        assert_eq!(manager.get_step_mode(), StepMode::Continue);
    }

    #[test]
    fn test_stack_trace() {
        let config = create_test_config();
        let mut manager = DebugManager::new(config);

        manager.push_frame("main", 1);
        manager.push_frame("function1", 10);
        manager.push_frame("function2", 25);

        let stack = manager.get_stack_trace();
        assert_eq!(stack.len(), 3);
        assert!(stack[0].contains("function2"));
        assert!(stack[1].contains("function1"));
        assert!(stack[2].contains("main"));

        manager.pop_frame();
        let stack = manager.get_stack_trace();
        assert_eq!(stack.len(), 2);
    }

    #[test]
    fn test_inspect_value() {
        let config = create_test_config();
        let manager = DebugManager::new(config);

        let value = Value::Object({
            let mut map = HashMap::new();
            map.insert("x".to_string(), Value::Int(10));
            map.insert("y".to_string(), Value::String("test".to_string()));
            map
        });

        let inspection = manager.inspect_value(&value, 0);
        assert!(inspection.contains("Object"));
        assert!(inspection.contains("x"));
        assert!(inspection.contains("10"));
        assert!(inspection.contains("y"));
        assert!(inspection.contains("test"));
    }

    #[test]
    fn test_format_duration() {
        let manager = DebugManager::new(create_test_config());

        assert_eq!(manager.format_duration(Duration::from_nanos(500)), "500ns");
        assert_eq!(
            manager.format_duration(Duration::from_micros(1500)),
            "1.5ms"
        );
        assert_eq!(manager.format_duration(Duration::from_millis(1200)), "1.2s");
        assert_eq!(manager.format_duration(Duration::from_secs(90)), "1m 30s");
    }
}

#[cfg(test)]
mod debug_config_tests {
    use ruchy::runtime::repl::debug::DebugConfig;

    #[test]
    fn test_default_config() {
        let config = DebugConfig::default();

        assert!(!config.trace_enabled);
        assert!(!config.break_on_error);
        assert!(!config.profile_enabled);
        assert!(!config.memory_tracking);
        assert_eq!(config.max_trace_depth, 1000);
        assert_eq!(config.event_buffer_size, 10000);
    }

    #[test]
    fn test_debug_mode_config() {
        let config = DebugConfig::debug_mode();

        assert!(config.trace_enabled);
        assert!(config.break_on_error);
        assert!(config.profile_enabled);
        assert!(config.memory_tracking);
    }

    #[test]
    fn test_production_config() {
        let config = DebugConfig::production();

        assert!(!config.trace_enabled);
        assert!(!config.break_on_error);
        assert!(!config.profile_enabled);
        assert!(!config.memory_tracking);
        assert_eq!(config.event_buffer_size, 100); // Minimal buffer
    }
}

#[cfg(test)]
mod profiler_tests {
    use ruchy::runtime::repl::debug::Profiler;
    use std::time::Duration;

    #[test]
    fn test_profiler_new() {
        let profiler = Profiler::new();
        assert!(profiler.is_empty());
    }

    #[test]
    fn test_profiler_timing() {
        let mut profiler = Profiler::new();

        profiler.start("test");
        std::thread::sleep(Duration::from_millis(10));
        profiler.end("test");

        let data = profiler.get_data("test");
        assert!(data.is_some());

        let data = data.unwrap();
        assert_eq!(data.call_count, 1);
        assert!(data.total_time >= Duration::from_millis(10));
    }

    #[test]
    fn test_profiler_nested() {
        let mut profiler = Profiler::new();

        profiler.start("outer");
        profiler.start("inner");
        std::thread::sleep(Duration::from_millis(5));
        profiler.end("inner");
        std::thread::sleep(Duration::from_millis(5));
        profiler.end("outer");

        let outer = profiler.get_data("outer").unwrap();
        let inner = profiler.get_data("inner").unwrap();

        assert!(outer.total_time >= Duration::from_millis(10));
        assert!(inner.total_time >= Duration::from_millis(5));
        assert!(inner.total_time < outer.total_time);
    }
}

#[cfg(test)]
mod memory_tracker_tests {
    use ruchy::runtime::repl::debug::MemoryTracker;

    #[test]
    fn test_memory_tracker_new() {
        let tracker = MemoryTracker::new();
        assert_eq!(tracker.total_allocated(), 0);
    }

    #[test]
    fn test_allocation_tracking() {
        let mut tracker = MemoryTracker::new();

        tracker.allocate("var1", 1024);
        assert_eq!(tracker.total_allocated(), 1024);

        tracker.allocate("var2", 2048);
        assert_eq!(tracker.total_allocated(), 3072);

        tracker.deallocate("var1");
        assert_eq!(tracker.total_allocated(), 2048);
    }

    #[test]
    fn test_peak_memory() {
        let mut tracker = MemoryTracker::new();

        tracker.allocate("var1", 1024);
        tracker.allocate("var2", 2048);
        assert_eq!(tracker.peak_allocated(), 3072);

        tracker.deallocate("var1");
        assert_eq!(tracker.total_allocated(), 2048);
        assert_eq!(tracker.peak_allocated(), 3072); // Peak remains
    }

    #[test]
    fn test_allocation_count() {
        let mut tracker = MemoryTracker::new();

        tracker.allocate("var1", 100);
        tracker.allocate("var2", 200);
        tracker.allocate("var3", 300);

        assert_eq!(tracker.allocation_count(), 3);

        tracker.deallocate("var2");
        assert_eq!(tracker.allocation_count(), 2);
    }
}
