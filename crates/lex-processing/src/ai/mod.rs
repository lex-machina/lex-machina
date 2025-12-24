//! AI module for LLM-powered preprocessing decisions.
//!
//! This module provides a trait-based abstraction for AI providers,
//! allowing the preprocessing pipeline to work with multiple LLM backends.
//!
//! # Feature Flag
//!
//! This module requires the `ai` feature flag to be enabled for the concrete
//! provider implementations. The [`AIProvider`] trait is always available
//! for custom implementations.
//!
//! ```toml
//! # Enable AI support (default)
//! lex_processing = { version = "0.1", features = ["ai"] }
//!
//! # Disable AI support for smaller binary
//! lex_processing = { version = "0.1", default-features = false }
//! ```
//!
//! # Architecture
//!
//! The module is built around the [`AIProvider`] trait, which defines the
//! interface for making preprocessing decisions. Concrete implementations
//! are provided for specific AI services:
//!
//! - [`GeminiProvider`] - Google Gemini API (requires `ai` feature)
//! - [`OpenRouterProvider`] - OpenRouter API (requires `ai` feature)
//!
//! # Adding a New Provider
//!
//! To add support for a new AI provider:
//!
//! 1. Create a new file (e.g., `src/ai/openai.rs`)
//! 2. Implement the [`AIProvider`] trait
//! 3. Export the new provider in this module
//!
//! # Example
//!
//! ```rust,ignore
//! use lex_processing::ai::{AIProvider, GeminiProvider};
//! use lex_processing::Pipeline;
//! use std::sync::Arc;
//!
//! // Create a provider
//! let provider = Arc::new(GeminiProvider::new("your-api-key")?);
//!
//! // Use it with the pipeline
//! let result = Pipeline::builder()
//!     .ai_provider(provider)
//!     .build()?
//!     .process(dataframe);
//! ```

// Provider trait is always available (for custom implementations)
mod provider;
pub use provider::AIProvider;

// Concrete providers require the "ai" feature
#[cfg(feature = "ai")]
mod gemini;
#[cfg(feature = "ai")]
mod openrouter;

#[cfg(feature = "ai")]
pub use gemini::{GeminiConfig, GeminiConfigBuilder, GeminiProvider};

#[cfg(feature = "ai")]
pub use openrouter::{OpenRouterConfig, OpenRouterConfigBuilder, OpenRouterProvider};
