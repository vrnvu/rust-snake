use crossterm::style::Color;

// Core monochrome palette
pub const BACKGROUND: Color = Color::Rgb {
    r: 18,
    g: 18,
    b: 18,
}; // Almost black
pub const SURFACE: Color = Color::Rgb {
    r: 28,
    g: 28,
    b: 28,
}; // Dark grey
pub const TEXT: Color = Color::Rgb {
    r: 255,
    g: 255,
    b: 255,
}; // Pure white
pub const INACTIVE: Color = Color::Rgb {
    r: 128,
    g: 128,
    b: 128,
}; // Medium grey

// High contrast accents
pub const SECONDARY: Color = Color::Rgb {
    r: 255,
    g: 88,
    b: 88,
}; // Vibrant red
pub const PRIMARY: Color = Color::Rgb {
    r: 88,
    g: 255,
    b: 158,
}; // Bright mint green
pub const ACCENT: Color = Color::Rgb {
    r: 88,
    g: 198,
    b: 255,
}; // Bright cyan
pub const ACTIVE: Color = Color::Rgb {
    r: 255,
    g: 255,
    b: 255,
}; // Pure white
