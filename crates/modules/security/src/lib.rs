//! Security module: unlock, protect, sign, redact, compare.
//!
//! Passwords and keys must never be logged or persisted. When options carry a
//! secret, move it into `secrecy::SecretString` on entry (see docs/PLAN.md →
//! Security).
//!
//! One file per tool. See CONTRIBUTING.md → "Implementing a tool".

use std::sync::Arc;

use dk_core::Tool;

mod compare_pdf;
mod protect_pdf;
mod redact_pdf;
mod sign_pdf;
mod unlock_pdf;

pub use compare_pdf::ComparePdf;
pub use protect_pdf::ProtectPdf;
pub use redact_pdf::RedactPdf;
pub use sign_pdf::SignPdf;
pub use unlock_pdf::UnlockPdf;

/// Every tool in this module, in catalog order.
pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![
        Arc::new(UnlockPdf),
        Arc::new(ProtectPdf),
        Arc::new(SignPdf),
        Arc::new(RedactPdf),
        Arc::new(ComparePdf),
    ]
}

#[cfg(test)]
mod tests {
    use dk_core::{catalog, ModuleId};

    #[test]
    fn implements_every_catalog_tool_of_this_module() {
        let ids: Vec<_> = super::tools().iter().map(|t| t.spec().id).collect();
        let expected: Vec<_> = catalog::tools_in(ModuleId::Security).map(|t| t.id).collect();
        assert_eq!(ids, expected);
    }
}
