//! Lucide icons by kebab-case name.
//!
//! The catalog in dk-core names icons as strings (core has no UI dependency), so
//! this one match maps names to `lucide_dioxus` components. Add a line here when
//! a new icon name is used. Unknown names render a plain file icon.

use dioxus::prelude::*;
use lucide_dioxus as lucide;

/// Decorative icon; give the surrounding control an accessible name.
#[component]
pub fn Icon(name: &'static str, #[props(default = 20)] size: usize) -> Element {
    match name {
        // App chrome
        "layout-grid" => rsx! { lucide::LayoutGrid { size } },
        "clock" => rsx! { lucide::Clock { size } },
        "star" => rsx! { lucide::Star { size } },
        "map" => rsx! { lucide::Map { size } },
        "settings" => rsx! { lucide::Settings { size } },
        "search" => rsx! { lucide::Search { size } },
        "sun" => rsx! { lucide::Sun { size } },
        "moon" => rsx! { lucide::Moon { size } },
        "monitor" => rsx! { lucide::Monitor { size } },
        "circle-help" => rsx! { lucide::CircleQuestionMark { size } },
        "hard-drive" => rsx! { lucide::HardDrive { size } },
        "shield-check" => rsx! { lucide::ShieldCheck { size } },
        "upload" => rsx! { lucide::Upload { size } },
        "plus" => rsx! { lucide::Plus { size } },
        "chevron-right" => rsx! { lucide::ChevronRight { size } },
        "chevron-down" => rsx! { lucide::ChevronDown { size } },
        "check" => rsx! { lucide::Check { size } },
        "chevron-left" => rsx! { lucide::ChevronLeft { size } },
        "arrow-left" => rsx! { lucide::ArrowLeft { size } },
        "x" => rsx! { lucide::X { size } },
        "circle-check" => rsx! { lucide::CircleCheck { size } },
        "folder-open" => rsx! { lucide::FolderOpen { size } },
        "rotate-ccw" => rsx! { lucide::RotateCcw { size } },
        "eye" => rsx! { lucide::Eye { size } },
        "sliders-horizontal" => rsx! { lucide::SlidersHorizontal { size } },
        "arrow-right" => rsx! { lucide::ArrowRight { size } },
        "ellipsis" => rsx! { lucide::Ellipsis { size } },
        "construction" => rsx! { lucide::Construction { size } },
        "refresh-cw" => rsx! { lucide::RefreshCw { size } },
        "external-link" => rsx! { lucide::ExternalLink { size } },
        "cloud-off" => rsx! { lucide::CloudOff { size } },
        // Roadmap task status
        "square" => rsx! { lucide::Square { size } },
        "square-check" => rsx! { lucide::SquareCheck { size } },
        "circle-dashed" => rsx! { lucide::CircleDashed { size } },
        // Modules
        "layers" => rsx! { lucide::Layers { size } },
        "gauge" => rsx! { lucide::Gauge { size } },
        "file-input" => rsx! { lucide::FileInput { size } },
        "file-output" => rsx! { lucide::FileOutput { size } },
        "square-pen" => rsx! { lucide::SquarePen { size } },
        "shield" => rsx! { lucide::Shield { size } },
        "sparkles" => rsx! { lucide::Sparkles { size } },
        "scan-line" => rsx! { lucide::ScanLine { size } },
        // Tools
        "merge" => rsx! { lucide::Merge { size } },
        "split" => rsx! { lucide::Split { size } },
        "file-minus" => rsx! { lucide::FileMinus { size } },
        "scan" => rsx! { lucide::Scan { size } },
        "minimize-2" => rsx! { lucide::Minimize2 { size } },
        "wrench" => rsx! { lucide::Wrench { size } },
        "scan-text" => rsx! { lucide::ScanText { size } },
        "image" => rsx! { lucide::Image { size } },
        "file-text" => rsx! { lucide::FileText { size } },
        "presentation" => rsx! { lucide::Presentation { size } },
        "sheet" => rsx! { lucide::Sheet { size } },
        "code" => rsx! { lucide::Code { size } },
        "file-image" => rsx! { lucide::FileImage { size } },
        "file-type" => rsx! { lucide::FileType { size } },
        "archive" => rsx! { lucide::Archive { size } },
        "rotate-cw" => rsx! { lucide::RotateCw { size } },
        "list-ordered" => rsx! { lucide::ListOrdered { size } },
        "droplet" => rsx! { lucide::Droplet { size } },
        "crop" => rsx! { lucide::Crop { size } },
        "pencil" => rsx! { lucide::Pencil { size } },
        "clipboard-list" => rsx! { lucide::ClipboardList { size } },
        "lock-open" => rsx! { lucide::LockOpen { size } },
        "lock" => rsx! { lucide::Lock { size } },
        "pen-tool" => rsx! { lucide::PenTool { size } },
        "eraser" => rsx! { lucide::Eraser { size } },
        "git-compare" => rsx! { lucide::GitCompare { size } },
        "languages" => rsx! { lucide::Languages { size } },
        "file-code" => rsx! { lucide::FileCode { size } },
        "camera" => rsx! { lucide::Camera { size } },
        _ => rsx! { lucide::File { size } },
    }
}

#[cfg(test)]
mod tests {
    /// Every icon the catalog names must have an explicit arm (not the fallback).
    #[test]
    fn catalog_icons_are_mapped() {
        let src = include_str!("icon.rs");
        let names = dk_core::catalog::TOOLS
            .iter()
            .map(|t| t.icon)
            .chain(dk_core::ModuleId::ALL.iter().map(|m| m.icon()));
        for name in names {
            assert!(
                src.contains(&format!("\"{name}\" =>")),
                "icon `{name}` not mapped"
            );
        }
    }
}
