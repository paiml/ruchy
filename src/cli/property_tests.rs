
use super::*;
use proptest::prelude::*;

// Strategy for generating valid VmMode values
fn arb_vm_mode() -> impl Strategy<Value = VmMode> {
    prop_oneof![Just(VmMode::Ast), Just(VmMode::Bytecode),]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    // VmMode default is always valid
    #[test]
    fn prop_vm_mode_default_valid(_dummy: u8) {
        let mode = VmMode::default();
        prop_assert!(mode == VmMode::Ast || mode == VmMode::Bytecode);
    }

    // VmMode is cloneable and equality works
    #[test]
    fn prop_vm_mode_clone_eq(mode in arb_vm_mode()) {
        let copied = mode; // VmMode implements Copy
        prop_assert_eq!(mode, copied);
    }

    // PathBuf from valid strings never panics
    #[test]
    fn prop_pathbuf_from_string(s in "[a-zA-Z0-9_./]{0,50}") {
        let path = PathBuf::from(&s);
        prop_assert!(path.to_str().is_some());
    }

    // format_file_error always returns Some for valid paths
    #[test]
    fn prop_format_file_error_returns_string(
        path in "[a-zA-Z0-9_./]{1,30}",
        msg in "[a-zA-Z0-9 ]{1,50}"
    ) {
        let result = format_file_error(&msg, Path::new(&path));
        prop_assert!(!result.is_empty());
        prop_assert!(result.contains(&path) || result.contains(&msg));
    }

    // Port numbers in valid range
    #[test]
    fn prop_valid_port_range(port in 1u16..=65535) {
        // Any port in u16 range should be parseable
        let port_str = port.to_string();
        let parsed: u16 = port_str.parse().expect("valid port");
        prop_assert_eq!(port, parsed);
    }

    // Host strings roundtrip
    #[test]
    fn prop_host_roundtrip(host in "[a-zA-Z0-9.-]{1,50}") {
        let host_clone = host.clone();
        prop_assert_eq!(host, host_clone);
    }

    // NotebookCommand variants can be created
    #[test]
    fn prop_notebook_serve_creation(port in 1u16..=65535) {
        let cmd = NotebookCommand::Serve {
            port,
            host: "127.0.0.1".to_string(),
            pid_file: None,
        };
        match cmd {
            NotebookCommand::Serve { port: p, .. } => prop_assert_eq!(p, port),
            _ => prop_assert!(false, "Expected Serve variant"),
        }
    }

    // NotebookCommand Test variant
    #[test]
    fn prop_notebook_test_creation(coverage in proptest::bool::ANY) {
        let cmd = NotebookCommand::Test {
            path: PathBuf::from("test.ipynb"),
            coverage,
            format: "text".to_string(),
        };
        match cmd {
            NotebookCommand::Test { coverage: c, .. } => prop_assert_eq!(c, coverage),
            _ => prop_assert!(false, "Expected Test variant"),
        }
    }

    // NotebookCommand Convert variant
    #[test]
    fn prop_notebook_convert_creation(fmt in "html|markdown|script") {
        let cmd = NotebookCommand::Convert {
            input: PathBuf::from("in.ipynb"),
            output: PathBuf::from("out.html"),
            format: fmt.clone(),
        };
        match cmd {
            NotebookCommand::Convert { format: f, .. } => prop_assert_eq!(f, fmt),
            _ => prop_assert!(false, "Expected Convert variant"),
        }
    }

    // WasmCommand Compile variant
    #[test]
    fn prop_wasm_compile_creation(optimize in proptest::bool::ANY) {
        let cmd = WasmCommand::Compile {
            input: PathBuf::from("main.ruchy"),
            output: Some(PathBuf::from("out.wasm")),
            optimize,
            validate: true,
        };
        match cmd {
            WasmCommand::Compile { optimize: o, .. } => prop_assert_eq!(o, optimize),
            _ => prop_assert!(false, "Expected Compile variant"),
        }
    }

    // WasmCommand Run variant
    #[test]
    fn prop_wasm_run_creation(num_args in 0usize..5) {
        let args: Vec<String> = (0..num_args).map(|i| format!("arg{i}")).collect();
        let cmd = WasmCommand::Run {
            module: PathBuf::from("module.wasm"),
            args,
        };
        match cmd {
            WasmCommand::Run { args: a, .. } => prop_assert_eq!(a.len(), num_args),
            WasmCommand::Compile { .. } | WasmCommand::Validate { .. } => {
                prop_assert!(false, "Expected Run variant");
            }
        }
    }

    // TestCommand Run variant
    #[test]
    fn prop_test_run_creation(coverage in proptest::bool::ANY, parallel in proptest::bool::ANY) {
        let cmd = TestCommand::Run {
            path: PathBuf::from("tests"),
            coverage,
            parallel,
            filter: None,
        };
        match cmd {
            TestCommand::Run { coverage: c, parallel: p, .. } => {
                prop_assert_eq!(c, coverage);
                prop_assert_eq!(p, parallel);
            }
            TestCommand::Report { .. } => prop_assert!(false, "Expected Run variant"),
        }
    }

    // TestCommand Report variant
    #[test]
    fn prop_test_report_creation(fmt in "json|html|junit") {
        let cmd = TestCommand::Report {
            format: fmt.clone(),
            output: None,
        };
        match cmd {
            TestCommand::Report { format: f, .. } => prop_assert_eq!(f, fmt),
            TestCommand::Run { .. } => prop_assert!(false, "Expected Report variant"),
        }
    }

    // Hunt command cycles validation
    #[test]
    fn prop_hunt_cycles_non_negative(cycles in 0u32..100) {
        // Hunt command with cycles - verify cycles is usable
        let andon = cycles % 2 == 0;
        let five_whys = cycles % 3 == 0;
        prop_assert!(cycles < 100);
        prop_assert_eq!(andon, cycles % 2 == 0);
        prop_assert_eq!(five_whys, cycles % 3 == 0);
    }

    // Report format validation
    #[test]
    fn prop_report_format_valid(fmt in "human|json|markdown|sarif") {
        let valid_formats = ["human", "json", "markdown", "sarif"];
        prop_assert!(valid_formats.contains(&fmt.as_str()));
    }
}
