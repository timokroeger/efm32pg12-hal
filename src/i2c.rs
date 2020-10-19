pub use embedded_error::I2cError as Error;

use crate::{
    cmu::{ClockControlExt, Cmu},
    gpio::*,
    hal::blocking::i2c::{Read, Write, WriteRead},
    pac::{i2c0::RegisterBlock, I2C0, I2C1},
};
use core::ops::Deref;
use embedded_error::ImplError;

/// I2C API
pub struct I2c<I> {
    raw: I,
}

impl<I: I2CX> I2c<I> {
    pub fn new<SCL, SDA>(
        i2c: I,
        _scl: SCL,
        _sda: SDA,
        //config: &Config,
        cmu: &mut Cmu,
    ) -> Self
    where
        SCL: PinLocation<I, SclPin>,
        SDA: PinLocation<I, SclPin>,
    {
        cmu.enable_clock(&i2c);

        let hfperclk = cmu.hfperclk();

        // Configure I2C standard mode
        assert!(hfperclk >= 2_000_000);
        let freq_scl = 100_000;
        let n_high = 4;
        let n_low = 4;
        let div = (hfperclk - (8 * freq_scl)) / ((n_high + n_low) * freq_scl) - 1;
        assert!(div < 512);

        i2c.clkdiv
            .modify(|_, w| unsafe { w.div().bits(div as u16) });

        i2c.ctrl.write(|w| w.en().set_bit());

        // Busy flag is set after reset, use the ABORT command to clear it.
        if i2c.state.read().busy().bit_is_set() {
            i2c.cmd.write(|w| w.abort().set_bit());
        }

        // Clear pendig commands and the TX buffers.
        i2c.cmd.write(|w| w.clearpc().set_bit().cleartx().set_bit());

        // Route peripheral to pins.
        i2c.routeloc0
            .write(|w| unsafe { w.sclloc().bits(SCL::LOCATION).sdaloc().bits(SDA::LOCATION) });
        i2c.routepen
            .write(|w| w.sclpen().set_bit().sdapen().set_bit());

        Self { raw: i2c }
    }

    // Waits for an ACK or NACK of address or data byte.
    fn wait_for_ack(&mut self) -> Result<(), Error> {
        loop {
            let if_ = self.raw.if_.read();
            if if_.nack().bit_is_set() {
                self.raw.ifc.write(|w| w.nack().set_bit());
                self.raw.cmd.write(|w| w.stop().set_bit());
                return Err(Error::NACK);
            }
            if if_.ack().bit_is_set() {
                self.raw.ifc.write(|w| w.ack().set_bit());
                return Ok(());
            }
        }
    }

    fn write_no_stop(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        self.raw
            .txdata
            .write(|w| unsafe { w.txdata().bits(address << 1) });
        self.raw.cmd.write(|w| w.start().set_bit());

        self.wait_for_ack()?;

        for &b in buffer {
            self.raw.txdata.write(|w| unsafe { w.txdata().bits(b) });

            self.wait_for_ack()?;
        }

        Ok(())
    }

    /// Return the raw interface to the underlying peripheral.
    pub fn release(self) -> I {
        self.raw
    }
}

impl<I: I2CX> Read for I2c<I> {
    type Error = Error;

    fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Error> {
        // Do not try to read 0 bytes. It is not possible according to the I2C
        // specification, since the slave will always start sending the first
        // byte ACK on an address. The read operation can only be stopped by
        // NACKing a received byte, i.e., minimum 1 byte.
        if buffer.is_empty() {
            return Err(Error::Impl(ImplError::InvalidConfiguration));
        }

        self.raw.cmd.write(|w| w.start().set_bit());
        self.raw
            .txdata
            .write(|w| unsafe { w.txdata().bits((address << 1) | 1) });

        self.wait_for_ack()?;

        let last_idx = buffer.len() - 1;
        for (i, b) in buffer.iter_mut().enumerate() {
            // ACK all received bytes but the last.
            // Stop the transfer by sending a NACK.
            self.raw.cmd.write(|w| {
                if i < last_idx {
                    w.ack().set_bit()
                } else {
                    w.nack().set_bit()
                }
            });

            // Wait for byte to be received.
            while self.raw.if_.read().rxdatav().bit_is_clear() {}
            *b = self.raw.rxdata.read().rxdata().bits();
        }

        self.raw.cmd.write(|w| w.stop().set_bit());

        Ok(())
    }
}

