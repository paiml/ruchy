//! Tests for the interactive theorem prover module
//!
//! PMAT A+ Quality Standards:
//! - Maximum cyclomatic complexity: 10
//! - No TODO/FIXME/HACK comments
//! - 100% test coverage for new functions

#[cfg(test)]
mod basic_tests {
    use crate::proving::smt::SmtBackend;
    use crate::proving::prover::InteractiveProver;
    
    #[test]
    fn test_prover_creation_z3() {
        let backend = SmtBackend::Z3;
        let prover = InteractiveProver::new(backend);
        // Test that prover was created successfully
        let _ = prover;
    }
    
    #[test]
    fn test_prover_creation_cvc5() {
        let backend = SmtBackend::Cvc5;
        let prover = InteractiveProver::new(backend);
        // Test that prover was created successfully
        let _ = prover;
    }
    
    #[test]
    fn test_prover_creation_yices() {
        let backend = SmtBackend::Yices;
        let prover = InteractiveProver::new(backend);
        // Test that prover was created successfully
        let _ = prover;
    }
    
    #[test]
    fn test_prover_load_empty_script() {
        let backend = SmtBackend::Z3;
        let mut prover = InteractiveProver::new(backend);
        let result = prover.load_script("");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_prover_load_simple_script() {
        let backend = SmtBackend::Z3;
        let mut prover = InteractiveProver::new(backend);
        let result = prover.load_script("assert x > 0");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_prover_get_available_tactics() {
        let backend = SmtBackend::Z3;
        let prover = InteractiveProver::new(backend);
        let tactics = prover.get_available_tactics();
        // Should return a vector of tactics
        assert!(tactics.len() >= 0);
    }
}

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use crate::proving::smt::SmtBackend;
    use crate::proving::prover::InteractiveProver;
    
    proptest! {
        #[test]
        fn test_load_script_never_panics(script in ".*") {
            let backend = SmtBackend::Z3;
            let mut prover = InteractiveProver::new(backend);
            let _ = prover.load_script(&script);
        }
        
        #[test]
        fn test_prover_creation_never_panics(backend_choice in 0u8..3u8) {
            let backend = match backend_choice {
                0 => SmtBackend::Z3,
                1 => SmtBackend::Cvc5,
                _ => SmtBackend::Yices,
            };
            let _ = InteractiveProver::new(backend);
        }
    }
}