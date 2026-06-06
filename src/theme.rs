use ratatui::style::Color;

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub border: Color,
    pub cursor: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self { border: Color::Reset, cursor: Color::Cyan }
    }
}

pub fn parse_color(s: &str) -> Result<Color, String> {
    let norm = s.trim().to_ascii_lowercase().replace('_', "-");
    match norm.as_str() {
        "" | "reset" | "default" => return Ok(Color::Reset),
        "black" => return Ok(Color::Black),
        "red" => return Ok(Color::Red),
        "green" => return Ok(Color::Green),
        "yellow" => return Ok(Color::Yellow),
        "blue" => return Ok(Color::Blue),
        "magenta" => return Ok(Color::Magenta),
        "cyan" => return Ok(Color::Cyan),
        "gray" | "grey" => return Ok(Color::Gray),
        "dark-gray" | "dark-grey" => return Ok(Color::DarkGray),
        "light-red" => return Ok(Color::LightRed),
        "light-green" => return Ok(Color::LightGreen),
        "light-yellow" => return Ok(Color::LightYellow),
        "light-blue" => return Ok(Color::LightBlue),
        "light-magenta" => return Ok(Color::LightMagenta),
        "light-cyan" => return Ok(Color::LightCyan),
        "white" => return Ok(Color::White),
        _ => {}
    }
    let hex = norm.strip_prefix('#').unwrap_or(&norm);
    if hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()) {
        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|e| e.to_string())?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|e| e.to_string())?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|e| e.to_string())?;
        return Ok(Color::Rgb(r, g, b));
    }
    Err(format!(
        "unknown color {s:?} (expected name like 'cyan'/'light-blue'/'reset' or hex '#RRGGBB')"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_named_colors_case_insensitively() {
        assert_eq!(parse_color("Cyan").unwrap(), Color::Cyan);
        assert_eq!(parse_color("LIGHT-RED").unwrap(), Color::LightRed);
        assert_eq!(parse_color("light_blue").unwrap(), Color::LightBlue);
    }

    #[test]
    fn parses_reset_aliases() {
        assert_eq!(parse_color("reset").unwrap(), Color::Reset);
        assert_eq!(parse_color("default").unwrap(), Color::Reset);
        assert_eq!(parse_color("").unwrap(), Color::Reset);
    }

    #[test]
    fn parses_hex_rgb_with_and_without_hash() {
        assert_eq!(parse_color("#ff8800").unwrap(), Color::Rgb(0xff, 0x88, 0x00));
        assert_eq!(parse_color("00ffaa").unwrap(), Color::Rgb(0x00, 0xff, 0xaa));
    }

    #[test]
    fn rejects_unknown_input() {
        assert!(parse_color("not-a-color").is_err());
        assert!(parse_color("#xyz123").is_err());
        assert!(parse_color("#fff").is_err());
    }
}