impl<I: I2CX> Write for I2c<I> {
    type Error = Error;

    fn write(&mut self, address: u8, buffer: &[u8]) -> Result<(), Error> {
        self.write_no_stop(address, buffer)?;
        self.raw.cmd.write(|w| w.stop().set_bit());
        Ok(())
    }
}

impl<I: I2CX> WriteRead for I2c<I> {
    type Error = Error;

    fn write_read(&mut self, address: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Error> {
        self.write_no_stop(address, bytes)?;
        self.read(address, buffer)?;
        Ok(())
    }
}

/// Internal trait used to implement the I2C API for PAC I2C instances.
pub trait I2CX: Deref<Target = RegisterBlock> + ClockControlExt {}
impl I2CX for I2C0 {}
impl I2CX for I2C1 {}

/// Marks a pin that can be used as I2C SCL signal.
pub struct SclPin;

impl_pin_locations!(I2C0, SclPin, Output, {
    PA1: 0,
    PA2: 1,
    PA3: 2,
    PA4: 3,
    PA5: 4,
    PB11: 5,
    PB12: 6,
    PB13: 7,
    PB14: 8,
    PB15: 9,
    PC6: 10,
    PC7: 11,
    PC8: 12,
    PC9: 13,
    PC10: 14,
    PC11: 15,
    PD9: 16,
    PD10: 17,
    PD11: 18,
    PD12: 19,
    PD13: 20,
    PD14: 21,
    PD15: 22,
    // Overwriting debug pins is not supported
    // PF0: 23,
    // PF1: 24,
    // PF2: 25,
    // PF3: 26,
    PF4: 27,
    PF5: 28,
    PF6: 29,
    PF7: 30,
    PA0: 31,
});

impl_pin_locations!(I2C1, SclPin, Output, {
    PA7: 0,
    PA8: 1,
    PA9: 2,
    PI2: 3,
    PI3: 4,
    PB6: 5,
    PB7: 6,
    PB8: 7,
    PB9: 8,
    PB10: 9,
    PJ14: 10,
    PJ15: 11,
    PC0: 12,
    PC1: 13,
    PC2: 14,
    PC3: 15,
    PC4: 16,
    PC5: 17,
    PC10: 18,
    PC11: 19,
    PF8: 20,
    PF9: 21,
    PF10: 22,
    PF11: 23,
    PF12: 24,
    PF13: 25,
    PF14: 26,
    PF15: 27,
    PK0: 28,
    PK1: 29,
    PK2: 30,
    PA6: 31,
});

/// Marks a pin that can be used as I2C SDA signal.
pub struct SdaPin;

impl_pin_locations!(I2C0, SdaPin, Output, {
    PA0: 0,
    PA1: 1,
    PA2: 2,
    PA3: 3,
    PA4: 4,
    PA5: 5,
    PB11: 6,
    PB12: 7,
    PB13: 8,
    PB14: 9,
    PB15: 10,
    PC6: 11,
    PC7: 12,
    PC8: 13,
    PC9: 14,
    PC10: 15,
    PC11: 16,
    PD9: 17,
    PD10: 18,
    PD11: 19,
    PD12: 20,
    PD13: 21,
    PD14: 22,
    PD15: 23,
    // Overwriting debug pins is not supported
    // PF0: 24,
    // PF1: 25,
    // PF2: 26,
    // PF3: 27,
    PF4: 28,
    PF5: 29,
    PF6: 30,
    PF7: 31,
});

impl_pin_locations!(I2C1, SdaPin, Output, {
    PA6: 0,
    PA7: 1,
    PA8: 2,
    PA9: 3,
    PI2: 4,
    PI3: 5,
    PB6: 6,
    PB7: 7,
    PB8: 8,
    PB9: 9,
    PB10: 10,
    PJ14: 11,
    PJ15: 12,
    PC0: 13,
    PC1: 14,
    PC2: 15,
    PC3: 16,
    PC4: 17,
    PC5: 18,
    PC10: 19,
    PC11: 20,
    PF8: 21,
    PF9: 22,
    PF10: 23,
    PF11: 24,
    PF12: 25,
    PF13: 26,
    PF14: 27,
    PF15: 28,
    PK0: 29,
    PK1: 30,
    PK2: 31,
});
