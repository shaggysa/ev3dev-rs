use fixed::{traits::ToFixed, types::I32F32};

pub(crate) struct Pid {
    p: I32F32,
    i: I32F32,
    d: I32F32,
}

impl Pid {
    pub(crate) fn new<Number>(p: Number, i: Number, d: Number) -> Self
    where
        Number: ToFixed,
    {
        Self {
            p: I32F32::from_num(p),
            i: I32F32::from_num(i),
            d: I32F32::from_num(d),
        }
    }
}
