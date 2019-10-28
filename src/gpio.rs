//! General purpose I/O (GPIO) pin API
use crate::{
    hal::digital::v2::{InputPin, OutputPin, StatefulOutputPin, ToggleableOutputPin},
    pac::{
        // Each GPIO has it’s own enum for the mode field even though the values are
        // shared by all GPIOs. Import the first to have nice names for the numeric
        // constants and use the provided u8 conversion to erase type information.
        gpio::{pa_model::MODE0_A as MODE, RegisterBlock},
        CMU,
        GPIO,
    },
};
use core::marker::PhantomData;

/// Extension trait to use the peripheral bit set and clear feature.
trait GpioClearSetExt {
    /// Returns a pointer to the register block in aliased peripheral bit clear
    /// memory region.
    ///
    /// Allows to clear bitfields without a read-modify-write operation.
    unsafe fn ptr_clear() -> *const RegisterBlock;

    /// Returns a pointer to the register block in aliased peripheral bit set
    /// memory region.
    ///
    /// Allows to set bitfields without a read-modify-write operation.
    unsafe fn ptr_set() -> *const RegisterBlock;
}

#[allow(clippy::cast_ptr_alignment)]
impl GpioClearSetExt for GPIO {
    unsafe fn ptr_clear() -> *const RegisterBlock {
        (Self::ptr() as *const u8).offset(0x0400_0000) as *const RegisterBlock
    }

    unsafe fn ptr_set() -> *const RegisterBlock {
        (Self::ptr() as *const u8).offset(0x0600_0000) as *const RegisterBlock
    }
}

/// Extension trait to split the GPIO register block into individual GPIO pins.
pub trait GpioExt {
    /// Splits the GPIO register block into individual GPIO pins.
    fn split(self, cmu: &mut CMU) -> Parts;
}

/// Internal trait to abstract away raw register manipulation.
/// Leaked because it is used as trait bound. Not relevant for the user.
pub trait PinTrait {
    fn clear_mode(&mut self);
    fn set_mode(&mut self, mode: MODE);
    fn clear_dout_bit(&mut self);
    fn set_dout_bit(&mut self);
    fn read_dout_bit(&self) -> bool;
    fn write_douttgl_bit(&mut self);
    fn read_din_bit(&self) -> bool;
}

macro_rules! gpios {
    ($(
        $field:ident,
        $type:ident,
        $mode_reg:ident,
        $mode_field:ident,
        $dout_reg:ident,
        $douttgl_reg:ident,
        $din_reg:ident,
        $pin_nr:expr;
    )*) => {
        /// Contains a field for each individual GPIO pin.
        pub struct Parts {
            $(pub $field: PinBuilder<$type, Floating, NoFilter>,)*
        }

        impl GpioExt for GPIO {
            fn split(self, cmu: &mut CMU) -> Parts {
                cmu.hfbusclken0.modify(|_, w| w.gpio().set_bit());

                Parts {
                    $(
                        $field: PinBuilder {
                            ty: $type,
                            _pull: PhantomData,
                            _filter: PhantomData,
                        },
                    )*
                }
            }
        }

        $(
            /// Marks a specific pin.
            ///
            /// Used as trait bound by the [`Pin`] type.
            pub struct $type;

            impl PinTrait for $type {
                fn clear_mode(&mut self) {
                    let mode_clear = unsafe { &(*GPIO::ptr_clear()).$mode_reg };
                    mode_clear.write_with_zero(|w| w.$mode_field().bits(!0));
                }

                fn set_mode(&mut self, mode: MODE) {
                    let mode_set = unsafe { &(*GPIO::ptr_set()).$mode_reg };
                    mode_set.write_with_zero(|w| w.$mode_field().bits(mode.into()));
                }

                fn clear_dout_bit(&mut self) {
                    let dout_clear = unsafe { &(*GPIO::ptr_clear()).$dout_reg };
                    dout_clear.write_with_zero(|w| unsafe { w.bits(1 << $pin_nr) });
                }

                fn set_dout_bit(&mut self) {
                    let dout_set = unsafe { &(*GPIO::ptr_set()).$dout_reg };
                    dout_set.write_with_zero(|w| unsafe { w.bits(1 << $pin_nr) });
                }

                fn read_dout_bit(&self) -> bool {
                    let dout_reg = unsafe { &(*GPIO::ptr()).$dout_reg };
                    dout_reg.read().bits() & (1 << $pin_nr) == 1 << $pin_nr
                }

                fn write_douttgl_bit(&mut self) {
                    let douttgl = unsafe { &(*GPIO::ptr()).$douttgl_reg };
                    douttgl.write(|w| unsafe { w.bits(1 << $pin_nr) });
                }

                fn read_din_bit(&self) -> bool {
                    let din_reg = unsafe { &(*GPIO::ptr()).$din_reg };
                    din_reg.read().bits() & (1 << $pin_nr) == 1 << $pin_nr
                }
            }
        )*
    }
}

