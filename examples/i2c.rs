#![no_std]
#![no_main]

use core::fmt::Write;
use cortex_m_rt::entry;
use efm32pg12_hal::{i2c::I2c, pac::Peripherals, prelude::*, serial::Config as SerialConfig};
use heapless::{consts::U18, String};
use panic_abort as _;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut cmu = peripherals.CMU.freeze();
    let gpio = peripherals.GPIO.split(&mut cmu);

    let btn0 = gpio.pf6.pull_up().input();

    // Enable VCOM connection on the starter kit.
    // For more details on the serial API see the serial example.
    let _vcom_enable = gpio.pa5.push_pull_output(true);
    let tx_pin = gpio.pa0.push_pull_output(true);
    let rx_pin = gpio.pa1.input();
    let mut serial = peripherals
        .USART0
        .split(tx_pin, rx_pin, &SerialConfig::default(), &mut cmu);

    // Enable the SI7021 humidity sensor with I2C interface.
    // Uses I2C standard speed of approximately 100kHz.
    let _sensor_enable = gpio.pb10.push_pull_output(true);
    let scl = gpio.pc11.filter().open_drain_output(true);
    let sda = gpio.pc10.filter().open_drain_output(true);
    let mut i2c = I2c::new(peripherals.I2C0, scl, sda, &mut cmu);

    let mut prev_button_state = false;
    loop {
        let button_state = btn0.is_high().unwrap();
        if prev_button_state && !button_state {
            // Falling edge detected: Button was pressed.

            // Read the I2C sensor and write result to the serial.
            let addr = 0x40; // Default SI7021 address
            let cmd = [0xE5u8]; // Measure humidity
            let mut humidity_raw = [0u8; 2]; // Receive buffer
            if i2c.write_read(addr, &cmd, &mut humidity_raw).is_ok() {
                // Use formula from data sheet  to convert raw data to an
                // integer with two decimal points.
                let humidity =
                    ((12500 * i16::from_be_bytes(humidity_raw) as u32) >> 16) as i16 - 600;

                let mut line: String<U18> = String::new();
                writeln!(
                    &mut line,
                    "humidity: {}.{}%",
                    humidity / 100,
                    humidity % 100
                )
                .unwrap();
                serial.bwrite_all(line.as_bytes()).ok();
            } else {
                serial.bwrite_all(b"error\n").ok();
            }
        }
        prev_button_state = button_state;
    }
}
