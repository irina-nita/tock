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

/// Menu for configuring the Temperature capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    choice: Option<
        Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Temperature>,
    >,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't a Temperature already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => {
            // Unwrap is safe because if we already had a Temperature capsule configured
            // then there is a Temperature peripheral.
            let temp_peripherals = Vec::from(chip.peripherals().temp().unwrap());
            capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                Vec::from(temp_peripherals),
                on_temp_submit::<C>,
                inner,
            ))
        },
    }
}

/// Menu for configuring the Temperature capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    let temp_peripherals = Vec::from(chip.peripherals().temp().unwrap());
    // If we have at least one Temperature peripheral, we make a list with it.
    capsule_popup::<C, _>(crate::views::radio_group_with_null(
        Vec::from(temp_peripherals),
        on_temp_submit::<C>,
    ))
}

/// Configure a Temperature capsule based on the submitted Temperature peripheral.
fn on_temp_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::Temperature>>,
) {
    // Unwrap is safe because this function can't be called without having user data.
    let data = siv.user_data::<Data<C>>().unwrap();
    if let Some(temp) = submit {
        data.platform.update_temp(Rc::clone(temp));
    } else {
        data.platform.remove_temp();
    }
}
