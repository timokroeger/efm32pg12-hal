//! Serial API for the USART peripheral
pub use crate::pac::usart0::frame::{PARITY_A as Parity, STOPBITS_A as StopBits};
use crate::{
    cmu::{ClockControlExt, Cmu},
    gpio::*,
    hal::{
        blocking::serial::write::Default as BlockingWriteDefault,
        serial::{Read, Write},
    },
    pac::{usart0::RegisterBlock, USART0, USART1, USART2, USART3},
    util::PeripheralClearSetExt,
};
use core::{convert::Infallible, fmt, marker::PhantomData, ops::Deref};
pub use embedded_error::SerialError as Error;
use nb::{self, block};

/// Serial configuration.
///
/// Defaults to 115200bps, 8 data bits, no parity and 1 stop bit.
pub struct Config {
    /// Baudrate in bps.
    baudrate: u32,
    parity: Parity,
    stop_bits: StopBits,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            baudrate: 115200,
            parity: Parity::NONE,
            stop_bits: StopBits::ONE,
        }
    }
}

/// USART API
pub struct Usart<I> {
    raw: I,
}

impl<I> Usart<I>
where
    I: Instance,
{
    pub fn new<TX, RX>(usart: I, _tx: TX, _rx: RX, config: &Config, cmu: &mut Cmu) -> Usart<I>
    where
        TX: PinLocation<I, TxPin>,
        RX: PinLocation<I, RxPin>,
    {
        cmu.enable_clock(&usart);

        usart.frame.modify(|_, w| {
            w.parity()
                .variant(config.parity)
                .stopbits()
                .variant(config.stop_bits)
        });

        let ovs = 16;
        let clkdiv = 32 * cmu.hfperclk() / (ovs * config.baudrate) - 32;
        // TODO: Check accuracy of clock and lower OVS if it is off by too much.
        usart.clkdiv.modify(|_, w| unsafe { w.div().bits(clkdiv) });

        // Route peripheral to pins.
        usart
            .routeloc0
            .write(|w| unsafe { w.txloc().bits(TX::LOCATION).rxloc().bits(RX::LOCATION) });
        usart
            .routepen
            .write(|w| w.txpen().set_bit().rxpen().set_bit());

        Usart { raw: usart }
    }

    pub fn split(self) -> (Tx<I>, Rx<I>) {
        self.raw.cmd.write(|w| w.txen().set_bit().rxen().set_bit());
        (Tx { _priv: PhantomData }, Rx { _priv: PhantomData })
    }

    /// Return the raw interface to the underlying peripheral.
    pub fn release(self) -> I {
        self.raw
    }
}

/// Transmit part of the serial interface for a USART instance.
pub struct Tx<I> {
    _priv: PhantomData<I>,
}

impl<I: Instance> Tx<I> {
    /// Enables the `TXBL` interrupt which indicates that data can be sent with
    /// the `write()` method.
    pub fn enable_interrupt(&mut self) {
        let usart_set = unsafe { &*I::ptr_set() };
        usart_set.ien.write(|w| w.txbl().set_bit());
    }

    /// Disables the `TXBL` interrupt.
    pub fn disable_interrupt(&mut self) {
        let usart_clear = unsafe { &*I::ptr_clear() };
        usart_clear.ien.write(|w| w.txbl().set_bit());
    }
}

impl<I: Instance> Write<u8> for Tx<I> {
    type Error = Infallible;

