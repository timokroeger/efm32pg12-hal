# EFM32PG12 HAL

Hardware abstraction layer (HAL) for the [Silicon Labs EFM32PG12] family of ARM Cortex-M4
microcontrollers, written in Rust.

## Documentation

The [efm32pg12-pac](https://crates.io/crates/efm32pg12-pac) crate provides the register definitions
and is reexported as `pac` by this crate.

Additional vendor supplied documents:
- [Datasheet](https://www.silabs.com/documents/public/data-sheets/efm32pg12-datasheet.pdf)
- [Reference Manual](https://www.silabs.com/documents/public/reference-manuals/efm32pg12-rm.pdf)
- [Errata](https://www.silabs.com/documents/public/errata/efm32pg12-errata.pdf)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

[Silicon Labs EFM32PG12]: https://www.silabs.com/products/mcu/32-bit/efm32-pearl-gecko
