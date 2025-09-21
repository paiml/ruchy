use crate::notebook::testing::types::CellOutput;
use std::path::{Path, PathBuf};

/// Golden file manager for test snapshots
pub struct GoldenManager {
    base_path: PathBuf,
}

impl GoldenManager {
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::golden::GoldenManager;
    ///
    /// let instance = GoldenManager::new();
    /// // Verify behavior
    /// ```
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
        }
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::golden::GoldenManager;
    ///
    /// let mut instance = GoldenManager::new();
    /// let result = instance.save_golden();
    /// // Verify behavior
    /// ```
    pub fn save_golden(&self, path: &Path, output: &CellOutput) -> Result<(), String> {
        let full_path = self.base_path.join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {e}"))?;
        }
        let content = match output {
            CellOutput::Value(v) => v.clone(),
            CellOutput::DataFrame(df) => format!("{df:?}"),
            CellOutput::Error(e) => e.clone(),
            CellOutput::Html(h) => h.clone(),
            CellOutput::Plot(p) => format!("{p:?}"),
            CellOutput::None => String::new(),
        };
        std::fs::write(&full_path, content).map_err(|e| format!("Failed to write golden file: {e}"))
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::golden::GoldenManager;
    ///
    /// let mut instance = GoldenManager::new();
    /// let result = instance.load_golden();
    /// // Verify behavior
    /// ```
    pub fn load_golden(&self, path: &Path) -> Result<CellOutput, String> {
        let full_path = self.base_path.join(path);
        let content = std::fs::read_to_string(&full_path)
            .map_err(|e| format!("Failed to read golden file: {e}"))?;
        // For Sprint 0, assume all goldens are simple values
        Ok(CellOutput::Value(content))
    }
}
