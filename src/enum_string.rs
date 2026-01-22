// Props to the author of the enum_str crate for the macro design.
// It didn't have the option for exporting the generated enum,
// so I had to add that code here manually

pub trait AsStr {
    fn as_str(&self) -> &str;
}

#[macro_export]
#[doc(hidden)]
macro_rules! enum_str {
    ($name:ident, $(($key:ident, $value:expr),)*) => {
       #[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
       enum $name
        {
            $($key),*
        }

        impl AsStr for $name {
            fn as_str(&self) -> &str {
                match self {
                    $(
                        &$name::$key => $value
                    ),*
                }
            }
        }

        impl FromStr for $name {
            type Err = Ev3Error;

            fn from_str(val: &str) -> Result<Self, Self::Err> {
                match val
                 {
                    $(
                        $value => Ok($name::$key)
                    ),*,
                    _ => Err(Ev3Error::ParseStr{input: val.to_string(), to: stringify!($name).to_string()})
                }
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! crate_enum_str {
    ($name:ident, $(($key:ident, $value:expr),)*) => {
       #[derive(Debug, PartialEq, Copy, Clone)]
       pub(crate) enum $name
        {
            $($key),*
        }

        impl AsStr for $name {
            fn as_str(&self) -> &str {
                match self {
                    $(
                        &$name::$key => $value
                    ),*
                }
            }
        }

        impl FromStr for $name {
            type Err = Ev3Error;

            fn from_str(val: &str) -> Result<Self, Self::Err> {
                match val
                 {
                    $(
                        $value => Ok($name::$key)
                    ),*,
                    _ => Err(Ev3Error::ParseStr{input: val.to_string(), to: stringify!($name).to_string()})
                }
            }
        }
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! pub_enum_str {
    ($name:ident, $(($key:ident, $value:expr),)*) => {
       #[derive(Debug, PartialEq, Copy, Clone)]
       pub enum $name
        {
            $($key),*
        }

        impl AsStr for $name {
            fn as_str(&self) -> &str {
                match self {
                    $(
                        &$name::$key => $value
                    ),*
                }
            }
        }

        impl FromStr for $name {
            type Err = Ev3Error;

            fn from_str(val: &str) -> Result<Self, Self::Err> {
                match val
                 {
                    $(
                        $value => Ok($name::$key)
                    ),*,
                    _ => Err(Ev3Error::ParseStr{input: val.to_string(), to: stringify!($name).to_string()})
                }
            }
        }
    }
}
