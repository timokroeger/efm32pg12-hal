//! Bridges the Starter Kit virtual COM port (VCOM) on USART0 with USART3.
//! In this example we blindly forward all bytes two serial interfaces
//! interfaces: USART0RX -> USART3TX, USART3RX -> USART0TX.
//! The functionality is useful to do some quick host side testing of serial
//! device connected to the MCU (e.g. wifi module with AT command firmware)
//! without changing hardware connections.
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

use panic_rtt_target as _;
use rtt_target::rtt_init_default;

#[entry]
fn main() -> ! {
    rtt_init_default!();

    let peripherals = Peripherals::take().unwrap();
    let mut cmu = Cmu::new(peripherals.CMU);
    let gpio = Gpio::new(peripherals.GPIO, &mut cmu);

    // Enable VCOM connection on the starter kit.
    let _vcom_enable = gpio.pa5.push_pull_output(true);

    let (mut vcom_tx, mut vcom_rx) = {
        let vcom_tx_pin = gpio.pa0.push_pull_output(true);
        let vcom_rx_pin = gpio.pa1.input();
        let usart0 = Usart::new(
            peripherals.USART0,
            vcom_tx_pin,
            vcom_rx_pin,
            &Config::default(),
            &mut cmu,
        );
        usart0.split()
    };

    let (mut tx, mut rx) = {
        let tx_pin = gpio.pb6.push_pull_output(true);
        let rx_pin = gpio.pb7.input();
        let usart3 = Usart::new(
            peripherals.USART3,
            tx_pin,
            rx_pin,
            &Config::default(),
            &mut cmu,
        );
        usart3.split()
    };

    loop {
        if let Ok(b) = rx.read() {
            vcom_tx.write(b).ok();
        }
        if let Ok(b) = vcom_rx.read() {
            tx.write(b).ok();
        }
    }
}
