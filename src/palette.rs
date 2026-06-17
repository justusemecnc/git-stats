use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub text: Color,
    pub muted: Color,
    pub background: Color,
    pub heatmap_empty: Color,
    pub heatmap_low: Color,
    pub heatmap_mid: Color,
    pub heatmap_high: Color,
    pub highlight: Color,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            accent: Color::Magenta,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            text: Color::White,
            muted: Color::DarkGray,
            background: Color::Black,
            heatmap_empty: Color::DarkGray,
            heatmap_low: Color::Green,
            heatmap_mid: Color::Yellow,
            heatmap_high: Color::Red,
            highlight: Color::LightYellow,
        }
    }

    pub fn light() -> Self {
        Self {
            primary: Color::Blue,
            secondary: Color::Cyan,
            accent: Color::Magenta,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            text: Color::Black,
            muted: Color::Gray,
            background: Color::White,
            heatmap_empty: Color::Gray,
            heatmap_low: Color::Green,
            heatmap_mid: Color::Yellow,
            heatmap_high: Color::Red,
            highlight: Color::LightBlue,
        }
    }

    pub fn from_theme(theme: &str) -> Self {
        match theme {
            "light" => Self::light(),
            _ => Self::dark(),
        }
    }

    pub fn heatmap_color(&self, level: u8) -> Color {
        match level {
            0 => self.heatmap_empty,
            1 => self.heatmap_low,
            2 => self.heatmap_mid,
            _ => self.heatmap_high,
        }
    }
}
