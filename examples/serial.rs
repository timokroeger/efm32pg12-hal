#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efm32pg12_hal::{pac::Peripherals, prelude::*, serial::Config};
use nb::block;
use panic_abort as _;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut cmu = peripherals.CMU.freeze();
    let gpio = peripherals.GPIO.split(&mut cmu);

    // Enable VCOM connection on the starter kit.
    let _vcom_enable = gpio.pa5.push_pull_output(true);

    // Configure the serial pins
    let tx_pin = gpio.pa0.push_pull_output(true);
    let rx_pin = gpio.pa1.input();

    // Configures the serial port with 115200bps, 8 data bits and 1 stop bit.
    // The peripheral can easily be changed to USART1.
    // For USART2 or USART3 there is an compiler error because the selected
    // pins are not supported by these peripheral instances.
    let (mut tx, mut rx) = peripherals
        .USART0
        .split(tx_pin, rx_pin, &Config::default(), &mut cmu);

    // Echo back each received byte.
    loop {
        if let Ok(b) = block!(rx.read()) {
            block!(tx.write(b)).ok();
        }
    }
}