gpios!(
    pa0, PA0, pa_model, mode0, pa_dout, pa_douttgl, pa_din, 0;
    pa1, PA1, pa_model, mode1, pa_dout, pa_douttgl, pa_din, 1;
    pa2, PA2, pa_model, mode2, pa_dout, pa_douttgl, pa_din, 2;
    pa3, PA3, pa_model, mode3, pa_dout, pa_douttgl, pa_din, 3;
    pa4, PA4, pa_model, mode4, pa_dout, pa_douttgl, pa_din, 4;
    pa5, PA5, pa_model, mode5, pa_dout, pa_douttgl, pa_din, 5;
    pa6, PA6, pa_model, mode6, pa_dout, pa_douttgl, pa_din, 6;
    pa7, PA7, pa_model, mode7, pa_dout, pa_douttgl, pa_din, 7;
    pa8, PA8, pa_modeh, mode8, pa_dout, pa_douttgl, pa_din, 8;
    pa9, PA9, pa_modeh, mode9, pa_dout, pa_douttgl, pa_din, 9;

    pb6, PB6, pb_model, mode6, pb_dout, pb_douttgl, pb_din, 6;
    pb7, PB7, pb_model, mode7, pb_dout, pb_douttgl, pb_din, 7;
    pb8, PB8, pb_modeh, mode8, pb_dout, pb_douttgl, pb_din, 8;
    pb9, PB9, pb_modeh, mode9, pb_dout, pb_douttgl, pb_din, 9;
    pb10, PB10, pb_modeh, mode10, pb_dout, pb_douttgl, pb_din, 10;
    pb11, PB11, pb_modeh, mode11, pb_dout, pb_douttgl, pb_din, 11;
    pb12, PB12, pb_modeh, mode12, pb_dout, pb_douttgl, pb_din, 12;
    pb13, PB13, pb_modeh, mode13, pb_dout, pb_douttgl, pb_din, 13;
    pb14, PB14, pb_modeh, mode14, pb_dout, pb_douttgl, pb_din, 14;
    pb15, PB15, pb_modeh, mode15, pb_dout, pb_douttgl, pb_din, 15;

    pc0, PC0, pc_model, mode0, pc_dout, pc_douttgl, pc_din, 0;
    pc1, PC1, pc_model, mode1, pc_dout, pc_douttgl, pc_din, 1;
    pc2, PC2, pc_model, mode2, pc_dout, pc_douttgl, pc_din, 2;
    pc3, PC3, pc_model, mode3, pc_dout, pc_douttgl, pc_din, 3;
    pc4, PC4, pc_model, mode4, pc_dout, pc_douttgl, pc_din, 4;
    pc5, PC5, pc_model, mode5, pc_dout, pc_douttgl, pc_din, 5;
    pc6, PC6, pc_model, mode6, pc_dout, pc_douttgl, pc_din, 6;
    pc7, PC7, pc_model, mode7, pc_dout, pc_douttgl, pc_din, 7;
    pc8, PC8, pc_modeh, mode8, pc_dout, pc_douttgl, pc_din, 8;
    pc9, PC9, pc_modeh, mode9, pc_dout, pc_douttgl, pc_din, 9;
    pc10, PC10, pc_modeh, mode10, pc_dout, pc_douttgl, pc_din, 10;
    pc11, PC11, pc_modeh, mode11, pc_dout, pc_douttgl, pc_din, 11;

    pd8, PD8, pd_modeh, mode8, pd_dout, pd_douttgl, pd_din, 8;
    pd9, PD9, pd_modeh, mode9, pd_dout, pd_douttgl, pd_din, 9;
    pd10, PD10, pd_modeh, mode10, pd_dout, pd_douttgl, pd_din, 10;
    pd11, PD11, pd_modeh, mode11, pd_dout, pd_douttgl, pd_din, 11;
    pd12, PD12, pd_modeh, mode12, pd_dout, pd_douttgl, pd_din, 12;
    pd13, PD13, pd_modeh, mode13, pd_dout, pd_douttgl, pd_din, 13;
    pd14, PD14, pd_modeh, mode14, pd_dout, pd_douttgl, pd_din, 14;
    pd15, PD15, pd_modeh, mode15, pd_dout, pd_douttgl, pd_din, 15;

/*
    pf0, PF0, pf_model, mode0, pf_dout, pf_douttgl, pf_din, 0, Debug;
    pf1, PF1, pf_model, mode1, pf_dout, pf_douttgl, pf_din, 1, Debug;
    pf2, PF2, pf_model, mode2, pf_dout, pf_douttgl, pf_din, 2, Debug;
    pf3, PF3, pf_model, mode3, pf_dout, pf_douttgl, pf_din, 3, Debug;
*/
    pf4, PF4, pf_model, mode4, pf_dout, pf_douttgl, pf_din, 4;
    pf5, PF5, pf_model, mode5, pf_dout, pf_douttgl, pf_din, 5;
    pf6, PF6, pf_model, mode6, pf_dout, pf_douttgl, pf_din, 6;
    pf7, PF7, pf_model, mode7, pf_dout, pf_douttgl, pf_din, 7;
    pf8, PF8, pf_modeh, mode8, pf_dout, pf_douttgl, pf_din, 8;
    pf9, PF9, pf_modeh, mode9, pf_dout, pf_douttgl, pf_din, 9;
    pf10, PF10, pf_modeh, mode10, pf_dout, pf_douttgl, pf_din, 10;
    pf11, PF11, pf_modeh, mode11, pf_dout, pf_douttgl, pf_din, 11;
    pf12, PF12, pf_modeh, mode12, pf_dout, pf_douttgl, pf_din, 12;
    pf13, PF13, pf_modeh, mode13, pf_dout, pf_douttgl, pf_din, 13;
    pf14, PF14, pf_modeh, mode14, pf_dout, pf_douttgl, pf_din, 14;
    pf15, PF15, pf_modeh, mode15, pf_dout, pf_douttgl, pf_din, 15;

    pi0, PI0, pi_model, mode0, pi_dout, pi_douttgl, pi_din, 0;
    pi1, PI1, pi_model, mode1, pi_dout, pi_douttgl, pi_din, 1;
    pi2, PI2, pi_model, mode2, pi_dout, pi_douttgl, pi_din, 2;
    pi3, PI3, pi_model, mode3, pi_dout, pi_douttgl, pi_din, 3;

    pj14, PJ14, pj_modeh, mode14, pj_dout, pj_douttgl, pj_din, 14;
    pj15, PJ15, pj_modeh, mode15, pj_dout, pj_douttgl, pj_din, 15;

    pk0, PK0, pk_model, mode0, pk_dout, pk_douttgl, pk_din, 0;
    pk1, PK1, pk_model, mode1, pk_dout, pk_douttgl, pk_din, 1;
    pk2, PK2, pk_model, mode2, pk_dout, pk_douttgl, pk_din, 2;
);

