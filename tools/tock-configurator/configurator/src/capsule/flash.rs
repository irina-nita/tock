// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use std::rc::Rc;

use crate::menu::capsule_popup;
use crate::state::Data;
use cursive::view::Nameable;
use cursive::views::{Dialog, EditView};
use parse::peripherals::{Chip, DefaultPeripherals};

/// Menu for configuring the App Flash capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    choice: Option<(
        Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Flash>,
        usize,
    )>,
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't a App Flash already configured, we switch to another menu.
        None => config_none::<C>(chip),
        Some(inner) => { 
            let flash = Vec::from(chip.peripherals().flash().unwrap());
            // If we have at least one flash peripheral, we make a list with it.
            capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                Vec::from(flash),
                move |siv, choice| on_flash_submit::<C>(siv, choice, inner.1),
                inner.0,
            ))
        }
    }
}

/// Menu for configuring the App Flash capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    let flash_peripherals = Vec::from(chip.peripherals().flash().unwrap());
    crate::menu::capsule_popup::<C, _>(
        crate::views::radio_group_with_null(Vec::from(flash_peripherals), |siv, submit| {
            on_flash_submit::<C>(siv, submit, 512)
        }),
    )
}

/// Continue to buffer size configuration after choosing
/// a flash peripheral.
fn on_flash_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    flash: &Option<Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Flash>>,
    default_buffer_size: usize,
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    match flash {
        Some(flash) => siv.add_layer(buffer_size_popup::<C>(
            Rc::clone(flash),
            default_buffer_size,
        )),
        None => {
            data.platform.remove_flash();
        }
    }
}

/// Menu for configuring the buffer size for the uart.
fn buffer_size_popup<C: Chip + 'static + serde::ser::Serialize>(
    flash: Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Flash>,
    default_value: usize,
) -> cursive::views::Dialog {
    let flash_clone = Rc::clone(&flash);
    Dialog::around(
        EditView::new()
            .content(format!("{default_value}"))
            .on_submit(move |siv, name| on_buffer_size::<C>(siv, name, Rc::clone(&flash_clone)))
            .with_name("buffer_size"),
    )
    .title("Buffer Size")
    .button("Save", move |siv| {
        let count = siv
            .call_on_name("buffer_size", |view: &mut EditView| view.get_content())
            .unwrap();
        on_buffer_size::<C>(siv, &count, Rc::clone(&flash));
    })
}

/// Configure an App Flash capsule based on the Flash peripheral and the provided buffer size.
fn on_buffer_size<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    name: &str,
    flash: Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Flash>,
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    let buffer_size = if name.is_empty() {
        Ok(512)
    } else {
        name.parse::<usize>()
    };

    if let Ok(b) = buffer_size {
        data.platform.update_flash(flash, b);
    }
    siv.pop_layer();
}