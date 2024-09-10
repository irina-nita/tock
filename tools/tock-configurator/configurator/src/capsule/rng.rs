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

/// Menu for configuring the Rng capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    choice: Option<Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Rng>>,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't a RNG already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => {
            // Unwrap is safe because if we already had a RNG capsule configured
            // then there is a RNG peripheral.
            let rng_peripherals = Vec::from(chip.peripherals().rng().unwrap());
            capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                Vec::from(rng_peripherals),
                on_rng_submit::<C>,
                inner,
            ))
        },
    }
}

/// Menu for configuring the RNG capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    let rng_peripherals = Vec::from(chip.peripherals().rng().unwrap());
    // If we have at least one RNG peripheral, we make a list with it.
    capsule_popup::<C, _>(crate::views::radio_group_with_null(
        Vec::from(rng_peripherals),
        on_rng_submit::<C>,
    ))
}

/// Configure a RNG based on the submitted RNG.
fn on_rng_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::Rng>>,
) {
    // Unwrap is safe because this function can't be called without having user data.
    let data = siv.user_data::<Data<C>>().unwrap();
    if let Some(rng) = submit {
        data.platform.update_rng(Rc::clone(rng));
    } else {
        data.platform.remove_rng();
    }
}