// Use a private module to hide those types from the documentation.
use builder_types::*;
mod builder_types {
    /// Internal trait implemented by types that indicate a pull-up or pull-down
    /// resistor configuration.
    pub trait PullTrait {}

    /// Internal trait implemented by types that indicate a filter configuration
    /// for inputs.
    pub trait FilterTrait {}

    pub struct Floating;
    impl PullTrait for Floating {}

    pub struct PullDown;
    impl PullTrait for PullDown {}

    pub struct PullUp;
    impl PullTrait for PullUp {}

    pub struct NoFilter;
    impl FilterTrait for NoFilter {}

    pub struct Filter;
    impl FilterTrait for Filter {}
}

/// Builder type for pins.
///
/// This can be obtained by accessing a field of [`Parts`] or by calling
/// [`Pin::reset()`] on an existing pin.
pub struct PinBuilder<T: PinTrait, P: PullTrait, F: FilterTrait> {
    ty: T,
    _pull: PhantomData<P>,
    _filter: PhantomData<F>,
}

impl<T: PinTrait, P: PullTrait, F: FilterTrait> PinBuilder<T, P, F> {
    /// Disables any pull-up or pull-down resistors.
    pub fn floating(self) -> PinBuilder<T, Floating, F> {
        PinBuilder {
            ty: self.ty,
            _pull: PhantomData,
            _filter: PhantomData,
        }
    }

