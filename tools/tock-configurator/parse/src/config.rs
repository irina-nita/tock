// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use parse_macros::capsules_config;

use crate::LedType;
use crate::{DefaultPeripherals, SchedulerType, SyscallFilterType};
use crate::{Lsm303AccelDataRate, Lsm303MagnetoDataRate, Lsm303Range, Lsm303Scale};
use std::{collections::HashMap, num::NonZeroUsize, rc::Rc};
pub type CapsulesConfigurations<P> = HashMap<Index, Capsule<P>>;

capsules_config!(
    Index => Capsule<P: crate::DefaultPeripherals>,
    // The keys and values enums for the capsules map.
    {
        ALARM => Alarm { timer: Rc<P::Timer> },
        LED => Led { led_type: LedType, pins: Vec<<P::Gpio as crate::Gpio>::PinId> },
        SPI => Spi { spi: Rc<P::Spi> },
        I2C => I2c { i2c: Rc<P::I2c> },
        BLE => BleRadio { ble: Rc<P::BleAdvertisement>, timer: Rc<P::Timer> },
        FLASH => Flash { flash: Rc<P::Flash>, buffer_size: usize },
        LSM303AGR => Lsm303agr { i2c: Rc<P::I2c>,
                                 accel_data_rate: Lsm303AccelDataRate,
                                 low_power: bool,
                                 accel_scale: Lsm303Scale,
                                 accel_high_resolution: bool,
                                 temperature: bool,
                                 mag_data_rate: Lsm303MagnetoDataRate,
                                 mag_range: Lsm303Range  },
        CONSOLE => Console { uart: Rc<P::Uart>, baud_rate: usize},
        TEMPERATURE => Temperature { temp: Rc<P::Temperature> },
        RNG => Rng { rng: Rc<P::Rng> },
        GPIO => Gpio { pins: Vec<<P::Gpio as crate::Gpio>::PinId> },
        HMAC => Hmac { hmac: Rc<P::Hmac>, length: usize },
        KV_DRIVER => KvDriver { flash: Rc<P::Flash> },
        AES => Aes { aes: Rc<P::Aes>, number_of_blocks: usize },
    }
);

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub struct Configuration<P: DefaultPeripherals> {
    // The type of the board struct configured.
    // Considered neither optional nor required,
    // but more of a way to integrate with already-defined Tock platforms.
    pub r#type: String,

    // Capsules are the optional configuration fields.
    // The map representation removes the redundancy of having
    // multiple optional fields for serialization purposes.
    capsules: CapsulesConfigurations<P>,

    // The required configuration fields for the platform.
    pub scheduler: SchedulerType,
    pub process_count: usize,
    pub stack_size: NonZeroUsize,
    pub syscall_filter: SyscallFilterType,
}

impl<P: DefaultPeripherals> Default for Configuration<P> {
    fn default() -> Self {
        Self {
            r#type: String::from("AutogeneratedPlatform"),
            capsules: Default::default(),
            scheduler: Default::default(),
            process_count: Default::default(),
            stack_size: unsafe { NonZeroUsize::new_unchecked(0x900) },
            syscall_filter: SyscallFilterType::None,
        }
    }
}

// Configuration methods exposed for the `configurator` crate.
impl<P: DefaultPeripherals> Configuration<P> {
    /// Return a vector of the configured capsules.
    pub fn capsules(&self) -> Vec<&Capsule<P>> {
        self.capsules.values().collect()
    }

    pub fn capsule(&self, capsule: &Index) -> Option<&Capsule<P>> {
        self.capsules.get(capsule)
    }

    /// Update the console configuration.
    pub fn update_console(&mut self, uart: Rc<P::Uart>, baud_rate: usize) {
        self.capsules
            .insert(Index::CONSOLE, Capsule::Console { uart, baud_rate });
    }

    /// Update the alarm configuration.
    pub fn update_alarm(&mut self, timer: Rc<P::Timer>) {
        self.capsules.insert(Index::ALARM, Capsule::Alarm { timer });
    }

    /// Update the spi configuration.
    pub fn update_spi(&mut self, spi: Rc<P::Spi>) {
        self.capsules.insert(Index::SPI, Capsule::Spi { spi });
    }

    /// Update the i2c configuration.
    pub fn update_i2c(&mut self, i2c: Rc<P::I2c>) {
        self.capsules.insert(Index::I2C, Capsule::I2c { i2c });
    }

    /// Update the ble configuration.
    pub fn update_ble(&mut self, ble: Rc<P::BleAdvertisement>, timer: Rc<P::Timer>) {
        self.capsules
            .insert(Index::BLE, Capsule::BleRadio { ble, timer });
    }

    /// Update the temperature configuration.
    pub fn update_temp(&mut self, temp: Rc<P::Temperature>) {
        self.capsules
            .insert(Index::TEMPERATURE, Capsule::Temperature { temp });
    }

