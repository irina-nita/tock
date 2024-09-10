// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use crate::capsule::Chip;
use crate::menu::capsule_popup;
use crate::state::Data;
use cursive::view::Nameable;
use cursive::views::{Dialog, EditView, LinearLayout};
use parse::peripherals::DefaultPeripherals;
use std::rc::Rc;

/// Menu for configuring the Console capsule.
pub fn config<C: Chip + 'static + serde::Serialize>(
    chip: Rc<C>,
    previous_state: Option<(
        Rc<<<C as parse::peripherals::Chip>::Peripherals as DefaultPeripherals>::Uart>,
        usize,
    )>,
) -> cursive::views::LinearLayout {
    match previous_state {
        // If there isn't a Console already configured, we switch to another menu.
        None => config_none(chip),
        Some(inner) => {
            let uart_peripherals = Vec::from(chip.peripherals().uart().unwrap());
            // If we have at least one UART peripheral, we make a list with it.
            capsule_popup::<C, _>(crate::views::radio_group_with_null_known(
                uart_peripherals,
                move |siv, submit| on_uart_submit::<C>(siv, submit, inner.1),
                inner.0,
            ))
        },
    }
}

/// Menu for configuring the Console capsule when none was configured before.
fn config_none<C: Chip + 'static + serde::ser::Serialize>(chip: Rc<C>) -> LinearLayout {
    let uart_peripherals = Vec::from(chip.peripherals().uart().unwrap());
    crate::menu::capsule_popup::<C, _>(
        crate::views::radio_group_with_null(Vec::from(uart_peripherals), |siv, submit| {
            on_uart_submit::<C>(siv, submit, 112500)
        }),
    )
}

/// Initialize a board configuration session based on the submitted chip.
fn on_uart_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<C::Peripherals as DefaultPeripherals>::Uart>>,
    default_baud_rate: usize,
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    if let Some(uart) = submit {
        siv.add_layer(baud_rate_popup::<C>(Rc::clone(uart), default_baud_rate));
    } else {
        data.platform.remove_console();
    }
}

/// Menu for configuring the baud rate for the uart.
fn baud_rate_popup<C: Chip + 'static + serde::ser::Serialize>(
    uart: Rc<<C::Peripherals as DefaultPeripherals>::Uart>,
    default_value: usize,
) -> cursive::views::Dialog {
    let uartc = Rc::clone(&uart);
    Dialog::around(
        EditView::new()
            .content(format!("{default_value}"))
            .on_submit(move |siv, name| on_baud_submit::<C>(siv, Rc::clone(&uart), name))
            .with_name("baud_rate"),
    )
    .title("Baud_rate")
    .button("Save", move |siv| {
        let count = siv
            .call_on_name("baud_rate", |view: &mut EditView| view.get_content())
            .unwrap();
        on_baud_submit::<C>(siv, Rc::clone(&uartc), &count);
    })
}

/// Add the details for the uart and return to the UART selection.
fn on_baud_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    uart: Rc<<C::Peripherals as DefaultPeripherals>::Uart>,
    baud_rate_str: &str,
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    let baud_rate = if baud_rate_str.is_empty() {
        Ok(115200)
    } else {
        baud_rate_str.parse::<usize>()
    };

    if let Ok(br) = baud_rate {
        data.platform.update_console(Rc::clone(&uart), br);
    }
    siv.pop_layer();
}
