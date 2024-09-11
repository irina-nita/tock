// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use parse::NoSupport;

use crate::{timer, uart, FlashType, TemperatureType, UartType};
use std::rc::Rc;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Peripherals {
    // Default peripherals for the Microbit.
    uart: [Rc<crate::uart::Uart>; 1],
    timer: [Rc<crate::timer::Timer>; 1],
    ble: [Rc<crate::ble::Ble>; 1],
    rng: [Rc<crate::rng::Rng>; 1],
    temperature: [Rc<crate::temperature::Temperature>; 1],
    twi: [Rc<crate::twi::Twi>; 1],
    gpio: [Rc<crate::gpio::Gpio>; 1],
    flash: [Rc<crate::flash::Flash>; 1],
    spi: [Rc<crate::spi::Spi>; 1],
}

impl Peripherals {
    pub fn new() -> Self {
        Self {
            uart: [Rc::new(uart::Uart::new(UartType::Uart0))],
            timer: [Rc::new(timer::Timer::new(crate::TimerType::Rtc))],
            ble: [Rc::new(crate::ble::Ble::new(crate::BleType::RadioBle))],
            rng: [Rc::new(crate::rng::Rng::new(crate::RngType::Rng))],
            temperature: [Rc::new(crate::temperature::Temperature::new(
                TemperatureType::Temp,
            ))],
            twi: [Rc::new(crate::Twi::new())],
            flash: [Rc::new(crate::Flash::new(FlashType::Flash0))],
            gpio: [Rc::new(crate::gpio::Gpio::new())],
            spi: [Rc::new(crate::spi::Spi::new())]
        }
    }
}

impl Default for Peripherals {
    fn default() -> Self {
        Self::new()
    }
}

impl parse::Component for Peripherals {
    fn init_expr(&self) -> Result<parse::proc_macro2::TokenStream, parse::Error> {
        todo!()
    }

    fn before_init(&self) -> Option<parse::proc_macro2::TokenStream> {
        todo!()
    }

    fn after_init(&self) -> Option<parse::proc_macro2::TokenStream> {
        todo!()
    }
}

impl parse::DefaultPeripherals for Peripherals {
    type Gpio = crate::Gpio;
    type Uart = crate::Uart;
    type Timer = crate::Timer;
    type I2c = crate::Twi;
    type Spi = crate::Spi;
    type Rng = crate::Rng;
    type BleAdvertisement = crate::Ble;
    type Temperature = crate::Temperature;
    type Flash = crate::Flash;
    type Aes = NoSupport;
    type Hmac = NoSupport;

    fn uart(&self) -> Result<&[Rc<Self::Uart>], parse::Error> {
        Ok(&self.uart)
    }

    fn timer(&self) -> Result<&[Rc<Self::Timer>], parse::Error> {
        Ok(&self.timer)
    }

    fn i2c(&self) -> Result<&[Rc<Self::I2c>], parse::Error> {
        Ok(&self.twi)
    }

    fn ble(&self) -> Result<&[Rc<Self::BleAdvertisement>], parse::Error> {
        Ok(&self.ble)
    }

    fn flash(&self) -> Result<&[Rc<Self::Flash>], parse::Error> {
        Ok(&self.flash)
    }

    fn temp(&self) -> Result<&[Rc<Self::Temperature>], parse::Error> {
        Ok(&self.temperature)
    }

    fn rng(&self) -> Result<&[Rc<Self::Rng>], parse::Error> {
        Ok(&self.rng)
    }

    fn gpio(&self) -> Result<&[Rc<Self::Gpio>], parse::Error> {
        Ok(&self.gpio)
    }

    fn spi(&self) -> Result<&[Rc<Self::Spi>], parse::Error> {
        Ok(&self.spi)
    }
}
