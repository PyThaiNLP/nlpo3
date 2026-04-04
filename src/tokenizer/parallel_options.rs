// SPDX-FileCopyrightText: 2024-2026 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

/// Options for chunk-based segmentation with optional parallel execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParallelOptions {
    /// Target chunk size in bytes.
    pub chunk_size: usize,
    /// Whether parallel chunk processing is enabled.
    pub enabled: bool,
}

impl ParallelOptions {
    /// Default target chunk size used by chunk-based segmentation.
    pub const DEFAULT_CHUNK_SIZE: usize = 65_536; // 64 KB
    /// Minimum chunk size to enable parallel processing.
    pub const MIN_CHUNK_SIZE: usize = 16_384; // 16 KB
    /// Lower bound for auto-selected chunk size.
    pub const MIN_AUTO_CHUNK_SIZE: usize = 65_536; // 64 KB
    /// Upper bound for auto-selected chunk size.
    pub const MAX_AUTO_CHUNK_SIZE: usize = 1_048_576; // 1 MB

    /// Create options from an optional chunk size.
    ///
    /// If `chunk_size` is `None`, `0`, or below `MIN_CHUNK_SIZE`,
    /// parallel processing is disabled.
    pub const fn from_chunk_size(chunk_size: Option<usize>) -> Self {
        match chunk_size {
            Some(size) if size >= Self::MIN_CHUNK_SIZE => Self {
                chunk_size: size,
                enabled: true,
            },
            _ => Self {
                chunk_size: Self::DEFAULT_CHUNK_SIZE,
                enabled: false,
            },
        }
    }

    /// Returns true if parallel mode should be used for the input length.
    pub const fn should_parallelize(&self, input_len: usize) -> bool {
        self.enabled
            && input_len >= self.chunk_size
            && input_len >= Self::MIN_CHUNK_SIZE.saturating_mul(2)
    }

    /// Estimate a chunk size for the current machine and input size.
    ///
    /// Heuristic inputs:
    /// - input byte length
    /// - available CPU parallelism
    ///
    /// Memory usage is handled conservatively by limiting chunk fan-out and
    /// bounding chunk size to avoid excessive temporary allocations.
    pub fn auto_for_text(input_len: usize) -> Self {
        let workers = std::thread::available_parallelism()
            .map(|v| v.get())
            .unwrap_or(1);

        if workers <= 1 || input_len < Self::MIN_CHUNK_SIZE.saturating_mul(2) {
            return Self::from_chunk_size(None);
        }

        let target_chunks = workers.saturating_mul(4).clamp(1, 64);
        let chunk_size = input_len
            .div_ceil(target_chunks)
            .clamp(Self::MIN_AUTO_CHUNK_SIZE, Self::MAX_AUTO_CHUNK_SIZE);

        Self {
            chunk_size,
            enabled: true,
        }
    }
}

impl Default for ParallelOptions {
    fn default() -> Self {
        Self::from_chunk_size(None)
    }
}

#[cfg(test)]
mod tests {
    use super::ParallelOptions;

    #[test]
    fn test_should_parallelize_enforces_one_chunk_rule() {
        let options = ParallelOptions::from_chunk_size(Some(ParallelOptions::MIN_CHUNK_SIZE));
        assert!(options.enabled);

        let just_under = ParallelOptions::MIN_CHUNK_SIZE * 2 - 1;
        assert!(!options.should_parallelize(just_under));

        let at_threshold = ParallelOptions::MIN_CHUNK_SIZE * 2;
        assert!(options.should_parallelize(at_threshold));
    }

    #[test]
    fn test_auto_for_text_disables_small_input() {
        let options = ParallelOptions::auto_for_text(ParallelOptions::MIN_CHUNK_SIZE * 2 - 1);
        assert!(!options.enabled);
        assert_eq!(options.chunk_size, ParallelOptions::DEFAULT_CHUNK_SIZE);
    }
}
