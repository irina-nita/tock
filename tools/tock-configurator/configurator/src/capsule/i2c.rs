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

/// Menu for configuring the I2C capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    choice: Option<Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::I2c>>,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't an I2C already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => 
        {
            let i2c_peripherals = Vec::from(chip.peripherals().i2c().unwrap());
            // If we have at least one I2C peripheral, we make a list with it.
            capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                Vec::from(i2c_peripherals),
                on_i2c_submit::<C>,
                inner,
            ))
        }
    }
}

/// Menu for configuring the I2C capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    let i2c_peripherals = Vec::from(chip.peripherals().i2c().unwrap());
    capsule_popup::<C, _>(crate::views::radio_group_with_null(
        Vec::from(i2c_peripherals),
        on_i2c_submit::<C>,
    ))
}

/// Configure an I2C capsule based on the submitted I2C peripheral.
fn on_i2c_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::I2c>>,
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    if let Some(i2c) = submit {
        data.platform.update_i2c(Rc::clone(i2c));
    } else {
        data.platform.remove_i2c();
    }
}
