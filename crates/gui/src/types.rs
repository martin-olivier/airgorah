//! GUI-facing type definitions and re-exports.

pub use airgorah_common::types::{AP, AttackSoftware, AttackTarget, Settings};

pub struct BruteforceCharsetParams {
    pub lowercase: bool,
    pub uppercase: bool,
    pub numbers: bool,
    pub symbols: bool,
}

pub enum BruteforceCharset {
    Params(BruteforceCharsetParams),
    Specific(String),
}
