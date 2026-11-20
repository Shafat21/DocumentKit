//! Scan module: document scanner integration, auto crop & deskew.
//!
//! One file per tool. See CONTRIBUTING.md → "Implementing a tool".

use std::sync::Arc;

use dk_core::Tool;

mod deskew;
mod scanner;

pub use deskew::Deskew;
pub use scanner::Scanner;

/// Every tool in this module, in catalog order.
pub fn tools() -> Vec<Arc<dyn Tool>> {
    vec![Arc::new(Scanner), Arc::new(Deskew)]
}

#[cfg(test)]
mod tests {
    use dk_core::{catalog, ModuleId};

    #[test]
    fn implements_every_catalog_tool_of_this_module() {
        let ids: Vec<_> = super::tools().iter().map(|t| t.spec().id).collect();
        let expected: Vec<_> = catalog::tools_in(ModuleId::Scan).map(|t| t.id).collect();
        assert_eq!(ids, expected);
    }
}
