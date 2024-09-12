// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use crate::items::ToMenuItem;
use crate::menu::{capsule_popup, pin_list_disabled};
use crate::state::{on_exit_submit, on_quit_submit, Data, PinFunction};
use crate::views;
use cursive::views::{Checkbox, ListChild, ListView};
use cursive::Cursive;
use parse::peripherals::{Chip, DefaultPeripherals, Gpio};
use std::rc::Rc;

use super::ConfigMenu;

#[derive(Debug)]
pub(crate) struct LedConfig;

impl ConfigMenu for LedConfig {
    /// Menu for configuring the led capsule.
    fn config<C: Chip + 'static + serde::ser::Serialize>(
        chip: Rc<C>,
    ) -> cursive::views::LinearLayout {
        let gpio_peripherals = Vec::from(chip.peripherals().gpio().unwrap());
        capsule_popup::<C, _>(views::select_menu(
            Vec::from(gpio_peripherals)
                .into_iter()
                .map(|elem| elem.to_menu_item())
                .collect(),
            |siv, submit| {
                crate::state::on_gpio_submit::<C, _>(siv, submit.clone(), led_type_popup::<C>)
            },
        ))
    }
}

fn led_type_popup<C: Chip + 'static + serde::ser::Serialize>(
    gpio: Rc<<C::Peripherals as DefaultPeripherals>::Gpio>,
) -> cursive::views::LinearLayout {
    capsule_popup::<C, _>(views::select_menu(
        vec![
            ("LedHigh", parse::capsules::led::LedType::LedHigh),
            ("LedLow", parse::capsules::led::LedType::LedLow),
        ],
        move |siv, choice| on_led_type_submit::<C>(siv, gpio.clone(), choice.clone()),
    ))
}

fn led_pins_popup<C: Chip + 'static + serde::ser::Serialize>(
    gpio: Rc<<C::Peripherals as DefaultPeripherals>::Gpio>,
    pin_list: Vec<(
        <<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio as Gpio>::PinId,
        PinFunction,
    )>,
    led_type: parse::capsules::led::LedType,
) -> cursive::views::LinearLayout {
    let view = pin_list_disabled::<C>(pin_list, PinFunction::Led, "led_pins");
    let gpio_clone = Rc::clone(&gpio);
    let led_type_clone = led_type.clone();
    crate::menu::checkbox_popup(
        view,
        move |siv: &mut Cursive| {
            on_led_pin_submit::<C>(siv, Rc::clone(&gpio), led_type.clone(), false)
        },
        move |siv: &mut Cursive| {
            on_led_pin_submit::<C>(siv, Rc::clone(&gpio_clone), led_type_clone.clone(), true)
        },
    )
}

fn on_led_type_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    gpio: Rc<<C::Peripherals as DefaultPeripherals>::Gpio>,
    led_type: parse::capsules::led::LedType,
) {
    let data = siv.user_data::<Data<C>>().unwrap();
    let pin_list = data.gpio(&gpio).unwrap().pins().clone();
    crate::state::push_layer::<_, C>(siv, led_pins_popup::<C>(gpio, pin_list, led_type));
}

fn on_led_pin_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    gpio: Rc<<C::Peripherals as DefaultPeripherals>::Gpio>,
    led_type: parse::capsules::led::LedType,
    quit: bool,
) {
    let mut selected_pins_labels = Vec::new();
    siv.call_on_name("led_pins", |list_view: &mut ListView| {
        list_view.children().iter().for_each(|child| {
            if let ListChild::Row(label, view) = child {
                view.downcast_ref::<Checkbox>().map(|c| {
                    c.is_checked()
                        .then(|| selected_pins_labels.push(label.clone()))
                });
            }
        })
    });

    let data = siv.user_data::<Data<C>>().unwrap();
    let mut selected_pins = Vec::new();
    if let Some(pins) = gpio.pins() {
        pins.as_ref().iter().for_each(|pin| {
            // Convert from label to PinId.
            selected_pins_labels
                .contains(&format!("{}", pin))
                .then(|| selected_pins.push(*pin));
        });
    }

    let mut unselected_pins = Vec::new();
    for (pin, pin_function) in data.gpio(&gpio).unwrap().pins() {
        if *pin_function == PinFunction::Led && !selected_pins.contains(pin) {
            unselected_pins.push(*pin);
        }
    }

    // For each previously selected pin that got unselected,
    // update its status in the internal configurator data.
    unselected_pins.iter().for_each(|pin| {
        data.change_pin_status(Rc::clone(&gpio), *pin, PinFunction::None);
    });

    // For each selected pin, update its status in the internal
    // configurator data.
    selected_pins.iter().for_each(|pin| {
        data.change_pin_status(Rc::clone(&gpio), *pin, PinFunction::Led);
    });

    if selected_pins.is_empty() {
        data.platform.remove_led();
    } else {
        data.platform.update_led(led_type, selected_pins);
    }

    if quit {
        on_quit_submit::<C>(siv);
    } else {
        on_exit_submit::<C>(siv);
    }
}
