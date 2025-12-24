use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    fn new(r: u8, g: u8, b: u8) -> Self {
        RGB { r, g, b }
    }

    /// Calculate Euclidean distance between two colors
    fn distance(&self, other: &RGB) -> f64 {
        let dr = (self.r as i32 - other.r as i32) as f64;
        let dg = (self.g as i32 - other.g as i32) as f64;
        let db = (self.b as i32 - other.b as i32) as f64;
        (dr * dr + dg * dg + db * db).sqrt()
    }
}

/// Available colored square emojis mapped to their approximate RGB values
fn get_color_emoji_map() -> HashMap<&'static str, RGB> {
    let mut map = HashMap::new();
    map.insert("🟥", RGB::new(255, 0, 0));      // Red (pure red for better matching)
    map.insert("🟧", RGB::new(255, 165, 0));    // Orange
    map.insert("🟨", RGB::new(255, 255, 0));    // Yellow (pure yellow)
    map.insert("🟩", RGB::new(0, 255, 0));      // Green (pure green)
    map.insert("🟦", RGB::new(0, 0, 255));      // Blue (pure blue)
    map.insert("🟪", RGB::new(128, 0, 128));    // Purple
    map.insert("🟫", RGB::new(165, 42, 42));    // Brown
    map.insert("⬛", RGB::new(0, 0, 0));        // Black
    map.insert("⬜", RGB::new(255, 255, 255));  // White
    map
}

/// Named CSS colors mapped to RGB values (subset of common colors)
fn get_named_colors() -> HashMap<&'static str, RGB> {
    let mut map = HashMap::new();
    // Reds
    map.insert("red", RGB::new(255, 0, 0));
    map.insert("darkred", RGB::new(139, 0, 0));
    map.insert("crimson", RGB::new(220, 20, 60));
    map.insert("firebrick", RGB::new(178, 34, 34));
    map.insert("maroon", RGB::new(128, 0, 0));

    // Oranges
    map.insert("orange", RGB::new(255, 165, 0));
    map.insert("darkorange", RGB::new(255, 140, 0));
    map.insert("orangered", RGB::new(255, 69, 0));
    map.insert("coral", RGB::new(255, 127, 80));
    map.insert("tomato", RGB::new(255, 99, 71));

    // Yellows
    map.insert("yellow", RGB::new(255, 255, 0));
    map.insert("gold", RGB::new(255, 215, 0));
    map.insert("khaki", RGB::new(240, 230, 140));

    // Greens
    map.insert("green", RGB::new(0, 128, 0));
    map.insert("lime", RGB::new(0, 255, 0));
    map.insert("darkgreen", RGB::new(0, 100, 0));
    map.insert("olive", RGB::new(128, 128, 0));
    map.insert("forestgreen", RGB::new(34, 139, 34));
    map.insert("seagreen", RGB::new(46, 139, 87));
    map.insert("lightgreen", RGB::new(144, 238, 144));

    // Blues
    map.insert("blue", RGB::new(0, 0, 255));
    map.insert("darkblue", RGB::new(0, 0, 139));
    map.insert("navy", RGB::new(0, 0, 128));
    map.insert("skyblue", RGB::new(135, 206, 235));
    map.insert("lightblue", RGB::new(173, 216, 230));
    map.insert("cyan", RGB::new(0, 255, 255));
    map.insert("aqua", RGB::new(0, 255, 255));
    map.insert("teal", RGB::new(0, 128, 128));

    // Purples
    map.insert("purple", RGB::new(128, 0, 128));
    map.insert("violet", RGB::new(238, 130, 238));
    map.insert("magenta", RGB::new(255, 0, 255));
    map.insert("indigo", RGB::new(75, 0, 130));
    map.insert("orchid", RGB::new(218, 112, 214));

    // Browns
    map.insert("brown", RGB::new(165, 42, 42));
    map.insert("sienna", RGB::new(160, 82, 45));
    map.insert("saddlebrown", RGB::new(139, 69, 19));
    map.insert("chocolate", RGB::new(210, 105, 30));
    map.insert("tan", RGB::new(210, 180, 140));

    // Grays
    map.insert("black", RGB::new(0, 0, 0));
    map.insert("white", RGB::new(255, 255, 255));
    map.insert("gray", RGB::new(128, 128, 128));
    map.insert("grey", RGB::new(128, 128, 128));
    map.insert("silver", RGB::new(192, 192, 192));
    map.insert("lightgray", RGB::new(211, 211, 211));
    map.insert("darkgray", RGB::new(169, 169, 169));

    // Pinks
    map.insert("pink", RGB::new(255, 192, 203));
    map.insert("hotpink", RGB::new(255, 105, 180));
    map.insert("deeppink", RGB::new(255, 20, 147));

    map
}

