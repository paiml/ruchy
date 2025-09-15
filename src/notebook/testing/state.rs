/// State management for notebook testing sessions
#[derive(Clone, Default)]
pub struct TestState {
    variables: Vec<(String, String)>,
}
impl TestState {
/// # Examples
/// 
/// ```
/// use ruchy::notebook::testing::state::new;
/// 
/// let result = new(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn new() -> Self {
        Self::default()
    }
/// # Examples
/// 
/// ```
/// use ruchy::notebook::testing::state::is_empty;
/// 
/// let result = is_empty(());
/// assert_eq!(result, Ok(()));
/// ```
pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }
/// # Examples
/// 
/// ```
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
/// use ruchy::notebook::testing::state::get_variable;
/// 
/// let result = get_variable("example");
/// assert_eq!(result, Ok(()));
/// ```
pub fn get_variable(&self, name: &str) -> Option<&String> {
        self.variables
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }
}
