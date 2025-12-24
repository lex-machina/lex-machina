//! Google Gemini AI provider implementation.
//!
//! This module provides the [`GeminiProvider`] which implements the [`AIProvider`]
//! trait for Google's Gemini API (<https://ai.google.dev/>).
//!
//! Gemini provides powerful multimodal AI capabilities, making it a flexible
//! choice for AI-powered preprocessing decisions.

use std::time::Duration;

use crate::types::DecisionQuestion;

use super::AIProvider;
use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Default Gemini API endpoint.
const DEFAULT_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models/";

/// Default model to use for preprocessing decisions.
const DEFAULT_MODEL: &str = "gemini-flash-lite-latest";

/// Default timeout for API requests in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default temperature for model responses (low for deterministic outputs).
const DEFAULT_TEMPERATURE: f32 = 0.1;

/// Default max tokens for responses.
const DEFAULT_MAX_TOKENS: u32 = 1000;

// Gemini API request structures
#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Serialize, Deserialize)]
struct Content {
    role: String,
    parts: Vec<Part>,
}

#[derive(Serialize, Deserialize)]
struct Part {
    text: String,
}

#[derive(Serialize)]
struct GenerationConfig {
    temperature: f32,
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: u32,
}

// Gemini API response structures
#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<Candidate>>,
}

#[derive(Deserialize)]
struct Candidate {
    content: Option<CandidateContent>,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct CandidateContent {
    parts: Option<Vec<Part>>,
}

/// Configuration for the Gemini provider.
#[derive(Debug, Clone)]
pub struct GeminiConfig {
    /// The model to use (e.g., "gemini-2.0-flash", "gemini-flash-lite-latest").
    pub model: String,
    /// Temperature for response generation (0.0 - 2.0).
    pub temperature: f32,
    /// Maximum tokens in the response.
    pub max_tokens: u32,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Base URL for the API (useful for proxies or custom endpoints).
    pub base_url: String,
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self {
            model: DEFAULT_MODEL.to_owned(),
            temperature: DEFAULT_TEMPERATURE,
            max_tokens: DEFAULT_MAX_TOKENS,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            base_url: DEFAULT_BASE_URL.to_owned(),
        }
    }
}

impl GeminiConfig {
    /// Create a new configuration builder.
    pub fn builder() -> GeminiConfigBuilder {
        GeminiConfigBuilder::default()
    }
}

/// Builder for [`GeminiConfig`].
#[derive(Default)]
pub struct GeminiConfigBuilder {
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    timeout_secs: Option<u64>,
    base_url: Option<String>,
}

impl GeminiConfigBuilder {
    /// Set the model to use.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the temperature (0.0 - 2.0).
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Set the maximum tokens.
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set the request timeout in seconds.
    pub fn timeout_secs(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = Some(timeout_secs);
        self
    }

    /// Set a custom base URL.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    /// Build the configuration.
    pub fn build(self) -> GeminiConfig {
        GeminiConfig {
            model: self.model.unwrap_or_else(|| DEFAULT_MODEL.to_owned()),
            temperature: self.temperature.unwrap_or(DEFAULT_TEMPERATURE),
            max_tokens: self.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            timeout_secs: self.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS),
            base_url: self.base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_owned()),
        }
    }
}

/// Google Gemini AI provider for making preprocessing decisions.
///
/// This provider uses Google's Gemini API for intelligent data
/// preprocessing decisions.
///
/// # Example
///
/// ```rust,ignore
/// use lex_processing::ai::{GeminiProvider, GeminiConfig};
///
/// // Simple usage with defaults
/// let provider = GeminiProvider::new("your-api-key")?;
///
/// // With custom configuration
/// let config = GeminiConfig::builder()
///     .model("gemini-2.0-flash")
///     .temperature(0.2)
///     .build();
/// let provider = GeminiProvider::with_config("your-api-key", config)?;
/// ```
pub struct GeminiProvider {
    api_key: String,
    config: GeminiConfig,
    client: Client,
}

