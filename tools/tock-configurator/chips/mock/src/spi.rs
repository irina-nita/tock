// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

//! Not fully supported yet.

use parse::constants::PERIPHERALS;
use parse::peripheral;

#[derive(Debug)]
#[peripheral(serde, ident = "spi")]
pub struct Spi {}

impl parse::Spi for Spi {}
impl parse::Component for Spi {}

impl std::fmt::Display for Spi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "spi")
    }
}
