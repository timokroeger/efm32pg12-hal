#![no_std]
#![no_main]

use cortex_m_rt::entry;
use efm32pg12_hal::{
    cmu::Cmu,
    gpio::Gpio,
    pac::Peripherals,
    prelude::*,
    usart::{Config, Usart},
};
use panic_halt as _;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut cmu = Cmu::new(peripherals.CMU);
    let gpio = Gpio::new(peripherals.GPIO, &mut cmu);

    // Enable VCOM connection on the starter kit.
    let _vcom_enable = gpio.pa5.push_pull_output(true);

    // Configure the serial pins
    let vcom_tx_pin = gpio.pa0.push_pull_output(true);
    let vcom_rx_pin = gpio.pa1.input();

    // Configures the serial port with 115200bps, 8 data bits and 1 stop bit.
    // The peripheral can easily be changed to USART1.
    // For USART2 or USART3 there is an compiler error because the selected
    // pins are not supported by these peripheral instances.
    let vcom = Usart::new(
        peripherals.USART0,
        vcom_tx_pin,
        vcom_rx_pin,
        &Config::default(),
        &mut cmu,
    );
    let (mut vcom_tx, mut vcom_rx) = vcom.split();

    let tx_pin = gpio.pb6.push_pull_output(true);
    let rx_pin = gpio.pb7.input();
    let usart3 = Usart::new(
        peripherals.USART3,
        tx_pin,
        rx_pin,
        &Config::default(),
        &mut cmu,
    );
    let (mut tx, mut rx) = usart3.split();

    // Echo back each received byte.
    loop {
        if let Ok(b) = rx.read() {
            vcom_tx.write(b).ok();
        }
        if let Ok(b) = vcom_rx.read() {
            tx.write(b).ok();
        }
    }
}
