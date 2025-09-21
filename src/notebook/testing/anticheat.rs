// SPRINT6-002: Anti-cheating measures implementation
// PMAT Complexity: <10 per function
use chrono::Timelike;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
#[derive(Debug, Clone)]
pub struct Submission {
    pub student_id: String,
    pub assignment_id: String,
    pub code: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub fingerprint: String,
}
#[derive(Debug, Clone)]
pub struct PlagiarismResult {
    pub is_plagiarized: bool,
    pub similarity_score: f64,
    pub matched_student: Option<String>,
    pub matched_sections: Vec<MatchedSection>,
}
#[derive(Debug, Clone)]
pub struct MatchedSection {
    pub start_line: usize,
    pub end_line: usize,
    pub similarity: f64,
}

#[derive(Debug, Clone)]
pub struct AntiCheatSystem {
    pub similarity_threshold: f64,
    pub submission_history: HashMap<String, Vec<Submission>>,
    pub fingerprint_db: HashMap<String, String>,
}

impl Default for AntiCheatSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl AntiCheatSystem {
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::anticheat::AntiCheatSystem;
    ///
    /// let instance = AntiCheatSystem::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::anticheat::AntiCheatSystem;
    ///
    /// let instance = AntiCheatSystem::new();
    /// // Verify behavior
    /// ```
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::anticheat::AntiCheatSystem;
    ///
    /// let instance = AntiCheatSystem::new();
    /// // Verify behavior
    /// ```
    pub fn new() -> Self {
        Self {
            similarity_threshold: 0.85,
            submission_history: HashMap::new(),
            fingerprint_db: HashMap::new(),
        }
    }
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::anticheat::AntiCheatSystem;
    ///
    /// let mut instance = AntiCheatSystem::new();
    /// let result = instance.with_threshold();
    /// // Verify behavior
    /// ```
    pub fn with_threshold(threshold: f64) -> Self {
        Self {
            similarity_threshold: threshold,
            submission_history: HashMap::new(),
            fingerprint_db: HashMap::new(),
        }
    }
    /// Check submission for plagiarism
    /// # Examples
    ///
    /// ```
    /// use ruchy::notebook::testing::anticheat::AntiCheatSystem;
    ///
    /// let mut instance = AntiCheatSystem::new();
    /// let result = instance.check_plagiarism();
    /// // Verify behavior
    /// ```
    pub fn check_plagiarism(&mut self, submission: &Submission) -> PlagiarismResult {
        let fingerprint = self.generate_fingerprint(&submission.code);
        // Check exact match first
        if let Some(matched_student) = self.fingerprint_db.get(&fingerprint) {
            if matched_student != &submission.student_id {
                return PlagiarismResult {
                    is_plagiarized: true,
                    similarity_score: 1.0,
                    matched_student: Some(matched_student.clone()),
                    matched_sections: vec![MatchedSection {
                        start_line: 0,
                        end_line: submission.code.lines().count(),
                        similarity: 1.0,
                    }],
                };
            }
        }
        // Check similarity with other submissions
        let mut max_similarity = 0.0;
        let mut matched_student = None;
        let mut matched_sections = Vec::new();
        for submissions in self.submission_history.values() {
            for other in submissions {
                if other.student_id == submission.student_id {
                    continue;
                }
                let similarity = self.calculate_similarity(&submission.code, &other.code);
                if similarity > max_similarity {
                    max_similarity = similarity;
                    matched_student = Some(other.student_id.clone());
                    matched_sections = self.find_matched_sections(&submission.code, &other.code);
                }
            }
        }
        // Store submission
        self.store_submission(submission.clone());
        PlagiarismResult {
            is_plagiarized: max_similarity >= self.similarity_threshold,
            similarity_score: max_similarity,
            matched_student,
            matched_sections,
        }
    }
    fn generate_fingerprint(&self, code: &str) -> String {
        let normalized = self.normalize_code(code);
        let mut hasher = Sha256::new();
        hasher.update(normalized);
        format!("{:x}", hasher.finalize())
    }
    fn normalize_code(&self, code: &str) -> String {
        // Remove comments and whitespace for comparison
        code.lines()
            .filter(|line| !line.trim().starts_with("//"))
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn calculate_similarity(&self, code1: &str, code2: &str) -> f64 {
        let tokens1 = self.tokenize(code1);
        let tokens2 = self.tokenize(code2);
        // Jaccard similarity
        let set1: HashSet<_> = tokens1.iter().collect();
        let set2: HashSet<_> = tokens2.iter().collect();
        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
    fn tokenize(&self, code: &str) -> Vec<String> {
        // Simple tokenization - split on whitespace and symbols
        code.split_whitespace()
            .flat_map(|word| word.split(|c: char| !c.is_alphanumeric() && c != '_'))
            .filter(|s| !s.is_empty())
            .map(std::string::ToString::to_string)
            .collect()
    }
    fn find_matched_sections(&self, code1: &str, code2: &str) -> Vec<MatchedSection> {
        let lines1: Vec<_> = code1.lines().collect();
        let lines2: Vec<_> = code2.lines().collect();
        let mut matches = Vec::new();
        // Find similar sections (simplified)
        for (i, line1) in lines1.iter().enumerate() {
            for line2 in &lines2 {
                if self.line_similarity(line1, line2) > 0.8 {
                    matches.push(MatchedSection {
                        start_line: i,
                        end_line: i + 1,
                        similarity: self.line_similarity(line1, line2),
                    });
                    break;
                }
            }
        }
        // Merge adjacent matches
        self.merge_adjacent_matches(matches)
    }
    fn line_similarity(&self, line1: &str, line2: &str) -> f64 {
        if line1.trim() == line2.trim() {
            1.0
        } else {
            let tokens1 = self.tokenize(line1);
            let tokens2 = self.tokenize(line2);
            if tokens1.is_empty() || tokens2.is_empty() {
                0.0
            } else {
                let common = tokens1.iter().filter(|t| tokens2.contains(t)).count();
                common as f64 / tokens1.len().max(tokens2.len()) as f64
            }
        }
    }
    fn merge_adjacent_matches(&self, mut matches: Vec<MatchedSection>) -> Vec<MatchedSection> {
        if matches.is_empty() {
            return matches;
        }
        matches.sort_by_key(|m| m.start_line);
        let mut merged = Vec::new();
        let mut current = matches[0].clone();
        for match_section in matches.into_iter().skip(1) {
            if match_section.start_line <= current.end_line + 1 {
                current.end_line = match_section.end_line;
                current.similarity = f64::midpoint(current.similarity, match_section.similarity);
            } else {
                merged.push(current);
                current = match_section;
            }
        }
        merged.push(current);
        merged
    }
    fn store_submission(&mut self, submission: Submission) {
        // Store fingerprint
        self.fingerprint_db.insert(
            submission.fingerprint.clone(),
            submission.student_id.clone(),
        );
        // Store submission history
        self.submission_history
            .entry(submission.assignment_id.clone())
            .or_default()
            .push(submission);
    }
}
/// Code obfuscation detector
pub struct ObfuscationDetector {
    suspicious_patterns: Vec<String>,
}
impl Default for ObfuscationDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl ObfuscationDetector {
    pub fn new() -> Self {
        Self {
            suspicious_patterns: vec![
                "eval".to_string(),
                "exec".to_string(),
                "compile".to_string(),
                "base64".to_string(),
                "decode".to_string(),
                "fromCharCode".to_string(),
            ],
        }
    }
    /// Check if code appears obfuscated
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::anticheat::is_obfuscated;
    ///
    /// let result = is_obfuscated("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn is_obfuscated(&self, code: &str) -> ObfuscationResult {
        let mut indicators = Vec::new();
        // Check for suspicious patterns
        for pattern in &self.suspicious_patterns {
            if code.contains(pattern) {
                indicators.push(format!("Contains suspicious pattern: {pattern}"));
            }
        }
        // Check for unusual variable names
        let var_names = self.extract_variable_names(code);
        let unusual_count = var_names
            .iter()
            .filter(|name| self.is_unusual_name(name))
            .count();
        if unusual_count > var_names.len() / 2 {
            indicators.push("High proportion of unusual variable names".to_string());
        }
        // Check for long single lines
        for line in code.lines() {
            if line.len() > 200 {
                indicators.push("Unusually long line detected".to_string());
                break;
            }
        }
        ObfuscationResult {
            is_likely_obfuscated: !indicators.is_empty(),
            confidence: indicators.len() as f64 / 10.0,
            indicators,
        }
    }
    fn extract_variable_names(&self, code: &str) -> Vec<String> {
        // Simple extraction - look for let/var declarations
        let mut names = Vec::new();
        for line in code.lines() {
            if let Some(pos) = line.find("let ") {
                let rest = &line[pos + 4..];
                if let Some(end) = rest.find(['=', ':', ';']) {
                    names.push(rest[..end].trim().to_string());
                }
            }
        }
        names
    }
    fn is_unusual_name(&self, name: &str) -> bool {
        // Check if name looks obfuscated
        name.len() == 1 ||  // Single letter
        name.chars().all(|c| c == '_') ||  // All underscores
        name.len() > 30 ||  // Very long
        name.chars().filter(|c| c.is_numeric()).count() > name.len() / 2 // Mostly numbers
    }
}
#[derive(Debug)]
pub struct ObfuscationResult {
    pub is_likely_obfuscated: bool,
    pub confidence: f64,
    pub indicators: Vec<String>,
}
/// Submission pattern analyzer
pub struct PatternAnalyzer {
    patterns: HashMap<String, SubmissionPattern>,
}
#[derive(Debug, Clone)]
struct SubmissionPattern {
    student_id: String,
    submission_times: Vec<chrono::DateTime<chrono::Utc>>,
    avg_time_between: Option<chrono::Duration>,
}
impl Default for PatternAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternAnalyzer {
    pub fn new() -> Self {
        Self {
            patterns: HashMap::new(),
        }
    }
    /// Analyze submission patterns for suspicious behavior
    /// # Examples
    ///
    /// ```ignore
    /// use ruchy::notebook::testing::anticheat::analyze_pattern;
    ///
    /// let result = analyze_pattern("example");
    /// assert_eq!(result, Ok(()));
    /// ```
    pub fn analyze_pattern(
        &mut self,
        student_id: &str,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> PatternAnalysis {
        let pattern = self
            .patterns
            .entry(student_id.to_string())
            .or_insert_with(|| SubmissionPattern {
                student_id: student_id.to_string(),
                submission_times: Vec::new(),
                avg_time_between: None,
            });
        pattern.submission_times.push(timestamp);
        // Calculate average time between submissions
        if pattern.submission_times.len() > 1 {
            let mut intervals = Vec::new();
            for i in 1..pattern.submission_times.len() {
                let interval = pattern.submission_times[i] - pattern.submission_times[i - 1];
                intervals.push(interval);
            }
            let total: chrono::Duration = intervals.iter().sum();
            pattern.avg_time_between = Some(total / intervals.len() as i32);
        }
        // Check for suspicious patterns
        let mut suspicious_indicators = Vec::new();
        // Rapid submissions
        if let Some(avg) = pattern.avg_time_between {
            if avg < chrono::Duration::seconds(30) {
                suspicious_indicators.push("Rapid successive submissions".to_string());
            }
        }
        // Late night pattern
        let late_night_count = pattern
            .submission_times
            .iter()
            .filter(|t| {
                let hour = t.hour();
                (2..=5).contains(&hour)
            })
            .count();
        if late_night_count > pattern.submission_times.len() / 2 {
            suspicious_indicators.push("Unusual late-night submission pattern".to_string());
        }
        PatternAnalysis {
            is_suspicious: !suspicious_indicators.is_empty(),
            indicators: suspicious_indicators,
            submission_count: pattern.submission_times.len(),
        }
    }
}
#[derive(Debug)]
pub struct PatternAnalysis {
    pub is_suspicious: bool,
    pub indicators: Vec<String>,
    pub submission_count: usize,
}
