//! Small, stateless UI building blocks. Styling lives in assets/main.css.

mod empty_state;
pub mod error_text;
mod icon;
mod modal;
mod select;
mod toggle;
mod tool_card;

pub use empty_state::EmptyState;
pub use icon::Icon;
pub use modal::Modal;
pub use select::Select;
pub use toggle::Toggle;
pub use tool_card::ToolCard;

/// 1536 → "1.5 KB".
pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn formats_sizes() {
        assert_eq!(super::format_size(512), "512 B");
        assert_eq!(super::format_size(1536), "1.5 KB");
        assert_eq!(super::format_size(5 * 1024 * 1024), "5.0 MB");
    }
}
