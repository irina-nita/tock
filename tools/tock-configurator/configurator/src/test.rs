// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: microbit Jipa <microbit.jipa@oxidos.io>

use std::cell::RefCell;

use cursive::{
    align::HAlign, backends::puppet::observed::ObservedScreen, backends::puppet::Backend, event::*,
    traits::*, views::*, Cursive, CursiveRunner, Vec2, reexports::crossbeam_channel
};

use crate::{
    items::{ConfigurationField, KernelResources},
    menu::chip_select,
    utils::items::{SupportedCapsule, SupportedChip},
    views,
};

pub struct BasicSetup {
    siv: CursiveRunner<Cursive>,
    screen_stream: crossbeam_channel::Receiver<ObservedScreen>,
    input: crossbeam_channel::Sender<Option<Event>>,
    last_screen: RefCell<Option<ObservedScreen>>,
}

impl BasicSetup {
    pub fn new() -> Self {
        let mut select = SelectView::new()
            // Center the text horizontally
            .h_align(HAlign::Center)
            // Use keyboard to jump to the pressed letters
            .autojump();

        select.insert_item_str(0, "item");
        select.insert_item_str(1, "second_item");

        let size = Vec2::new(80, 16);
        let backend = Backend::init(Some(size));
        let sink = backend.stream();
        let input = backend.input();

        // Init configurator with the default.
        let mut configurator = Cursive::new().into_runner(backend);

        configurator.set_theme(cursive::theme::Theme::retro());

        // First layer of the chip select.
        configurator.screen_mut().add_layer(
            views::main_dialog(
                chip_select(),
                None::<fn(&mut cursive::Cursive)>,
                Some(|siv: &mut Cursive| siv.quit()),
            )
            .full_width(),
        );

        input.send(Some(Event::Refresh)).unwrap();
        configurator.step();

        BasicSetup {
            siv: configurator,
            screen_stream: sink,
            input,
            last_screen: RefCell::new(None),
        }
    }

    pub fn last_screen(&self) -> Option<ObservedScreen> {
        while let Ok(screen) = self.screen_stream.try_recv() {
            self.last_screen.replace(Some(screen));
        }

        self.last_screen.borrow().clone()
    }

    pub fn dump_debug(&self) {
        if let Some(s) = self.last_screen().as_ref() {
            s.print_stdout()
        }
    }

    pub fn hit_keystroke(&mut self, key: Key) {
        self.input.send(Some(Event::Key(key))).unwrap();
        self.siv.step();
    }

    pub fn write_char(&mut self, ch: char) {
        self.input.send(Some(Event::Char(ch))).unwrap();
        self.siv.step();
    }

    pub fn write_string(&mut self, string: &str) {
        for ch in string.chars() {
            self.write_char(ch);
        }
    }
}

fn select_item(s: &mut BasicSetup, index: usize) {
    for _ in 0..index {
        s.hit_keystroke(Key::Down);
    }
}

fn go_up(s: &mut BasicSetup, index: usize) {
    for _ in 0..index {
        s.hit_keystroke(Key::Up);
    }
}

fn tabs_and_enters(s: &mut BasicSetup, tab_count: usize, enter_count: usize) {
    for _ in 0..tab_count {
        s.hit_keystroke(Key::Tab);
    }
    for _ in 0..enter_count {
        s.hit_keystroke(Key::Enter);
    }
}

#[test]
fn rng() {
    let mut s = BasicSetup::new();

    select_item(&mut s, SupportedChip::MicroBit as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::RNG as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn temperature() {
    let mut s = BasicSetup::new();

    select_item(&mut s, SupportedChip::MicroBit as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::TEMPERATURE as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn i2c() {
    let mut s = BasicSetup::new();

    select_item(&mut s, SupportedChip::MicroBit as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::I2C as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn console() {
    let mut s = BasicSetup::new();

    select_item(&mut s, SupportedChip::MicroBit as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::CONSOLE as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn flash() {
    let mut s = BasicSetup::new();

    select_item(&mut s, SupportedChip::MicroBit as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::FLASH as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn alarm() {
    let mut s = BasicSetup::new();

    select_item(&mut s, SupportedChip::MicroBit as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::ALARM as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn ble() {
    let mut s = BasicSetup::new();

    select_item(&mut s, SupportedChip::MicroBit as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::BLE as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 3, 3);
    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn gpio() {
    let mut s = BasicSetup::new();

    select_item(&mut s, SupportedChip::MicroBit as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::GPIO as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 43, 1);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 42, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 3, 3);
    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn scheduler() {
    let mut s = BasicSetup::new();

    select_item(&mut s, SupportedChip::MicroBit as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::KernelResources as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, KernelResources::Scheduler as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn syscall_filter() {
    let mut s = BasicSetup::new();

    select_item(&mut s, SupportedChip::MicroBit as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::SysCallFilter as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn processes() {
    let mut s = BasicSetup::new();

    select_item(&mut s, SupportedChip::MicroBit as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Processes as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Backspace);

    s.write_string("12");

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Processes as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);

    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn stack_memory() {
    let mut s = BasicSetup::new();

    select_item(&mut s, SupportedChip::MicroBit as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::StackMem as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);

    s.write_string("12");
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::StackMem as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.write_string("12");
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::StackMem as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);

    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}
