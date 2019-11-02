#![no_std]
#![no_main]

use efm32pg12_hal as hal;

extern crate panic_itm;

use cortex_m_rt::entry;
use hal::{
    gpio::{Input, Output, Pin, PushPull},
    pac::Peripherals,
    prelude::*,
};

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut cmu = peripherals.CMU;
    let gpio = peripherals.GPIO.split(&mut cmu);

    let mut led0: Pin<_, Output<PushPull>> = gpio.pf4.into();
    let mut led1: Pin<_, Output<PushPull>> = gpio.pf5.into();

    // External pull-up resistor is too weak. Touching the backside of the
    // board makes the input toggle. Enable the internal pull-up improve
    // input noise resistance.
    let btn0: Pin<_, Input> = gpio.pf6.pull_up().into();
    let btn1: Pin<_, Input> = gpio.pf7.pull_up().into();

    // Each button controls a LED.
    loop {
        if btn0.is_low().unwrap() {
            led0.set_high().unwrap();
        } else {
            led0.set_low().unwrap();
        }

        if btn1.is_low().unwrap() {
            led1.set_high().unwrap();
        } else {
            led1.set_low().unwrap();
        }
    }
}
