#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn with_red(self, r: f32) -> Color {
        Color { r, ..self }
    }

    pub fn with_green(self, g: f32) -> Color {
        Color { g, ..self }
    }

    pub fn with_blue(self, b: f32) -> Color {
        Color { b, ..self }
    }

    pub fn with_alpha(self, a: f32) -> Color {
        Color { a, ..self }
    }

    pub fn multiply(self, other: Color) -> Color {
        Color {
            r: self.r * other.r,
            g: self.g * other.g,
            b: self.b * other.b,
            a: self.a * other.a,
        }
    }

    pub fn from_rgba(red: u8, green: u8, blue: u8, a: f32) -> Color {
        Color {
            r: red as f32 / 255.0,
            g: green as f32 / 255.0,
            b: blue as f32 / 255.0,
            a,
        }
    }

    pub fn from_hex(hex: &str) -> Color {
        let trimmed_hex = hex.trim_start_matches('#');
        match trimmed_hex.len() {
            3 => {
                let longer_hex: Vec<String> = trimmed_hex
                    .chars()
                    .map(|single_char| single_char.to_string().repeat(2))
                    .collect();
                Color::from_hex(&longer_hex.concat())
            }
            6 => {
                let red = u8::from_str_radix(&trimmed_hex[0..=1], 16).unwrap();
                let green = u8::from_str_radix(&trimmed_hex[2..=3], 16).unwrap();
                let blue = u8::from_str_radix(&trimmed_hex[4..=5], 16).unwrap();
                Color::from_rgba(red, green, blue, 1.0)
            }
            _ => panic!("Malformed hex string"),
        }
    }
}

impl Color {
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    pub const ORANGE: Color = Color {
        r: 1.0,
        g: 0.5,
        b: 0.0,
        a: 1.0,
    };
    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.5,
        a: 1.0,
    };
    pub const PURPLE: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
    pub const INDIGO: Color = Color {
        r: 0.5,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
}
