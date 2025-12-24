//! AI provider trait for abstracting LLM interactions.
//!
//! This module defines the [`AIProvider`] trait that enables support for
//! multiple AI providers (OpenRouter, OpenAI, Anthropic, Ollama, etc.)
//! without changing the core pipeline logic.
//!
//! # Implementing a New Provider
//!
//! To add a new AI provider:
//!
//! 1. Create a new file in `src/ai/` (e.g., `openai.rs`)
//! 2. Implement the [`AIProvider`] trait for your provider struct
//! 3. Export the provider in `src/ai/mod.rs`
//!
//! # Example
//!
//! ```rust,ignore
//! use lex_processing::ai::{AIProvider, OpenRouterProvider};
//! use lex_processing::types::DecisionQuestion;
//!
//! // Create a provider
//! let provider = OpenRouterProvider::new("your-api-key".to_string())?;
//!
//! // Use it with the pipeline
//! let pipeline = Pipeline::builder()
//!     .ai_provider(&provider)
//!     .build()?;
//! ```

use crate::types::DecisionQuestion;
use anyhow::Result;

/// Trait for AI providers that can make preprocessing decisions.
///
/// This trait abstracts the interaction with various LLM providers,
/// allowing the preprocessing pipeline to work with any AI backend.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to allow usage across threads.
///
/// # Error Handling
///
/// Implementations should return meaningful errors via `anyhow::Result`.
/// The pipeline will fall back to rule-based decisions if AI fails.
pub trait AIProvider: Send + Sync {
    /// Make a preprocessing decision based on a question.
    ///
    /// The implementation should:
    /// 1. Build an appropriate prompt from the question
    /// 2. Call the AI provider's API
    /// 3. Parse and validate the response
    /// 4. Return one of the valid options from `question.options`
    ///
    /// # Arguments
    ///
    /// * `question` - The decision question containing context, options, and sample data
    ///
    /// # Returns
    ///
    /// A `Result<String>` containing the chosen option value (must match one of `question.options`)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API call fails
    /// - The response cannot be parsed
    /// - The AI returns an invalid option (implementation may choose to handle this gracefully)
    fn make_preprocessing_decision(&self, question: &DecisionQuestion) -> Result<String>;

    /// Get the provider name for logging and debugging.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// assert_eq!(provider.name(), "OpenRouter");
    /// ```
    fn name(&self) -> &str;

    /// Get the model being used by this provider.
    ///
    /// Returns `None` if the provider doesn't expose model information.
    fn model(&self) -> Option<&str> {
        None
    }
}
