//! Serial API
use crate::{
    cmu::{ClockControlExt, Cmu},
    gpio::*,
    pac::{usart0::RegisterBlock, USART0, USART1, USART2, USART3},
};
use core::ops::Deref;

pub struct Usart<U: Peripheral>(U);

impl<U: Peripheral> Usart<U> {
    pub fn new<TX, RX>(usart: U, _tx: TX, _rx: RX, cmu: &mut Cmu) -> Self
    where
        TX: TxPin<U>,
        RX: RxPin<U>,
    {
        cmu.enable_clock(&usart);

        // TODO: Configure

        // Route peripheral to pins.
        usart
            .routeloc0
            .write(|w| unsafe { w.txloc().bits(TX::LOCATION).rxloc().bits(RX::LOCATION) });
        usart
            .routepen
            .write(|w| w.txpen().set_bit().rxpen().set_bit());

        Self(usart)
    }
}

pub trait Peripheral: Deref<Target = RegisterBlock> + ClockControlExt {}

impl Peripheral for USART0 {}
impl Peripheral for USART1 {}
impl Peripheral for USART2 {}
impl Peripheral for USART3 {}

macro_rules! impl_pin {
    ($PIN_TRAIT:ty, $PIN_MODE:ty, {$($PIN:ty: $loc:expr,)*}) => {
        $(
            impl $PIN_TRAIT for Pin<$PIN, $PIN_MODE> {
                const LOCATION: u8 = $loc;
            }
        )*
    }
}

pub trait TxPin<U: Peripheral> {
    const LOCATION: u8;
}

impl_pin!(TxPin<USART0>, Output, {
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

impl_pin!(TxPin<USART1>, Output, {
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

impl_pin!(TxPin<USART2>, Output, {
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

impl_pin!(TxPin<USART3>, Output, {
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

pub trait RxPin<U: Peripheral> {
    const LOCATION: u8;
}

impl_pin!(RxPin<USART0>, Input, {
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

impl_pin!(RxPin<USART1>, Input, {
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

impl_pin!(RxPin<USART2>, Input, {
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

impl_pin!(RxPin<USART3>, Input, {
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