impl GeminiProvider {
    /// Create a new Gemini provider with default configuration.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Google AI API key
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(api_key, GeminiConfig::default())
    }

    /// Create a new Gemini provider with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Google AI API key
    /// * `config` - Custom configuration options
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn with_config(api_key: impl Into<String>, config: GeminiConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| anyhow!("Failed to build HTTP client: {}", e))?;

        Ok(Self {
            api_key: api_key.into(),
            config,
            client,
        })
    }

    fn build_decision_prompt(&self, question: &DecisionQuestion) -> String {
        let mut prompt = format!(
            "You are a data scientist. Analyze this data and make a decision.\n\n\
            TASK: {}\n\
            DESCRIPTION: {}\n\n\
            SAMPLE DATA:\n{}\n\n\
            AVAILABLE OPTIONS (return the exact value only):\n",
            question.issue_type, question.description, question.sample_data
        );

        for option in &question.options {
            prompt.push_str(&format!("- {}: {}\n", option.option, option.description));
        }

        let valid_values: Vec<&str> = question.options.iter().map(|o| o.option.as_str()).collect();
        prompt.push_str(&format!(
            "\nCRITICAL: You MUST return ONLY the exact value from the options above.\n\
            Do NOT add brackets, quotes, or any other text.\n\
            Do NOT return the option number or description.\n\n\
            Example: if you choose 'classification', return exactly: classification\n\n\
            VALID VALUES: {:?}\n\n\
            YOUR DECISION (exact value only): ",
            valid_values
        ));

        prompt
    }

    fn call_api(&self, prompt: &str) -> Result<String> {
        let request = GeminiRequest {
            contents: vec![Content {
                role: "user".to_owned(),
                parts: vec![Part {
                    text: prompt.to_owned(),
                }],
            }],
            generation_config: GenerationConfig {
                temperature: self.config.temperature,
                max_output_tokens: self.config.max_tokens,
            },
        };

        // Build URL: {base_url}{model}:generateContent?key={api_key}
        let url = format!(
            "{}{}:generateContent?key={}",
            self.config.base_url, self.config.model, self.api_key
        );

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "Gemini API error {}: {}",
                response.status(),
                response.text()?
            ));
        }

        let result: GeminiResponse = response.json()?;

        // Extract text from the first candidate's content parts
        // Handle optional fields gracefully - Gemini may return empty responses
        // or responses blocked by safety filters
        let text = result
            .candidates
            .as_ref()
            .and_then(|candidates| candidates.first())
            .and_then(|c| {
                // Check if response was blocked
                if let Some(reason) = &c.finish_reason
                    && (reason == "SAFETY" || reason == "BLOCKED") {
                        return None;
                    }
                c.content.as_ref()
            })
            .and_then(|content| content.parts.as_ref())
            .and_then(|parts| parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| anyhow!("No response content from Gemini API"))?;

        Ok(text)
    }

    fn extract_decision(&self, response: &str, question: &DecisionQuestion) -> Result<String> {
        let decision = response.trim().replace(['[', ']', '"', '\''], "");

        let valid_options: Vec<&str> = question.options.iter().map(|o| o.option.as_str()).collect();

        // Direct exact match (case insensitive)
        for opt in &valid_options {
            if decision.eq_ignore_ascii_case(opt) {
                return Ok(opt.to_string());
            }
        }

        // Check if response contains a valid option
        for opt in &valid_options {
            if decision.to_lowercase().contains(&opt.to_lowercase()) {
                return Ok(opt.to_string());
            }
        }

        // For problem_type_selection, handle special cases
        if question.issue_type == "problem_type_selection" {
            if decision.to_lowercase().contains("classification") {
                return Ok("classification".to_string());
            } else if decision.to_lowercase().contains("regression") {
                return Ok("regression".to_string());
            }
        }

        warn!("AI returned '{}', using rule-based selection", response);
        Ok(self.rule_based_fallback(question))
    }

    fn rule_based_fallback(&self, question: &DecisionQuestion) -> String {
        let valid_options: Vec<&str> = question.options.iter().map(|o| o.option.as_str()).collect();

        if question.issue_type == "target_column_selection" {
            let mut scored_options: Vec<(String, i32)> = Vec::new();

            for opt in &question.options {
                let mut score = 0i32;
                let opt_lower = opt.option.to_lowercase();
                let desc_lower = opt.description.to_lowercase();

                // Positive indicators for target columns
                let target_words = ["target", "label", "class", "outcome", "result"];
                if target_words.iter().any(|w| opt_lower.contains(w)) {
                    score += 10;
                }
                if target_words.iter().any(|w| desc_lower.contains(w)) {
                    score += 8;
                }
                if desc_lower.contains("binary") {
                    score += 5;
                }
                if desc_lower.contains("categorical") && desc_lower.contains("unique: 2") {
                    score += 7;
                }

                // Negative indicators (avoid these)
                let id_words = ["id", "key", "index", "number", "code", "ref"];
                if id_words.iter().any(|w| opt_lower.contains(w)) {
                    score -= 15;
                }
                if desc_lower.contains("identifier") {
                    score -= 12;
                }
                if desc_lower.contains("missing: 100") {
                    score -= 20;
                }

                scored_options.push((opt.option.clone(), score));
            }

            scored_options.sort_by(|a, b| b.1.cmp(&a.1));

            if !scored_options.is_empty() && scored_options[0].1 > 0 {
                return scored_options[0].0.clone();
            }
        }

        // Fallback to first option
        valid_options.first().unwrap_or(&"").to_string()
    }
}

