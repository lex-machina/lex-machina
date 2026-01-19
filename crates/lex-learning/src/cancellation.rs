//! Cancellation token for stopping long-running operations.
//!
//! This module provides [`CancellationToken`], a thread-safe mechanism for
//! signalling that an operation should be cancelled. It's used primarily
//! for cancelling ML training operations.
//!
//! # Example
//!
//! ```
//! use lex_learning::CancellationToken;
//!
//! let token = CancellationToken::new();
//!
//! // Check if cancelled (initially false)
//! assert!(!token.is_cancelled());
//!
//! // Request cancellation
//! token.cancel();
//!
//! // Now cancelled
//! assert!(token.is_cancelled());
//!
//! // Reset for reuse
//! token.reset();
//! assert!(!token.is_cancelled());
//! ```
//!
//! # Thread Safety
//!
//! `CancellationToken` is `Send + Sync`, allowing safe use across threads.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// A token that can be used to signal cancellation of an operation.
///
/// Created via [`CancellationToken::new()`]. The token can be shared across
/// threads using [`Clone`](Clone). Call [`cancel()`] to signal cancellation,
/// and [`is_cancelled()`] to check if cancellation has been requested.
///
/// # Example
///
/// ```rust,ignore
/// let token = CancellationToken::new();
///
/// // Share token with training thread
/// let token_clone = token.clone();
///
/// // In training thread:
/// if token_clone.is_cancelled() {
///     // Clean up and return
/// }
///
/// // In main thread:
/// token.cancel();  // Signal cancellation
/// ```
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

// Static assertions for thread safety - required for Tauri integration
// where pipeline runs on background thread but tokens are shared
static_assertions::assert_impl_all!(CancellationToken: Send, Sync);

impl CancellationToken {
    /// Verify at runtime that CancellationToken is Send + Sync (optional)
    #[allow(dead_code)]
    pub fn assert_thread_safe() {
        fn _check_send_sync<T: Send + Sync>() {}
        _check_send_sync::<CancellationToken>();
    }
}

impl CancellationToken {
    /// Creates a new cancellation token.
    ///
    /// The token starts in the non-cancelled state.
    ///
    /// # Example
    ///
    /// ```
    /// use lex_learning::CancellationToken;
    ///
    /// let token = CancellationToken::new();
    /// assert!(!token.is_cancelled());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Request cancellation of the operation.
    ///
    /// This method is thread-safe and can be called from any thread.
    /// The operation should periodically check [`is_cancelled()`] and
    /// stop processing when `true` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use lex_learning::CancellationToken;
    ///
    /// let token = CancellationToken::new();
    /// token.cancel();
    /// assert!(token.is_cancelled());
    /// ```
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    /// Check if cancellation has been requested.
    ///
    /// Returns `true` if [`cancel()`] has been called on this token
    /// or any of its clones.
    ///
    /// # Example
    ///
    /// ```
    /// use lex_learning::CancellationToken;
    ///
    /// let token = CancellationToken::new();
    /// assert!(!token.is_cancelled());
    ///
    /// token.cancel();
    /// assert!(token.is_cancelled());
    /// ```
    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Reset the token for reuse.
    ///
    /// Clears the cancellation flag, allowing the token to be reused
    /// for another operation.
    ///
    /// # Example
    ///
    /// ```
    /// use lex_learning::CancellationToken;
    ///
    /// let token = CancellationToken::new();
    /// token.cancel();
    /// assert!(token.is_cancelled());
    ///
    /// token.reset();
    /// assert!(!token.is_cancelled());
    /// ```
    pub fn reset(&self) {
        self.cancelled.store(false, Ordering::SeqCst);
    }

    /// Returns a closure that checks if cancellation has been requested.
    ///
    /// This is useful for passing to Python callbacks that expect a
    /// cancellation check function.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let token = CancellationToken::new();
    /// let check_fn = token.as_check_fn();
    ///
    /// // Use in Python callback
    /// if check_fn() {
    ///     // Handle cancellation
    /// }
    /// ```
    pub fn as_check_fn(&self) -> impl Fn() -> bool + Send + Sync + 'static {
        let cancelled = self.cancelled.clone();
        move || cancelled.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cancellation_token_default_not_cancelled() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_cancel() {
        let token = CancellationToken::new();
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_clone_shares_state() {
        let token1 = CancellationToken::new();
        let token2 = token1.clone();

        assert!(!token1.is_cancelled());
        assert!(!token2.is_cancelled());

        token1.cancel();

        assert!(token1.is_cancelled());
        assert!(token2.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_reset() {
        let token = CancellationToken::new();
        token.cancel();
        assert!(token.is_cancelled());

        token.reset();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn test_cancellation_token_as_check_fn() {
        let token = CancellationToken::new();
        let check = token.as_check_fn();

        assert!(!check());

        token.cancel();
        assert!(check());
    }

    #[test]
    fn test_cancellation_token_thread_safe() {
        use std::thread;

        let token = CancellationToken::new();
        let token_clone = token.clone();

        let handle = thread::spawn(move || {
            // Wait a bit, then cancel
            thread::sleep(std::time::Duration::from_millis(50));
            token_clone.cancel();
        });

        // Poll for cancellation
        for _ in 0..100 {
            if token.is_cancelled() {
                break;
            }
            thread::sleep(std::time::Duration::from_millis(1));
        }

        assert!(token.is_cancelled());

        handle.join().unwrap();
    }
}
