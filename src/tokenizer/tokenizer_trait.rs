// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result as AnyResult;

/// Shared interface for all Thai word tokenizers.
///
/// The `segment` method is read-only and safe to call from multiple threads
/// concurrently. The `Send + Sync` supertraits are required so that
/// `Box<dyn Tokenizer>` and
/// `Arc<dyn Tokenizer>` can be shared across threads without additional
/// bounds at call sites.  Implementations may expose additional mutation
/// methods (for example, to add or remove words) that require `&mut self`
/// and may use copy-on-write internally (such as cloning an underlying
/// dictionary).
///
/// Each concrete tokenizer implementation may support additional options
/// via specialized methods. See the concrete type's documentation for details.
pub trait Tokenizer: Send + Sync {
    /// Segment text into words.
    fn segment(&self, text: &str) -> AnyResult<Vec<String>>;
}
