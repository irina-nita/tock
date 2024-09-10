// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use std::rc::Rc;

use crate::menu::capsule_popup;
use crate::state::Data;
use parse::peripherals::{Chip, DefaultPeripherals};

/// Menu for configuring the SPI Controller capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    choice: Option<Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Spi>>,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't a SPI Controller already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => {
            // Unwrap is safe because if we already had a RNG capsule configured
            // then there is a RNG peripheral.
            let spi_peripherals = Vec::from(chip.peripherals().spi().unwrap());
            capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                Vec::from(spi_peripherals),
                on_spi_submit::<C>,
                inner,
            ))
        }
    }
}

/// Menu for configuring the SPI capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    let spi_peripherals = Vec::from(chip.peripherals().spi().unwrap());
    // If we have at least one SPI peripheral, we make a list with it.
    capsule_popup::<C, _>(crate::views::radio_group_with_null(
        Vec::from(spi_peripherals),
        on_spi_submit::<C>,
    ))
}

/// Configure a SPI controller based on the submitted SPI.
fn on_spi_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::Spi>>,
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    if let Some(spi) = submit {
        data.platform.update_spi(Rc::clone(spi));
    } else {
        data.platform.remove_spi();
    }
}