    /// Enables a pull-up resistor.
    pub fn pull_up(self) -> PinBuilder<T, PullUp, F> {
        PinBuilder {
            ty: self.ty,
            _pull: PhantomData,
            _filter: PhantomData,
        }
    }

    /// Enables a pull-down resistor.
    pub fn pull_pown(self) -> PinBuilder<T, PullDown, F> {
        PinBuilder {
            ty: self.ty,
            _pull: PhantomData,
            _filter: PhantomData,
        }
    }

    /// Enables the glitch filter on the input circuitry.
    pub fn filter(self) -> PinBuilder<T, P, Filter> {
        PinBuilder {
            ty: self.ty,
            _pull: PhantomData,
            _filter: PhantomData,
        }
    }

    /// Disables the glitch filter on the input circuitry.
    pub fn no_filter(self) -> PinBuilder<T, P, NoFilter> {
        PinBuilder {
            ty: self.ty,
            _pull: PhantomData,
            _filter: PhantomData,
        }
    }
}

/// Implemented by types that indicate an GPIO mode.
///
/// Used as trait bound by the [`Pin`] type.
pub trait Mode {}

/// Marks a GPIO pin as being disabled.
pub struct Disabled;
impl Mode for Disabled {}

/// Marks a GPIO pin as being used by a debug connection (SDW or JTAG).
pub struct Debug;
impl Mode for Debug {}

/// Marks a GPIO pin as being configured as input.
pub struct Input;
impl Mode for Input {}

/// Marks a GPIO pin as being configured as output.
pub struct Output<OM: OutputMode>(OM);
impl<OM: OutputMode> Mode for Output<OM> {}

/// Implemented by types that indicate a GPIO output mode.
///
/// Used as trait bound by the [`Output`] type.
pub trait OutputMode {}

/// Marks a GPIO output pin as push-pull.
pub struct PushPull;
impl OutputMode for PushPull {}

/// Marks a GPIO output pin as open-source.
pub struct OpenSource;
impl OutputMode for OpenSource {}

/// Marks a GPIO output pin as open-drain.
pub struct OpenDrain;
impl OutputMode for OpenDrain {}

/// GPIO pin
// TODO: More documentation
pub struct Pin<T: PinTrait, M: Mode> {
    ty: T,
    _mode: PhantomData<M>,
}

