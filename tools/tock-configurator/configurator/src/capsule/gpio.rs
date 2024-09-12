// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use crate::items::ToMenuItem;
use crate::menu::{capsule_popup, checkbox_popup, pin_list_disabled};
use crate::state::{on_exit_submit, on_quit_submit, Data, GpioMap, PinFunction};
use crate::views;
use cursive::views::{Checkbox, ListChild, ListView};
use parse::peripherals::{Chip, DefaultPeripherals, Gpio};
use std::rc::Rc;

use super::ConfigMenu;

#[derive(Debug)]
pub(crate) struct GpioConfig;

impl ConfigMenu for GpioConfig {
    /// Menu for configuring the GPIO capsule.
    fn config<C: Chip + 'static + serde::ser::Serialize>(
        chip: Rc<C>,
    ) -> cursive::views::LinearLayout {
        let gpio_peripherals = Vec::from(chip.peripherals().gpio().unwrap());
        // If we have at least one GPIO peripheral, we make a list with it.
        capsule_popup::<C, _>(views::select_menu(
            Vec::from(gpio_peripherals)
                .into_iter()
                .map(|elem| elem.to_menu_item())
                .collect(),
            |siv, submit| on_gpio_capsule_submit::<C>(siv, Rc::clone(submit)),
        ))
    }
}

/// Continue to the pin selection menu after choosing a GPIO peripheral.
fn on_gpio_capsule_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    submit: Rc<<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio>,
) {
    siv.pop_layer();
    let data = siv.user_data::<Data<C>>().unwrap();
    // This never panics because the GPIO will always exist.
    let pin_list = data.gpio(&submit).unwrap().pins().clone();

    siv.add_layer(gpio_pins_popup::<C>(submit, pin_list));
}

/// Menu with a list of the pins from the selected GPIO.
fn gpio_pins_popup<C: Chip + 'static + serde::ser::Serialize>(
    gpio: Rc<<C::Peripherals as DefaultPeripherals>::Gpio>,
    pin_list: GpioMap<C>,
) -> cursive::views::LinearLayout {
    let view = pin_list_disabled::<C>(pin_list, PinFunction::Gpio, "gpio_pins");
    let gpio_clone = Rc::clone(&gpio);
    checkbox_popup::<_, _, _>(
        view,
        move |siv| on_gpio_pin_submit::<C>(siv, false, Rc::clone(&gpio)),
        move |siv| on_gpio_pin_submit::<C>(siv, true, Rc::clone(&gpio_clone)),
    )
}

/// Configure a GPIO capsule based on the selected pins.
fn on_gpio_pin_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    quit: bool,
    gpio: Rc<<C::Peripherals as DefaultPeripherals>::Gpio>,
) {
    // Get the selected pins' labels from the configuration menu.
    let mut selected_pins_labels = Vec::new();

    // Retrieve the selected pins from the gpio pins list.
    siv.call_on_name("gpio_pins", |list_view: &mut ListView| {
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
    // The newly selected pins and the newly removed pins by the user.
    let mut selected_pins = Vec::new();

    if let Some(pins) = gpio.pins() {
        pins.as_ref().iter().for_each(|pin| {
            // Convert from label to PinId.
            selected_pins_labels
                .contains(&format!("{}", pin))
                .then(|| selected_pins.push(*pin));
        });
    }

    // Create a list with all the previously selected pins that
    // are now unselected.
    let mut unselected_pins = Vec::new();
    for (pin, pin_function) in data.gpio(&gpio).unwrap().pins() {
        if *pin_function == PinFunction::Gpio && !selected_pins.contains(pin) {
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
        data.change_pin_status(Rc::clone(&gpio), *pin, PinFunction::Gpio);
    });

    if selected_pins.is_empty() {
        data.platform.remove_gpio();
    } else {
        data.platform.update_gpio(selected_pins);
    }

    if quit {
        on_quit_submit::<C>(siv);
    } else {
        on_exit_submit::<C>(siv);
    }
}