    fn write(&mut self, word: u8) -> nb::Result<(), Self::Error> {
        let usart = unsafe { &*I::ptr() };
        if usart.status.read().txbl().bit() {
            usart.txdata.write(|w| unsafe { w.txdata().bits(word) });
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn flush(&mut self) -> nb::Result<(), Self::Error> {
        let usart = unsafe { &*I::ptr() };
        if usart.status.read().txidle().bit() {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<I: Instance> BlockingWriteDefault<u8> for Tx<I> {}

impl<I: Instance> fmt::Write for Tx<I>
where
    Self: BlockingWriteDefault<u8>,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        use embedded_hal::blocking::serial::Write;
        self.bwrite_all(s.as_bytes()).map_err(|_| fmt::Error)?;
        block!(self.flush()).map_err(|_| fmt::Error)?;
        Ok(())
    }
}

/// Receive part of the serial interface for a USART instance.
pub struct Rx<I> {
    _priv: PhantomData<I>,
}

impl<I: Instance> Rx<I> {
    /// Enables the `RXDATAV` interrupt which indicates that data was received
    /// and can be read with the `read()` method.
    pub fn enable_interrupt(&mut self) {
        let usart_set = unsafe { &*I::ptr_set() };
        usart_set.ien.write(|w| w.rxdatav().set_bit());
    }

    /// Disables the `RXDATAV` interrupt.
    pub fn disable_interrupt(&mut self) {
        let usart_clear = unsafe { &*I::ptr_clear() };
        usart_clear.ien.write(|w| w.rxdatav().set_bit());
    }
}

impl<I: Instance> Read<u8> for Rx<I> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Self::Error> {
        let usart = unsafe { &*I::ptr() };
        if usart.status.read().rxdatav().bit_is_clear() {
            return Err(nb::Error::WouldBlock);
        }

        let rxdatax = usart.rxdatax.read();
        if rxdatax.ferr().bit_is_set() {
            return Err(nb::Error::Other(Error::FrameFormat));
        }
        if rxdatax.perr().bit_is_set() {
            return Err(nb::Error::Other(Error::Parity));
        }

        Ok(rxdatax.rxdata().bits() as u8)
    }
}

/// Internal trait used to implement the serial API for PAC USART instances.
pub trait Instance:
    ClockControlExt
    + Deref<Target = RegisterBlock>
    + PeripheralClearSetExt<RegisterBlock = RegisterBlock>
{
}

impl Instance for USART0 {}
impl Instance for USART1 {}
impl Instance for USART2 {}
impl Instance for USART3 {}

/// Marks a pin that can be used as USART TX signal.
pub struct TxPin;

impl_pin_locations!(USART0, TxPin, Output, {
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

impl_pin_locations!(USART1, TxPin, Output, {
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

impl_pin_locations!(USART2, TxPin, Output, {
    PA5: 0,
    PA6: 1,
    PA7: 2,
    PA8: 3,
    PA9: 4,
    PI0: 5,
    PI1: 6,
    PI2: 7,
    PI3: 8,
    PB6: 9,
    PB7: 10,
    PB8: 11,
    PB9: 12,
    PB10: 13,
    // Overwriting debug pins is not supported
    // PF0: 14,
    // PF1: 15,
    // PF3: 16,
    PF4: 17,
    PF5: 18,
    PF6: 19,
    PF7: 20,
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

impl_pin_locations!(USART3, TxPin, Output, {
    PD8: 0,
    PD9: 1,
    PD10: 2,
    PD11: 3,
    PD12: 4,
    PD13: 5,
    PD14: 6,
    PD15: 7,
    PI2: 8,
    PI3: 9,
    PB6: 10,
    PB7: 11,
    PB8: 12,
    PB9: 13,
    PB10: 14,
    PB11: 15,
    PJ14: 16,
    PJ15: 17,
    PC0: 18,
    PC1: 19,
    PC2: 20,
    PC3: 21,
    PC4: 22,
    PC5: 23,
    PF11: 24,
    PF12: 25,
    PF13: 26,
    PF14: 27,
    PF15: 28,
    PK0: 29,
    PK1: 30,
    PK2: 31,
});

/// Marks a pin that can be used as USART RX signal.
pub struct RxPin;

impl_pin_locations!(USART0, RxPin, Input, {
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

impl_pin_locations!(USART1, RxPin, Input, {
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

impl_pin_locations!(USART2, RxPin, Input, {
    PA6: 0,
    PA7: 1,
    PA8: 2,
    PA9: 3,
    PI0: 4,
    PI1: 5,
    PI2: 6,
    PI3: 7,
    PB6: 8,
    PB7: 9,
    PB8: 10,
    PB9: 11,
    PB10: 12,
    // Overwriting debug pins is not supported
    // PF0: 13,
    // PF1: 14,
    // PF3: 15,
    PF4: 16,
    PF5: 17,
    PF6: 18,
    PF7: 19,
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
    PA5: 31,
});

impl_pin_locations!(USART3, RxPin, Input, {
    PD9: 0,
    PD10: 1,
    PD11: 2,
    PD12: 3,
    PD13: 4,
    PD14: 5,
    PD15: 6,
    PI2: 7,
    PI3: 8,
    PB6: 9,
    PB7: 10,
    PB8: 11,
    PB9: 12,
    PB10: 13,
    PB11: 14,
    PJ14: 15,
    PJ15: 16,
    PC0: 17,
    PC1: 18,
    PC2: 19,
    PC3: 20,
    PC4: 21,
    PC5: 22,
    PF11: 23,
    PF12: 24,
    PF13: 25,
    PF14: 26,
    PF15: 27,
    PK0: 28,
    PK1: 29,
    PK2: 30,
    PD8: 31,
});