/// Parse a color string and return RGB values
fn parse_color(color_str: &str) -> Option<RGB> {
    let color = color_str.trim().to_lowercase();

    // Check named colors first
    if let Some(rgb) = get_named_colors().get(color.as_str()) {
        return Some(*rgb);
    }

    // Try hex color (#fff, #ffffff)
    if color.starts_with('#') {
        let hex = &color[1..];

        // 3-digit hex (#rgb)
        if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            return Some(RGB::new(r, g, b));
        }

        // 6-digit hex (#rrggbb)
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(RGB::new(r, g, b));
        }
    }

    // Try rgb() or rgba()
    if color.starts_with("rgb") {
        let start = color.find('(')?;
        let end = color.find(')')?;
        let values_str = &color[start + 1..end];

        let values: Vec<&str> = values_str.split(',').map(|s| s.trim()).collect();
        if values.len() >= 3 {
            let r = values[0].parse::<u8>().ok()?;
            let g = values[1].parse::<u8>().ok()?;
            let b = values[2].parse::<u8>().ok()?;
            return Some(RGB::new(r, g, b));
        }
    }

    None
}

/// Find the closest colored square emoji for a given color string
pub fn get_closest_color_emoji(color_str: Option<&str>) -> &'static str {
    let default_emoji = "🟦"; // Blue square as fallback

    let color_str = match color_str {
        Some(s) if !s.is_empty() => s,
        _ => return default_emoji,
    };

    let target_color = match parse_color(color_str) {
        Some(rgb) => rgb,
        None => return default_emoji,
    };

    let emoji_map = get_color_emoji_map();

    // Find the emoji with the smallest color distance
    let mut best_match = default_emoji;
    let mut min_distance = f64::MAX;

    for (emoji, rgb) in emoji_map.iter() {
        let distance = target_color.distance(rgb);
        if distance < min_distance {
            min_distance = distance;
            best_match = emoji;
        }
    }

    best_match
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_colors() {
        assert_eq!(get_closest_color_emoji(Some("#ff0000")), "🟥"); // Red
        assert_eq!(get_closest_color_emoji(Some("#00ff00")), "🟩"); // Green
        assert_eq!(get_closest_color_emoji(Some("#0000ff")), "🟦"); // Blue
        assert_eq!(get_closest_color_emoji(Some("#fff")), "⬜"); // White
        assert_eq!(get_closest_color_emoji(Some("#000")), "⬛"); // Black
    }

    #[test]
    fn test_named_colors() {
        assert_eq!(get_closest_color_emoji(Some("red")), "🟥");
        assert_eq!(get_closest_color_emoji(Some("blue")), "🟦");
        assert_eq!(get_closest_color_emoji(Some("yellow")), "🟨");
        assert_eq!(get_closest_color_emoji(Some("purple")), "🟪");
    }

    #[test]
    fn test_rgb_colors() {
        assert_eq!(get_closest_color_emoji(Some("rgb(255, 0, 0)")), "🟥"); // Red
        assert_eq!(get_closest_color_emoji(Some("rgb(0, 255, 0)")), "🟩"); // Green
        assert_eq!(get_closest_color_emoji(Some("rgba(0, 0, 255, 1)")), "🟦"); // Blue
    }

    #[test]
    fn test_invalid_colors() {
        assert_eq!(get_closest_color_emoji(Some("invalid")), "🟦"); // Default
        assert_eq!(get_closest_color_emoji(None), "🟦"); // Default
        assert_eq!(get_closest_color_emoji(Some("")), "🟦"); // Default
    }
}
