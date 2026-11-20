//! Theme preference. The UI maps it to `data-theme` on the root element.

/// User's theme choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    /// Always light.
    Light,
    /// Always dark.
    Dark,
    /// Follow the operating system.
    #[default]
    System,
}

impl Theme {
    /// Settings key the theme is stored under.
    pub const SETTING_KEY: &'static str = "theme";

    /// Value stored in settings and used as `data-theme`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark => "dark",
            Theme::System => "system",
        }
    }

    /// Parse a stored value; unknown values fall back to [`Theme::System`].
    pub fn parse(value: &str) -> Self {
        match value {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::System,
        }
    }

    /// Next theme for the toggle button: Light → Dark → System → Light.
    pub const fn next(self) -> Self {
        match self {
            Theme::Light => Theme::Dark,
            Theme::Dark => Theme::System,
            Theme::System => Theme::Light,
        }
    }

    /// Label for the toggle button's accessible name.
    pub const fn label(self) -> &'static str {
        match self {
            Theme::Light => "Light theme",
            Theme::Dark => "Dark theme",
            Theme::System => "System theme",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_str() {
        for t in [Theme::Light, Theme::Dark, Theme::System] {
            assert_eq!(Theme::parse(t.as_str()), t);
        }
        assert_eq!(Theme::parse("garbage"), Theme::System);
    }

    #[test]
    fn next_cycles_all_three() {
        let t = Theme::Light;
        assert_eq!(t.next().next().next(), t);
        assert_ne!(t.next(), t);
    }
}
