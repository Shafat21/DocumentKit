//! A recent-document row, shared by the home page and the Recent documents page.

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;
use dk_core::store::RecentDoc;

use super::{format_size, Icon};

/// Seconds since the Unix epoch, now.
pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let yoe = year - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// `YYYY-MM-DDTHH:MM:SSZ` → Unix seconds.
fn parse_utc(iso: &str) -> Option<i64> {
    let num = |range: std::ops::Range<usize>| iso.get(range)?.parse::<i64>().ok();
    let days = days_from_civil(num(0..4)?, num(5..7)?, num(8..10)?);
    Some(days * 86_400 + num(11..13)? * 3600 + num(14..16)? * 60 + num(17..19)?)
}

/// "Just now", "5 min ago", "3 h ago", "2 days ago", else the date.
pub fn relative_time(opened_at: &str, now: u64) -> String {
    let date = opened_at.get(..10).unwrap_or(opened_at).to_owned();
    let Some(then) = parse_utc(opened_at) else {
        return date;
    };
    let ago = (now as i64 - then).max(0);
    match ago {
        0..60 => "Just now".into(),
        60..3_600 => format!("{} min ago", ago / 60),
        3_600..86_400 => format!("{} h ago", ago / 3_600),
        86_400..172_800 => "Yesterday".into(),
        172_800..604_800 => format!("{} days ago", ago / 86_400),
        _ => date,
    }
}

/// Recency bucket for grouping: "Today" (< 24 h), "This week" (< 7 days), "Earlier".
pub fn group_label(opened_at: &str, now: u64) -> &'static str {
    match parse_utc(opened_at).map(|t| (now as i64 - t).max(0)) {
        Some(0..86_400) => "Today",
        Some(86_400..604_800) => "This week",
        _ => "Earlier",
    }
}

/// Card for the grid view: first-page preview, name, details, hover actions.
#[component]
pub fn RecentCard(
    doc: RecentDoc,
    missing: bool,
    thumb: Option<String>,
    on_remove: EventHandler<String>,
) -> Element {
    let path = PathBuf::from(&doc.path);
    let reveal_path = path.clone();
    let when = relative_time(&doc.opened_at, now_secs());
    let remove_path = doc.path.clone();

    rsx! {
        article { class: if missing { "doc-card missing" } else { "doc-card" },
            button {
                class: "doc-card-open",
                r#type: "button",
                title: "{doc.path}",
                "aria-label": "Open {doc.name}",
                disabled: missing,
                onclick: move |_| crate::os::open(&path),
                div { class: "doc-thumb",
                    if let Some(src) = &thumb {
                        img { src: "{src}", alt: "", draggable: "false" }
                    } else {
                        Icon { name: "file-text", size: 36 }
                    }
                    if missing {
                        span { class: "tag-missing doc-tag", "File not found" }
                    }
                }
            }
            div { class: "doc-info",
                p { class: "doc-name", title: "{doc.name}", "{doc.name}" }
                p { class: "doc-meta muted",
                    "{doc.pages} pages · {format_size(doc.size_bytes)}"
                }
                p { class: "doc-when muted", "{when}" }
            }
            div { class: "doc-actions",
                button {
                    class: "icon-btn sm",
                    r#type: "button",
                    "data-tip": "Show in folder",
                    "data-tip-align": "end",
                    "aria-label": "Show {doc.name} in folder",
                    disabled: missing,
                    onclick: move |_| crate::os::reveal(&reveal_path),
                    Icon { name: "folder-open", size: 15 }
                }
                button {
                    class: "icon-btn sm",
                    r#type: "button",
                    "data-tip": "Remove from list",
                    "data-tip-align": "end",
                    "aria-label": "Remove {doc.name} from recent documents",
                    onclick: move |_| on_remove.call(remove_path.clone()),
                    Icon { name: "x", size: 15 }
                }
            }
        }
    }
}

/// One document. Clicking opens it in the default app; missing files can only be removed.
#[component]
pub fn RecentRow(
    doc: RecentDoc,
    #[props(default)] missing: bool,
    on_remove: Option<EventHandler<String>>,
) -> Element {
    let path = PathBuf::from(&doc.path);
    let reveal_path = path.clone();
    let when = relative_time(&doc.opened_at, now_secs());
    let remove_path = doc.path.clone();

    rsx! {
        li { class: if missing { "recent-row missing" } else { "recent-row" },
            button {
                class: "recent-open",
                r#type: "button",
                title: "{doc.path}",
                disabled: missing,
                onclick: move |_| crate::os::open(&path),
                span { class: "file-badge", "PDF" }
                span { class: "recent-text",
                    span { class: "recent-name", "{doc.name}" }
                    span { class: "muted",
                        if missing {
                            span { class: "tag-missing", "File not found" }
                        } else {
                            "{doc.pages} pages · {format_size(doc.size_bytes)}"
                        }
                    }
                }
                span { class: "muted recent-when", "{when}" }
            }
            div { class: "recent-actions",
                button {
                    class: "icon-btn sm",
                    r#type: "button",
                    "data-tip": "Show in folder",
                    "data-tip-align": "end",
                    "aria-label": "Show {doc.name} in folder",
                    disabled: missing,
                    onclick: move |_| crate::os::reveal(&reveal_path),
                    Icon { name: "folder-open", size: 16 }
                }
                if let Some(on_remove) = on_remove {
                    button {
                        class: "icon-btn sm",
                        r#type: "button",
                        "data-tip": "Remove from list",
                        "data-tip-align": "end",
                        "aria-label": "Remove {doc.name} from recent documents",
                        onclick: move |_| on_remove.call(remove_path.clone()),
                        Icon { name: "x", size: 16 }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_utc_including_leap_day() {
        assert_eq!(parse_utc("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_utc("2000-02-29T00:00:00Z"), Some(951_782_400));
        assert_eq!(parse_utc("garbage"), None);
    }

    #[test]
    fn groups_by_recency() {
        let then = "2026-11-24T10:00:00Z";
        let t = parse_utc(then).unwrap() as u64;
        assert_eq!(group_label(then, t + 3600), "Today");
        assert_eq!(group_label(then, t + 3 * 86_400), "This week");
        assert_eq!(group_label(then, t + 30 * 86_400), "Earlier");
        assert_eq!(group_label("bad", t), "Earlier");
    }

    #[test]
    fn formats_relative_times() {
        let then = "2026-11-24T10:00:00Z";
        let t = parse_utc(then).unwrap() as u64;
        assert_eq!(relative_time(then, t + 10), "Just now");
        assert_eq!(relative_time(then, t + 300), "5 min ago");
        assert_eq!(relative_time(then, t + 3 * 3600), "3 h ago");
        assert_eq!(relative_time(then, t + 30 * 3600), "Yesterday");
        assert_eq!(relative_time(then, t + 3 * 86_400), "3 days ago");
        assert_eq!(relative_time(then, t + 30 * 86_400), "2026-11-24");
        assert_eq!(relative_time("bad", t), "bad");
    }
}