impl<T, M: Mode> Pin<T, M>
where
    T: PinTrait,
{
    pub fn reset(mut self) -> PinBuilder<T, Floating, NoFilter> {
        self.ty.clear_mode();

        // When the pin was configured as output or as input with the glitch
        // filter enabled the DOUT bit might be set. The DOUT bit enables a
        // pull up resistor in DISABLED mode. Clear the DOUT bit to disable
        // the pull-up as soon as possible.
        self.ty.clear_dout_bit();

        PinBuilder {
            ty: self.ty,
            _pull: PhantomData,
            _filter: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, Floating, NoFilter>> for Pin<T, Disabled>
where
    T: PinTrait,
{
    fn from(pb: PinBuilder<T, Floating, NoFilter>) -> Self {
        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, PullUp, NoFilter>> for Pin<T, Disabled>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, PullUp, NoFilter>) -> Self {
        pb.ty.set_dout_bit();

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, Floating, NoFilter>> for Pin<T, Input>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, Floating, NoFilter>) -> Self {
        pb.ty.set_mode(MODE::INPUT);

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, Floating, Filter>> for Pin<T, Input>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, Floating, Filter>) -> Self {
        // Change to INPUT mode first, so that setting the DOUT bit does not
        // accidentally activate the pull-up resistor while still in DISABLED mode.
        pb.ty.set_mode(MODE::INPUT);
        pb.ty.set_dout_bit();

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, PullDown, NoFilter>> for Pin<T, Input>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, PullDown, NoFilter>) -> Self {
        pb.ty.set_mode(MODE::INPUTPULL);

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, PullUp, NoFilter>> for Pin<T, Input>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, PullUp, NoFilter>) -> Self {
        pb.ty.set_dout_bit();
        pb.ty.set_mode(MODE::INPUTPULL);

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, PullDown, Filter>> for Pin<T, Input>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, PullDown, Filter>) -> Self {
        pb.ty.set_mode(MODE::INPUTPULLFILTER);

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, PullUp, Filter>> for Pin<T, Input>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, PullUp, Filter>) -> Self {
        pb.ty.set_dout_bit();
        pb.ty.set_mode(MODE::INPUTPULLFILTER);

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, Floating, NoFilter>> for Pin<T, Output<PushPull>>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, Floating, NoFilter>) -> Self {
        pb.ty.set_mode(MODE::PUSHPULL);

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, Floating, NoFilter>> for Pin<T, Output<OpenSource>>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, Floating, NoFilter>) -> Self {
        pb.ty.set_mode(MODE::WIREDOR);

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, PullDown, NoFilter>> for Pin<T, Output<OpenSource>>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, PullDown, NoFilter>) -> Self {
        pb.ty.set_mode(MODE::WIREDORPULLDOWN);

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, Floating, NoFilter>> for Pin<T, Output<OpenDrain>>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, Floating, NoFilter>) -> Self {
        pb.ty.set_mode(MODE::WIREDAND);

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, Floating, Filter>> for Pin<T, Output<OpenDrain>>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, Floating, Filter>) -> Self {
        pb.ty.set_mode(MODE::WIREDANDFILTER);

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, PullUp, NoFilter>> for Pin<T, Output<OpenDrain>>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, PullUp, NoFilter>) -> Self {
        pb.ty.set_mode(MODE::WIREDANDPULLUP);

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

impl<T> From<PinBuilder<T, PullUp, Filter>> for Pin<T, Output<OpenDrain>>
where
    T: PinTrait,
{
    fn from(mut pb: PinBuilder<T, PullUp, Filter>) -> Self {
        pb.ty.set_mode(MODE::WIREDANDPULLUPFILTER);

        Self {
            ty: pb.ty,
            _mode: PhantomData,
        }
    }
}

/// Internal trait to prevent duplicate implemenations of embedded-hal traits.
/// Leaked because it is used as trait bound. Not relevant for the user.
pub trait InputAvailable {}
impl InputAvailable for Input {}
impl<OM: OutputMode> InputAvailable for Output<OM> {}

impl<T, M> InputPin for Pin<T, M>
where
    T: PinTrait,
    M: Mode + InputAvailable,
{
    type Error = ();

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.ty.read_din_bit())
    }

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.ty.read_din_bit())
    }
}

impl<T, OM> OutputPin for Pin<T, Output<OM>>
where
    T: PinTrait,
    OM: OutputMode,
{
    type Error = ();

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.ty.clear_dout_bit();
        Ok(())
    }

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.ty.set_dout_bit();
        Ok(())
    }
}

impl<T, OM> StatefulOutputPin for Pin<T, Output<OM>>
where
    T: PinTrait,
    OM: OutputMode,
{
    fn is_set_low(&self) -> Result<bool, Self::Error> {
        Ok(!self.ty.read_dout_bit())
    }

    fn is_set_high(&self) -> Result<bool, Self::Error> {
        Ok(self.ty.read_dout_bit())
    }
}

impl<T, OM> ToggleableOutputPin for Pin<T, Output<OM>>
where
    T: PinTrait,
    OM: OutputMode,
{
    type Error = ();

    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.ty.write_douttgl_bit();
        Ok(())
    }
}
