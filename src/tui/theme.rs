use ratatui::style::{Color, Style};
use serde::Deserialize;

/// Complete theme definition deserialized from TOML.
#[derive(Debug, Clone, Deserialize)]
pub struct Theme {
    pub general: General,
    pub input: Input,
    pub list: List,
    pub preview: Preview,
    pub footer: Footer,
}

#[derive(Debug, Clone, Deserialize)]
pub struct General {
    pub background: String,
    pub foreground: String,
    pub border: String,
    pub border_focused: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub prompt: String,
    pub text: String,
    pub cursor: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct List {
    pub text: String,
    pub selected_bg: String,
    pub selected_fg: String,
    pub match_fg: String,
    pub badge_missing: String,
    pub badge_sidecar: String,
    pub badge_partial: String,
    pub badge_complete: String,
    pub badge_checking: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Preview {
    pub title: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Footer {
    pub key: String,
    pub text: String,
}

fn parse_hex(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    if let Ok(v) = u32::from_str_radix(s, 16) {
        let r = ((v >> 16) & 0xFF) as u8;
        let g = ((v >> 8) & 0xFF) as u8;
        let b = (v & 0xFF) as u8;
        Color::Rgb(r, g, b)
    } else {
        Color::Reset
    }
}

impl Theme {
    /// Load the built-in default theme.
    pub fn built_in() -> Self {
        toml::from_str(include_str!("../../themes/default.toml"))
            .expect("default theme should be valid TOML")
    }

    /// Load a theme by name or path. Falls back to default on any error.
    pub fn load(name_or_path: &str) -> Self {
        // Try built-in default first
        if name_or_path == "default" {
            return Self::built_in();
        }
        // Try as an absolute or relative path
        if let Ok(content) = std::fs::read_to_string(name_or_path)
            && let Ok(theme) = toml::from_str(&content)
        {
            return theme;
        }
        // Try in config dir themes/
        if let Some(config_dir) = directories::ProjectDirs::from("xyz", "musichoarders", "cov") {
            let themedir = config_dir.config_dir().join("themes");
            let tpath = themedir.join(format!("{}.toml", name_or_path));
            if let Ok(content) = std::fs::read_to_string(&tpath)
                && let Ok(theme) = toml::from_str(&content)
            {
                return theme;
            }
        }
        Self::built_in()
    }

    // Style accessors
    pub fn border_style(&self) -> Style {
        Style::default().fg(parse_hex(&self.general.border))
    }
    pub fn border_focused_style(&self) -> Style {
        Style::default().fg(parse_hex(&self.general.border_focused))
    }
    pub fn background_style(&self) -> Style {
        Style::default().bg(parse_hex(&self.general.background))
    }
    pub fn input_prompt_style(&self) -> Style {
        Style::default().fg(parse_hex(&self.input.prompt))
    }
    pub fn input_text_style(&self) -> Style {
        Style::default().fg(parse_hex(&self.input.text))
    }
    pub fn input_cursor_style(&self) -> Style {
        Style::default().fg(parse_hex(&self.input.cursor))
    }
    pub fn list_text_style(&self) -> Style {
        Style::default().fg(parse_hex(&self.list.text))
    }
    pub fn selected_style(&self) -> Style {
        Style::default()
            .fg(parse_hex(&self.list.selected_fg))
            .bg(parse_hex(&self.list.selected_bg))
    }
    pub fn match_style(&self) -> Style {
        Style::default().fg(parse_hex(&self.list.match_fg))
    }
    pub fn badge_style(&self, badge: crate::tui::artwork::Badge) -> Style {
        let hex = match badge {
            crate::tui::artwork::Badge::Checking => &self.list.badge_checking,
            crate::tui::artwork::Badge::Missing => &self.list.badge_missing,
            crate::tui::artwork::Badge::SidecarOnly => &self.list.badge_sidecar,
            crate::tui::artwork::Badge::Partial => &self.list.badge_partial,
            crate::tui::artwork::Badge::Complete => &self.list.badge_complete,
        };
        Style::default().fg(parse_hex(hex))
    }
    pub fn preview_title_style(&self) -> Style {
        Style::default().fg(parse_hex(&self.preview.title))
    }
    pub fn preview_label_style(&self) -> Style {
        Style::default().fg(parse_hex(&self.preview.label))
    }
    pub fn preview_value_style(&self) -> Style {
        Style::default().fg(parse_hex(&self.preview.value))
    }
    pub fn footer_key_style(&self) -> Style {
        Style::default().fg(parse_hex(&self.footer.key))
    }
    pub fn footer_text_style(&self) -> Style {
        Style::default().fg(parse_hex(&self.footer.text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_parses() {
        let theme = Theme::built_in();
        assert_eq!(theme.general.background, "#101319");
        assert_eq!(theme.general.foreground, "#e8eaf0");
    }

    #[test]
    fn test_load_default() {
        let theme = Theme::load("default");
        assert_eq!(theme.general.background, "#101319");
    }

    #[test]
    fn test_parse_hex() {
        let c = parse_hex("#01acd7");
        assert_eq!(c, Color::Rgb(1, 172, 215));
    }
}
