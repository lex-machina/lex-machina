//! OpenRouter AI provider implementation.
//!
//! This module provides the [`OpenRouterProvider`] which implements the [`AIProvider`]
//! trait for the OpenRouter API (<https://openrouter.ai/>).
//!
//! OpenRouter provides access to multiple LLM models through a unified API,
//! making it a flexible choice for AI-powered preprocessing decisions.

use super::AIProvider;
use crate::types::DecisionQuestion;
use anyhow::{anyhow, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::warn;

/// Default OpenRouter API endpoint.
const DEFAULT_BASE_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

/// Default model to use for preprocessing decisions.
const DEFAULT_MODEL: &str = "deepseek/deepseek-chat";

/// Default timeout for API requests in seconds.
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Default temperature for model responses (low for deterministic outputs).
const DEFAULT_TEMPERATURE: f32 = 0.1;

/// Default max tokens for responses.
const DEFAULT_MAX_TOKENS: u32 = 100;

#[derive(Debug, Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenRouterResponse {
    choices: Option<Vec<Choice>>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Option<Message>,
}

/// Configuration for the OpenRouter provider.
#[derive(Debug, Clone)]
pub struct OpenRouterConfig {
    /// The model to use (e.g., "deepseek/deepseek-chat", "openai/gpt-4").
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

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            model: DEFAULT_MODEL.to_string(),
            temperature: DEFAULT_TEMPERATURE,
            max_tokens: DEFAULT_MAX_TOKENS,
            timeout_secs: DEFAULT_TIMEOUT_SECS,
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }
}

impl OpenRouterConfig {
    /// Create a new configuration builder.
    pub fn builder() -> OpenRouterConfigBuilder {
        OpenRouterConfigBuilder::default()
    }
}

/// Builder for [`OpenRouterConfig`].
#[derive(Default)]
pub struct OpenRouterConfigBuilder {
    model: Option<String>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    timeout_secs: Option<u64>,
    base_url: Option<String>,
}

impl OpenRouterConfigBuilder {
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
    pub fn build(self) -> OpenRouterConfig {
        OpenRouterConfig {
            model: self.model.unwrap_or_else(|| DEFAULT_MODEL.to_string()),
            temperature: self.temperature.unwrap_or(DEFAULT_TEMPERATURE),
            max_tokens: self.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            timeout_secs: self.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS),
            base_url: self.base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
        }
    }
}

/// OpenRouter AI provider for making preprocessing decisions.
///
/// This provider uses the OpenRouter API to access various LLM models
/// for intelligent data preprocessing decisions.
///
/// # Example
///
/// ```rust,ignore
/// use lex_processing::ai::{OpenRouterProvider, OpenRouterConfig};
///
/// // Simple usage with defaults
/// let provider = OpenRouterProvider::new("your-api-key")?;
///
/// // With custom configuration
/// let config = OpenRouterConfig::builder()
///     .model("openai/gpt-4")
///     .temperature(0.2)
///     .build();
/// let provider = OpenRouterProvider::with_config("your-api-key", config)?;
/// ```
pub struct OpenRouterProvider {
    api_key: String,
    config: OpenRouterConfig,
    client: Client,
}

impl OpenRouterProvider {
    /// Create a new OpenRouter provider with default configuration.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your OpenRouter API key
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(api_key, OpenRouterConfig::default())
    }

    /// Create a new OpenRouter provider with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your OpenRouter API key
    /// * `config` - Custom configuration options
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be created.
    pub fn with_config(api_key: impl Into<String>, config: OpenRouterConfig) -> Result<Self> {
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

        for opt in &question.options {
            prompt.push_str(&format!("- {}: {}\n", opt.option, opt.description));
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
        let request = OpenRouterRequest {
            model: self.config.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: self.config.temperature,
            max_tokens: self.config.max_tokens,
        };

        let response = self
            .client
            .post(&self.config.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/your-repo")
            .header("X-Title", "AutoML-Preprocessor")
            .json(&request)
            .send()?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "OpenRouter API Error {}: {}",
                response.status(),
                response.text()?
            ));
        }

        let result: OpenRouterResponse = response.json()?;
        
        // Extract content from the first choice's message
        // Handle optional fields gracefully
        let text = result
            .choices
            .as_ref()
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.message.as_ref())
            .map(|msg| msg.content.clone())
            .ok_or_else(|| anyhow!("No response content from OpenRouter API"))?;
        
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

