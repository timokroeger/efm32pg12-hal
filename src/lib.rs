#![no_std]

pub use efm32pg12_pac as pac;
use embedded_hal as hal;

pub mod cmu;
#[macro_use]
pub mod gpio;
pub mod i2c;
pub mod usart;
pub mod prelude {
    pub use crate::{
        cmu::CmuExt as _,
        gpio::GpioExt as _,
        hal::{digital::v2::*, prelude::*},
        usart::UsartExt as _,
    };
}
mod util;
