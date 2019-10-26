#![no_std]

pub use efm32pg12_pac as pac;
use embedded_hal as hal;

pub mod gpio;
pub mod prelude {
    pub use crate::{
        gpio::GpioExt as _,
        hal::{digital::v2::*, prelude::*},
    };
}
