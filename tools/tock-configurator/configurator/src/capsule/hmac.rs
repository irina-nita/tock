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

/// Menu for configuring the HMAC capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    choice: Option<(
        Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Hmac>,
        usize,
    )>,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't a HMAC already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => 
        {
            let hmac_peripherals = Vec::from(chip.peripherals().hmac().unwrap());
            // If we have at least one HMAC peripheral, we make a list with it.
            capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                Vec::from(hmac_peripherals),
                move |siv, submit| on_hmac_submit::<C>(siv, submit, inner.1),
                inner.0,
            ))
        }
    }
}

/// Menu for configuring the HMAC capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    let hmac_peripherals = Vec::from(chip.peripherals().hmac().unwrap());
    capsule_popup::<C, _>(crate::views::radio_group_with_null(
        Vec::from(hmac_peripherals),
        |siv, submit| on_hmac_submit::<C>(siv, submit, 16),
    ))
}


/// Initialize a board configuration session based on the submitted chip.
fn on_hmac_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::Hmac>>,
    default_buffer_len: usize
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    if let Some(hmac) = submit {
        siv.add_layer(buffer_len_popup::<C>(hmac.clone(), default_buffer_len));
    } else {
        data.platform.remove_hmac();
    }
}

/// Menu for configuring the buffer length for the hmac.
fn buffer_len_popup<C: Chip + 'static + serde::ser::Serialize>(
    hmac: Rc<<C::Peripherals as DefaultPeripherals>::Hmac>,
    default_value: usize,
) -> cursive::views::Dialog {
    let hmac_clone = hmac.clone();
    Dialog::around(
        EditView::new()
            .content(format!("{default_value}"))
            .on_submit(move |siv, name| on_buffer_len_submit::<C>(siv, name, hmac.clone()))
            .with_name("buffer_len"),
    )
    .title("Buffer_len")
    .button("Save", move |siv| {
        let count = siv
            .call_on_name("buffer_len", |view: &mut EditView| view.get_content())
            .unwrap();
        on_buffer_len_submit::<C>(siv, &count, hmac_clone.clone());
    })
}

/// Add the details for the hmac and return to the hmac selection.
fn on_buffer_len_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    name: &str,
    hmac: Rc<<C::Peripherals as DefaultPeripherals>::Hmac>,
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    let buffer_len = if name.is_empty() {
        Ok(16)
    } else {
        name.parse::<usize>()
    };

    if let Ok(buffer_len) = buffer_len {
        data.platform.update_hmac(hmac.clone(), buffer_len);
    }

    siv.pop_layer();
}