impl AIProvider for GeminiProvider {
    fn make_preprocessing_decision(&self, question: &DecisionQuestion) -> Result<String> {
        let prompt = self.build_decision_prompt(question);

        match self.call_api(&prompt) {
            Ok(response) => {
                let decision = self.extract_decision(&response, question)?;
                Ok(decision)
            }
            Err(e) => {
                warn!("AI decision failed: {}", e);
                Ok(self.rule_based_fallback(question))
            }
        }
    }

    fn name(&self) -> &str {
        "Gemini"
    }

    fn model(&self) -> Option<&str> {
        Some(&self.config.model)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SolutionOption;
    use std::collections::HashMap;

    // -------------------------------------------------------------------------
    // Helper functions
    // -------------------------------------------------------------------------

    fn create_test_question(issue_type: &str, options: Vec<(&str, &str)>) -> DecisionQuestion {
        DecisionQuestion {
            id: "test-question".to_string(),
            issue_type: issue_type.to_string(),
            description: "Test description".to_string(),
            business_impact: "Test impact".to_string(),
            detection_details: HashMap::new(),
            affected_columns: vec!["test_column".to_string()],
            options: options
                .into_iter()
                .map(|(opt, desc)| SolutionOption {
                    option: opt.to_string(),
                    description: desc.to_string(),
                    pros: None,
                    cons: None,
                    best_for: None,
                })
                .collect(),
            sample_data: "sample,data\n1,2\n3,4".to_string(),
        }
    }

    fn create_classification_question() -> DecisionQuestion {
        create_test_question(
            "problem_type_selection",
            vec![
                ("classification", "Binary or multi-class classification"),
                ("regression", "Predict continuous values"),
            ],
        )
    }

    fn create_target_column_question() -> DecisionQuestion {
        create_test_question(
            "target_column_selection",
            vec![
                ("PassengerId", "Identifier column, high uniqueness"),
                ("Survived", "Binary target column, unique: 2"),
                ("Age", "Numeric feature, missing: 20%"),
            ],
        )
    }

    fn create_imputation_question() -> DecisionQuestion {
        create_test_question(
            "imputation_strategy",
            vec![
                ("median", "Use median value for imputation"),
                ("mean", "Use mean value for imputation"),
                ("knn", "Use K-nearest neighbors imputation"),
                ("drop", "Drop rows with missing values"),
            ],
        )
    }

    // -------------------------------------------------------------------------
    // GeminiResponse parsing tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_valid_response_structure() {
        // Test that we can deserialize a valid Gemini response
        let json = r#"{
            "candidates": [{
                "content": {
                    "parts": [{"text": "classification"}]
                },
                "finishReason": "STOP"
            }]
        }"#;

