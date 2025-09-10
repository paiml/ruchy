use crate::notebook::testing::types::*;

/// State management for notebook testing sessions
#[derive(Clone, Default)]
pub struct TestState {
    variables: Vec<(String, String)>,
}

impl TestState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    pub fn set_variable(&mut self, name: String, value: String) {
        if let Some(var) = self.variables.iter_mut().find(|(n, _)| n == &name) {
            var.1 = value;
        } else {
            self.variables.push((name, value));
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }
}