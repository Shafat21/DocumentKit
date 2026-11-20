//! Every tool implementation, collected from the module crates once at first use.

use std::sync::{Arc, LazyLock};

use dk_core::Tool;

static TOOLS: LazyLock<Vec<Arc<dyn Tool>>> = LazyLock::new(|| {
    let mut tools = Vec::with_capacity(dk_core::catalog::TOOLS.len());
    tools.extend(dk_organize::tools());
    tools.extend(dk_optimize::tools());
    tools.extend(dk_convert_to::tools());
    tools.extend(dk_convert_from::tools());
    tools.extend(dk_edit::tools());
    tools.extend(dk_security::tools());
    tools.extend(dk_intelligence::tools());
    tools.extend(dk_scan::tools());
    tools
});

/// Implementation for a catalog tool id.
#[allow(dead_code)] // First caller: the first real tool page.
pub fn find(id: &str) -> Option<Arc<dyn Tool>> {
    TOOLS.iter().find(|t| t.spec().id == id).cloned()
}

#[cfg(test)]
mod tests {
    #[test]
    fn registry_matches_catalog_exactly() {
        let registered: Vec<_> = super::TOOLS.iter().map(|t| t.spec().id).collect();
        let catalog: Vec<_> = dk_core::catalog::TOOLS.iter().map(|t| t.id).collect();
        assert_eq!(registered, catalog);
    }
}