        let response: GeminiResponse = serde_json::from_str(json).unwrap();
        assert!(response.candidates.is_some());
        let candidates = response.candidates.unwrap();
        assert_eq!(candidates.len(), 1);
        assert!(candidates[0].content.is_some());
        let content = candidates[0].content.as_ref().unwrap();
        assert!(content.parts.is_some());
        let parts = content.parts.as_ref().unwrap();
        assert_eq!(parts[0].text, "classification");
    }

    #[test]
    fn test_parse_response_with_empty_candidates() {
        let json = r#"{"candidates": []}"#;

        let response: GeminiResponse = serde_json::from_str(json).unwrap();
        assert!(response.candidates.is_some());
        assert!(response.candidates.unwrap().is_empty());
    }

    #[test]
    fn test_parse_response_with_null_candidates() {
        let json = r#"{"candidates": null}"#;

        let response: GeminiResponse = serde_json::from_str(json).unwrap();
        assert!(response.candidates.is_none());
    }

    #[test]
    fn test_parse_response_missing_content() {
        let json = r#"{"candidates": [{"content": null, "finishReason": "STOP"}]}"#;

        let response: GeminiResponse = serde_json::from_str(json).unwrap();
        assert!(response.candidates.is_some());
        let candidates = response.candidates.unwrap();
        assert!(candidates[0].content.is_none());
    }

    #[test]
    fn test_parse_response_missing_parts() {
        let json = r#"{"candidates": [{"content": {"parts": null}, "finishReason": "STOP"}]}"#;

        let response: GeminiResponse = serde_json::from_str(json).unwrap();
        let candidates = response.candidates.unwrap();
        let content = candidates[0].content.as_ref().unwrap();
        assert!(content.parts.is_none());
    }

    #[test]
    fn test_parse_response_safety_blocked() {
        let json = r#"{"candidates": [{"content": null, "finishReason": "SAFETY"}]}"#;

        let response: GeminiResponse = serde_json::from_str(json).unwrap();
        let candidates = response.candidates.unwrap();
        assert_eq!(candidates[0].finish_reason.as_deref(), Some("SAFETY"));
    }

    #[test]
    fn test_parse_malformed_json() {
        let json = r#"{"candidates": "not an array"}"#;

        let result: Result<GeminiResponse, _> = serde_json::from_str(json);
        // This should fail because candidates should be an array
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_response_multiple_parts() {
        let json = r#"{
            "candidates": [{
                "content": {
                    "parts": [
                        {"text": "First part"},
                        {"text": "Second part"}
                    ]
                },
                "finishReason": "STOP"
            }]
        }"#;

        let response: GeminiResponse = serde_json::from_str(json).unwrap();
        let candidates = response.candidates.unwrap();
        let parts = candidates[0].content.as_ref().unwrap().parts.as_ref().unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0].text, "First part");
        assert_eq!(parts[1].text, "Second part");
    }

    // -------------------------------------------------------------------------
    // extract_decision tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_extract_decision_exact_match() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider.extract_decision("classification", &question).unwrap();
        assert_eq!(result, "classification");
    }

    #[test]
    fn test_extract_decision_case_insensitive() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider.extract_decision("CLASSIFICATION", &question).unwrap();
        assert_eq!(result, "classification");

        let result = provider.extract_decision("Regression", &question).unwrap();
        assert_eq!(result, "regression");
    }

    #[test]
    fn test_extract_decision_with_brackets() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider.extract_decision("[classification]", &question).unwrap();
        assert_eq!(result, "classification");
    }

    #[test]
    fn test_extract_decision_with_quotes() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider.extract_decision("\"classification\"", &question).unwrap();
        assert_eq!(result, "classification");

        let result = provider.extract_decision("'regression'", &question).unwrap();
        assert_eq!(result, "regression");
    }

    #[test]
    fn test_extract_decision_with_whitespace() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider.extract_decision("  classification  ", &question).unwrap();
        assert_eq!(result, "classification");
    }

    #[test]
    fn test_extract_decision_contains_option() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider
            .extract_decision("I recommend classification for this task", &question)
            .unwrap();
        assert_eq!(result, "classification");
    }

    #[test]
    fn test_extract_decision_problem_type_special_handling() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider
            .extract_decision("This is a binary classification problem", &question)
            .unwrap();
        assert_eq!(result, "classification");

        let result = provider
            .extract_decision("Use regression analysis", &question)
            .unwrap();
        assert_eq!(result, "regression");
    }

    #[test]
    fn test_extract_decision_invalid_falls_back() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider
            .extract_decision("I don't know what to choose", &question)
            .unwrap();
        // Should return first option as fallback
        assert_eq!(result, "classification");
    }

    // -------------------------------------------------------------------------
    // rule_based_fallback tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_rule_based_fallback_target_column_scoring() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_target_column_question();

        let result = provider.rule_based_fallback(&question);
        // Should select "Survived" because it has "target" indicators
        assert_eq!(result, "Survived");
    }

    #[test]
    fn test_rule_based_fallback_avoids_identifier_columns() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_test_question(
            "target_column_selection",
            vec![
                ("user_id", "Identifier column, high uniqueness"),
                ("email_key", "Primary key field"),
                ("score", "Numeric target column, categorical"),
            ],
        );

        let result = provider.rule_based_fallback(&question);
        // Should avoid id/key columns and select "score"
        assert_eq!(result, "score");
    }

    #[test]
    fn test_rule_based_fallback_non_target_question() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_imputation_question();

        let result = provider.rule_based_fallback(&question);
        // For non-target questions, should return first option
        assert_eq!(result, "median");
    }

    #[test]
    fn test_rule_based_fallback_empty_options() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_test_question("test", vec![]);

        let result = provider.rule_based_fallback(&question);
        assert_eq!(result, "");
    }

    // -------------------------------------------------------------------------
    // build_decision_prompt tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_build_decision_prompt_contains_required_parts() {
        let provider = GeminiProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let prompt = provider.build_decision_prompt(&question);

        // Should contain task type
        assert!(prompt.contains("problem_type_selection"));
        // Should contain options
        assert!(prompt.contains("classification"));
        assert!(prompt.contains("regression"));
        // Should contain sample data
        assert!(prompt.contains("sample,data"));
        // Should contain instructions
        assert!(prompt.contains("CRITICAL"));
        assert!(prompt.contains("VALID VALUES"));
    }

    // -------------------------------------------------------------------------
    // Config builder tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_config_builder_defaults() {
        let config = GeminiConfig::builder().build();

        assert_eq!(config.model, DEFAULT_MODEL);
        assert_eq!(config.temperature, DEFAULT_TEMPERATURE);
        assert_eq!(config.max_tokens, DEFAULT_MAX_TOKENS);
        assert_eq!(config.timeout_secs, DEFAULT_TIMEOUT_SECS);
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn test_config_builder_custom_values() {
        let config = GeminiConfig::builder()
            .model("gemini-2.0-flash")
            .temperature(0.5)
            .max_tokens(2000)
            .timeout_secs(60)
            .base_url("https://custom.api.com/")
            .build();

        assert_eq!(config.model, "gemini-2.0-flash");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, 2000);
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.base_url, "https://custom.api.com/");
    }

    // -------------------------------------------------------------------------
    // Provider trait implementation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_provider_name() {
        let provider = GeminiProvider::new("test-key").unwrap();
        assert_eq!(provider.name(), "Gemini");
    }

    #[test]
    fn test_provider_model() {
        let provider = GeminiProvider::new("test-key").unwrap();
        assert_eq!(provider.model(), Some(DEFAULT_MODEL));

        let config = GeminiConfig::builder()
            .model("custom-model")
            .build();
        let provider = GeminiProvider::with_config("test-key", config).unwrap();
        assert_eq!(provider.model(), Some("custom-model"));
    }
}
