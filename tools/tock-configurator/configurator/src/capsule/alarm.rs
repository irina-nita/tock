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

/// Menu for configuring the Alarm capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    previous_state: Option<
        Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Timer>,
    >,
) -> cursive::views::LinearLayout {
    match previous_state {
        // If there isn't an Alarm already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => {
            // Unwrap is safe because if we already had an Alarm capsule configured
            // then there is a Timer peripheral.
            let timer_peripherals = Vec::from(chip.peripherals().timer().unwrap());
            capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                timer_peripherals,
                on_timer_submit::<C>,
                inner,
            ))
        },
    }
}

/// Menu for configuring the Alarm capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    let timer_peripherals = Vec::from(chip.peripherals().timer().unwrap());
    // If we have at least one Timer peripheral, we make a list with it.
    capsule_popup::<C, _>(crate::views::radio_group_with_null(
        timer_peripherals,
        on_timer_submit::<C>,
    ))
}

/// Configure an Alarm capsule based on the submitted Timer peripheral.
fn on_timer_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::Timer>>,
) {
    // Unwrap is safe because this function can't be called without having user data.
    let data = siv.user_data::<Data<C>>().unwrap();
    match submit {
        Some(timer) => data.platform.update_alarm(Rc::clone(timer)),
        None => data.platform.remove_alarm(),
    }
}