    /// Update the rng configuration.
    pub fn update_rng(&mut self, rng: Rc<P::Rng>) {
        self.capsules.insert(Index::RNG, Capsule::Rng { rng });
    }

    /// Update the lsm303agr configuration.
    // FIXME: Move the LSM config to a struct.
    pub fn update_lsm303agr(
        &mut self,
        i2c: Rc<P::I2c>,
        accel_data_rate: Lsm303AccelDataRate,
        low_power: bool,
        accel_scale: Lsm303Scale,
        accel_high_resolution: bool,
        temperature: bool,
        mag_data_rate: Lsm303MagnetoDataRate,
        mag_range: Lsm303Range,
    ) {
        #![allow(clippy::too_many_arguments)]
        self.capsules.insert(
            Index::LSM303AGR,
            Capsule::Lsm303agr {
                i2c,
                accel_data_rate,
                low_power,
                accel_scale,
                accel_high_resolution,
                temperature,
                mag_data_rate,
                mag_range,
            },
        );
    }

    /// Update the alarm configuration.
    pub fn update_flash(&mut self, flash: Rc<P::Flash>, buffer_size: usize) {
        self.capsules
            .insert(Index::FLASH, Capsule::Flash { flash, buffer_size });
    }

    pub fn update_gpio(&mut self, pins: Vec<<P::Gpio as crate::Gpio>::PinId>) {
        self.capsules.insert(Index::GPIO, Capsule::Gpio { pins });
    }

    pub fn update_led(&mut self, led_type: LedType, pins: Vec<<P::Gpio as crate::Gpio>::PinId>) {
        self.capsules.insert(
            Index::LED,
            Capsule::Led {
                led_type: led_type,
                pins,
            },
        );
    }

    pub fn update_hmac(&mut self, hmac: Rc<P::Hmac>, length: usize) {
        self.capsules
            .insert(Index::HMAC, Capsule::Hmac { hmac, length });
    }

    pub fn update_aes(&mut self, aes: Rc<P::Aes>, number_of_blocks: usize) {
        self.capsules.insert(
            Index::AES,
            Capsule::Aes {
                aes,
                number_of_blocks,
            },
        );
    }

    pub fn update_kv_driver(&mut self, flash: Rc<P::Flash>) {
        self.capsules
            .insert(Index::KV_DRIVER, Capsule::KvDriver { flash });
    }

    /// Update the scheduler configuration.
    pub fn update_scheduler(&mut self, scheduler_type: SchedulerType) {
        self.scheduler = scheduler_type;
    }

    /// Update the stack size.
    pub fn update_stack_size(&mut self, stack_size: usize) {
        if let Some(s) = NonZeroUsize::new(stack_size) {
            self.stack_size = s;
        }
    }

    /// Update the type of syscall filter.
    pub fn update_syscall_filter(&mut self, syscall_filter: SyscallFilterType) {
        self.syscall_filter = syscall_filter
    }

    pub fn update_type(&mut self, ty: impl Into<String>) {
        self.r#type = ty.into();
    }

    /// Remove the console configuration.
    pub fn remove_console(&mut self) {
        self.capsules.remove(&Index::CONSOLE);
    }

    /// Remove the alarm configuration.
    pub fn remove_alarm(&mut self) {
        self.capsules.remove(&Index::ALARM);
    }

    /// Remove the spi configuration.
    pub fn remove_spi(&mut self) {
        self.capsules.remove(&Index::SPI);
    }

    /// Remove the i2c configuration.
    pub fn remove_i2c(&mut self) {
        self.capsules.remove(&Index::I2C);
    }

    /// Remove the gpio configuration.
    pub fn remove_gpio(&mut self) {
        self.capsules.remove(&Index::GPIO);
    }

    /// Remove the ble configuration.
    pub fn remove_ble(&mut self) {
        self.capsules.remove(&Index::BLE);
    }

    /// Remove the lsm303agr configuration.
    pub fn remove_lsm303agr(&mut self) {
        self.capsules.remove(&Index::LSM303AGR);
    }

    /// Remove the flash configuration.
    pub fn remove_flash(&mut self) {
        self.capsules.remove(&Index::FLASH);
    }

    /// Remove the temperature configuration.
    pub fn remove_temp(&mut self) {
        self.capsules.remove(&Index::TEMPERATURE);
    }

    /// Remove the rng configuration.
    pub fn remove_rng(&mut self) {
        self.capsules.remove(&Index::RNG);
    }

    /// Remove the LED configuration.
    pub fn remove_led(&mut self) {
        self.capsules.remove(&Index::LED);
    }

    pub fn remove_hmac(&mut self) {
        self.capsules.remove(&Index::HMAC);
    }

    pub fn remove_kv_driver(&mut self) {
        self.capsules.remove(&Index::KV_DRIVER);
    }

    pub fn remove_aes(&mut self) {
        self.capsules.remove(&Index::AES);
    }
}
