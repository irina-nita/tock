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

/// Menu for configuring the KV Driver capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    choice: Option<Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Flash>>,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't a KV Driver already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => 
        {
            let flash_peripherals = Vec::from(chip.peripherals().flash().unwrap());
            // If we have at least one Flash peripheral, we make a list with it.
            capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                Vec::from(flash_peripherals),
                on_flash_submit::<C>,
                inner,
            ))
        }
    }
}

/// Menu for configuring the KV Driver capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    let flash_peripherals = Vec::from(chip.peripherals().flash().unwrap());
    capsule_popup::<C, _>(crate::views::radio_group_with_null(
        Vec::from(flash_peripherals),
        on_flash_submit::<C>,
    ))
}



/// Configure a Flash info capsule based on the submitted Flash peripheral.
fn on_flash_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<<C as Chip>::Peripherals as DefaultPeripherals>::Flash>>,
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    match submit {
        Some(flash) => data.platform.update_kv_driver(Rc::clone(&flash)),
        None => data.platform.remove_kv_driver(),
    }
}
