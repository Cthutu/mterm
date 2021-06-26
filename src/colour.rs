/// Generate a u32 compatible with the presentation arrays from colour
/// components.
pub fn colour(r: u8, g: u8, b: u8) -> u32 {
    0xff000000u32 + ((b as u32) << 16) + ((g as u32) << 8) + (r as u32)
}

/// Basic colours for convenience.
///
/// Use into() to convert to a u32.
pub enum Colour {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl From<Colour> for u32 {
    fn from(c: Colour) -> Self {
        match c {
            Colour::Black => colour(0, 0, 0),
            Colour::Red => colour(255, 0, 0),
            Colour::Green => colour(0, 255, 0),
            Colour::Yellow => colour(255, 255, 0),
            Colour::Blue => colour(0, 0, 255),
            Colour::Magenta => colour(255, 0, 255),
            Colour::Cyan => colour(0, 255, 255),
            Colour::White => colour(255, 255, 255),
        }
    }
}
