// SPDX-FileCopyrightText: 2024 PyThaiNLP Project
// SPDX-License-Identifier: Apache-2.0

//! Thai soundex algorithms for phonetic matching.

pub mod lk82;
pub mod udom83;

pub use lk82::lk82;
pub use udom83::udom83;
