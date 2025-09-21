/// State management for notebook testing sessions
#[derive(Clone, Default)]
pub struct TestState {
    variables: Vec<(String, String)>,
}
impl TestState {
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::state::TestState;
    ///
    /// let instance = TestState::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self::default()
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::state::TestState;
    ///
    /// let mut instance = TestState::new();
    /// let result = instance.is_empty();
    /// // Verify behavior
    /// ```
    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::state::set_variable;
    ///
    /// let result = set_variable(());
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn set_variable(&mut self, name: String, value: String) {
        if let Some(var) = self.variables.iter_mut().find(|(n, _)| n == &name) {
            var.1 = value;
        } else {
            self.variables.push((name, value));
        }
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::state::TestState;
    ///
    /// let mut instance = TestState::new();
    /// let result = instance.get_variable();
    /// // Verify behavior
    /// ```
    pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }
}
