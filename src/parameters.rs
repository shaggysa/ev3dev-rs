use crate::enum_string::AsStr;
use crate::error::Ev3Error;
use std::fmt::{self, Display};
use std::str::FromStr;

use crate::pub_enum_str;

pub_enum_str! {
    SensorPort,

    (In1, "ev3-ports:in1"),
    (In2, "ev3-ports:in2"),
    (In3, "ev3-ports:in3"),
    (In4, "ev3-ports:in4"),
}

pub_enum_str! {
    MotorPort,

    (OutA, "ev3-ports:outA"),
    (OutB, "ev3-ports:outB"),
    (OutC, "ev3-ports:outC"),
    (OutD, "ev3-ports:outD"),
}

pub_enum_str! {
    Direction,

    (Clockwise, "normal"),
    (CounterClockwise, "inversed"),
}

pub_enum_str! {
    Stop,

    (Coast, "coast"),
    (Brake, "brake"),
    (Hold, "hold"),
}

#[derive(Debug)]
pub enum Color {
    None,
    Black,
    Blue,
    Green,
    Yellow,
    Red,
    White,
    Brown,
}

impl Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::None => "None",
                Self::Black => "Black",
                Self::Blue => "Blue",
                Self::Green => "Green",
                Self::Yellow => "Yellow",
                Self::Red => "Red",
                Self::White => "White",
                Self::Brown => "Brown",
            }
        )
    }
}

impl FromStr for Color {
    type Err = Ev3Error;
    fn from_str(s: &str) -> Result<Self, Ev3Error> {
        match s {
            "0" => Ok(Color::None),
            "1" => Ok(Color::Black),
            "2" => Ok(Color::Blue),
            "3" => Ok(Color::Green),
            "4" => Ok(Color::Yellow),
            "5" => Ok(Color::Red),
            "6" => Ok(Color::White),
            "7" => Ok(Color::Brown),
            _ => Err(Ev3Error::ParseStr {
                input: s.to_string(),
                to: "Color".to_string(),
            }),
        }
    }
}

/// a list of buttons on the EV3 beacon remote
#[derive(PartialEq, Eq, Hash, Debug)]
pub enum Button {
    RedUp,
    BlueUp,
    RedDown,
    BlueDown,
    BeaconOn,
}
