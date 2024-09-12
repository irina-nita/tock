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

/// Menu for configuring the AES capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    choice: Option<(
        Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Aes>,
        usize
    )>,
) -> cursive::views::LinearLayout {
    match choice {
        // If there isn't an AES already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => 
        {
            let aes_peripherals = Vec::from(chip.peripherals().aes().unwrap());
            // If we have at least one AES peripheral, we make a list with it.
            capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                Vec::from(aes_peripherals),
                move |siv, submit| on_aes_submit::<C>(siv, submit, inner.1),
                inner.0,
            ))
        }
    }
}

/// Menu for configuring the AES capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(
    chip: Rc<C>,
) -> cursive::views::LinearLayout {
    let aes_peripherals = Vec::from(chip.peripherals().aes().unwrap());
    capsule_popup::<C, _>(crate::views::radio_group_with_null(
        Vec::from(aes_peripherals),
        |siv, submit| on_aes_submit::<C>(siv, submit, 7),
    ))
}

/// Initialize a board configuration session based on the submitted chip.
fn on_aes_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::Aes>>,
    default_crypt_size: usize,
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    if let Some(aes) = submit {
        siv.add_layer(crypt_size_popup::<C>(aes.clone(), default_crypt_size));
    } else {
        data.platform.remove_aes();
    }
}

/// Menu for configuring the crypt size for the aes.
fn crypt_size_popup<C: Chip + 'static + serde::ser::Serialize>(
    aes: Rc<<C::Peripherals as DefaultPeripherals>::Aes>,
    default_value: usize,
) -> cursive::views::Dialog {
    let aes_clone = aes.clone();
    Dialog::around(
        EditView::new()
            .content(format!("{default_value}"))
            .on_submit(move |siv, name| on_crypt_size_submit::<C>(siv, name, aes.clone()))
            .with_name("crypt_size"),
    )
    .title("Crypt_Size")
    .button("Save", move |siv| {
        let count = siv
            .call_on_name("crypt_size", |view: &mut EditView| view.get_content())
            .unwrap();
        on_crypt_size_submit::<C>(siv, &count, aes_clone.clone());
    })
}

/// Add the details for the aes and return to the aes selection.
fn on_crypt_size_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    name: &str,
    aes: Rc<<C::Peripherals as DefaultPeripherals>::Aes>,
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    let crypt_size = if name.is_empty() {
        Ok(7)
    } else {
        name.parse::<usize>()
    };

    if let Ok(crypt_size) = crypt_size {
        data.platform.update_aes(aes.clone(), crypt_size);
    }

    siv.pop_layer();
}
