//! Clock Managened Unit (CMU) API
use crate::pac::{CMU, GPIO};

pub trait CmuExt {
    fn freeze(self) -> Cmu;
}

impl CmuExt for CMU {
    // TODO: Make this configurable
    fn freeze(self) -> Cmu {
        Cmu {
            raw: self,
        }
    }
}

pub struct Cmu {
    raw: CMU,
}

impl Cmu {
    pub fn enable_clock(&mut self, peripheral: &impl ClockControlExt) {
        peripheral.enable_clock(self);
    }
}

pub trait ClockControlExt {
    fn enable_clock(&self, clocks: &mut Cmu);
}

macro_rules! impl_clock_control_ext {
    ($type:ty, $reg:ident, $bit:ident) => {
        impl ClockControlExt for $type {
            fn enable_clock(&self, clocks: &mut Cmu) {
                clocks.raw.$reg.modify(|_, w| w.$bit().set_bit());
            }
        }
    };
}

impl_clock_control_ext!(GPIO, hfbusclken0, gpio);
