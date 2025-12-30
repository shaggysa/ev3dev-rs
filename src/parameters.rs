pub use ev3dev_lang_rust::motors::MotorPort;
pub use ev3dev_lang_rust::sensors::SensorPort;
pub use ev3dev_lang_rust::{Ev3Error, Ev3Result};

#[derive(PartialEq)]
pub enum Direction {
    ClockWise,
    CounterClockWise,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Stop {
    Coast,
    Brake,
    Hold
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Blue,
    Green,
    Yellow,
    Red,
    White,
    Brown,
    Orange,
    Purple,
}

impl Color {
    pub(crate) fn from_rgb(r: i32, g: i32, b: i32) -> Self {
        let r = (r as f32 / 1020.0 * 255.0) as u8;
        let g = (g as f32 / 1020.0 * 255.0) as u8;
        let b = (b as f32 / 1020.0 * 255.0) as u8;

        let colors = [
            (Color::Black, 0, 0, 0),
            (Color::Blue, 0, 0, 255),
            (Color::Green, 0, 255, 0),
            (Color::Yellow, 255, 255, 0),
            (Color::Red, 255, 0, 0),
            (Color::White, 255, 255, 255),
            (Color::Brown, 165, 42, 42),
            (Color::Orange, 255, 165, 0),
            (Color::Purple, 128, 0, 128),
        ];

        colors
            .iter()
            .min_by_key(|(_, cr, cg, cb)| {
                let dr = (*cr as i32 - r as i32).abs();
                let dg = (*cg as i32 - g as i32).abs();
                let db = (*cb as i32 - b as i32).abs();
                dr * dr + dg * dg + db * db
            })
            .map(|(color, _, _, _)| *color)
            .unwrap_or(Color::Black)
    }
}