impl AIProvider for OpenRouterProvider {
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
        "OpenRouter"
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
    // OpenRouterResponse parsing tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_parse_valid_response_structure() {
        // Test that we can deserialize a valid OpenRouter response
        let json = r#"{
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "classification"
                }
            }]
        }"#;

        let response: OpenRouterResponse = serde_json::from_str(json).unwrap();
        assert!(response.choices.is_some());
        let choices = response.choices.unwrap();
        assert_eq!(choices.len(), 1);
        assert!(choices[0].message.is_some());
        assert_eq!(choices[0].message.as_ref().unwrap().content, "classification");
    }

    #[test]
    fn test_parse_response_with_empty_choices() {
        let json = r#"{"choices": []}"#;

        let response: OpenRouterResponse = serde_json::from_str(json).unwrap();
        assert!(response.choices.is_some());
        assert!(response.choices.unwrap().is_empty());
    }

    #[test]
    fn test_parse_response_with_null_choices() {
        let json = r#"{"choices": null}"#;

        let response: OpenRouterResponse = serde_json::from_str(json).unwrap();
        assert!(response.choices.is_none());
    }

    #[test]
    fn test_parse_response_missing_message() {
        let json = r#"{"choices": [{"message": null}]}"#;

        let response: OpenRouterResponse = serde_json::from_str(json).unwrap();
        assert!(response.choices.is_some());
        let choices = response.choices.unwrap();
        assert!(choices[0].message.is_none());
    }

    #[test]
    fn test_parse_malformed_json() {
        let json = r#"{"choices": [{"message": "not an object"}]}"#;

        let result: Result<OpenRouterResponse, _> = serde_json::from_str(json);
        // This should fail because message should be an object, not a string
        assert!(result.is_err());
    }

    // -------------------------------------------------------------------------
    // extract_decision tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_extract_decision_exact_match() {
        let provider = OpenRouterProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider.extract_decision("classification", &question).unwrap();
        assert_eq!(result, "classification");
    }

    #[test]
    fn test_extract_decision_case_insensitive() {
        let provider = OpenRouterProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider.extract_decision("CLASSIFICATION", &question).unwrap();
        assert_eq!(result, "classification");

        let result = provider.extract_decision("Classification", &question).unwrap();
        assert_eq!(result, "classification");
    }

    #[test]
    fn test_extract_decision_with_brackets() {
        let provider = OpenRouterProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider.extract_decision("[classification]", &question).unwrap();
        assert_eq!(result, "classification");
    }

    #[test]
    fn test_extract_decision_with_quotes() {
        let provider = OpenRouterProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider.extract_decision("\"classification\"", &question).unwrap();
        assert_eq!(result, "classification");

        let result = provider.extract_decision("'regression'", &question).unwrap();
        assert_eq!(result, "regression");
    }

    #[test]
    fn test_extract_decision_with_whitespace() {
        let provider = OpenRouterProvider::new("test-key").unwrap();
        let question = create_classification_question();

        let result = provider.extract_decision("  classification  ", &question).unwrap();
        assert_eq!(result, "classification");
    }

    #[test]
    fn test_extract_decision_contains_option() {
        let provider = OpenRouterProvider::new("test-key").unwrap();
        let question = create_classification_question();

        // When response contains the option as a substring
        let result = provider
            .extract_decision("I recommend classification for this task", &question)
            .unwrap();
        assert_eq!(result, "classification");
    }

    #[test]
    fn test_extract_decision_problem_type_special_handling() {
        let provider = OpenRouterProvider::new("test-key").unwrap();
        let question = create_classification_question();

        // Test special handling for problem_type_selection
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
        let provider = OpenRouterProvider::new("test-key").unwrap();
        let question = create_classification_question();

        // Invalid response should fall back to rule-based selection
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
        let provider = OpenRouterProvider::new("test-key").unwrap();
        let question = create_target_column_question();

        let result = provider.rule_based_fallback(&question);
        // Should select "Survived" because it has "target" indicators and binary
        assert_eq!(result, "Survived");
    }

    #[test]
    fn test_rule_based_fallback_avoids_identifier_columns() {
        let provider = OpenRouterProvider::new("test-key").unwrap();
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
        let provider = OpenRouterProvider::new("test-key").unwrap();
        let question = create_imputation_question();

        let result = provider.rule_based_fallback(&question);
        // For non-target questions, should return first option
        assert_eq!(result, "median");
    }

    #[test]
    fn test_rule_based_fallback_empty_options() {
        let provider = OpenRouterProvider::new("test-key").unwrap();
        let question = create_test_question("test", vec![]);

        let result = provider.rule_based_fallback(&question);
        assert_eq!(result, "");
    }

    // -------------------------------------------------------------------------
    // build_decision_prompt tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_build_decision_prompt_contains_required_parts() {
        let provider = OpenRouterProvider::new("test-key").unwrap();
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
        let config = OpenRouterConfig::builder().build();

        assert_eq!(config.model, DEFAULT_MODEL);
        assert_eq!(config.temperature, DEFAULT_TEMPERATURE);
        assert_eq!(config.max_tokens, DEFAULT_MAX_TOKENS);
        assert_eq!(config.timeout_secs, DEFAULT_TIMEOUT_SECS);
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn test_config_builder_custom_values() {
        let config = OpenRouterConfig::builder()
            .model("openai/gpt-4")
            .temperature(0.5)
            .max_tokens(200)
            .timeout_secs(60)
            .base_url("https://custom.api.com")
            .build();

        assert_eq!(config.model, "openai/gpt-4");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, 200);
        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.base_url, "https://custom.api.com");
    }

    // -------------------------------------------------------------------------
    // Provider trait implementation tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_provider_name() {
        let provider = OpenRouterProvider::new("test-key").unwrap();
        assert_eq!(provider.name(), "OpenRouter");
    }

    #[test]
    fn test_provider_model() {
        let provider = OpenRouterProvider::new("test-key").unwrap();
        assert_eq!(provider.model(), Some(DEFAULT_MODEL));

        let config = OpenRouterConfig::builder()
            .model("custom-model")
            .build();
        let provider = OpenRouterProvider::with_config("test-key", config).unwrap();
        assert_eq!(provider.model(), Some("custom-model"));
    }
}
