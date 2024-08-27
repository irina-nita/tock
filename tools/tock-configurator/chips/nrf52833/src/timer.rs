// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use parse::constants::PERIPHERALS;
use parse::{peripheral, Component};
use quote::quote;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum TimerType {
    Rtc,
}

#[derive(Debug, PartialEq)]
#[peripheral(serde, ident = ".nrf52.rtc")]
pub struct Timer(TimerType);

impl Component for Timer {
    fn ty(&self) -> Result<parse::proc_macro2::TokenStream, parse::Error> {
        Ok(quote!(nrf52::rtc::Rtc<'static>))
    }
}

impl parse::Timer for Timer {
    fn frequency(&self) -> usize {
        0
    }
}

impl std::fmt::Display for Timer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rtc")
    }
}
