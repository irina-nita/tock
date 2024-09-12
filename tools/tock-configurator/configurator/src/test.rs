// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: microbit Jipa <microbit.jipa@oxidos.io>

use std::{cell::RefCell, num::NonZeroUsize, rc::Rc};

use cursive::{
    align::HAlign, backends::puppet::observed::ObservedScreen, backends::puppet::Backend, event::*,
    traits::*, views::*, Cursive, CursiveRunner, Vec2, reexports::crossbeam_channel
};

use parse::{Chip, DefaultPeripherals, LedType, SchedulerType, SyscallFilterType};

use crate::{
    items::{ConfigurationField, KernelResources}, menu::{chip_select, init_configurator}, state::Data, utils::items::SupportedCapsule, views
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

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::RNG as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::RNG).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Rng { rng } = data.platform.capsule(&parse::config::Index::RNG).unwrap() {
            let rng = Rc::clone(rng);
            assert_eq!([rng], chip.peripherals().rng().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::RNG).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Rng { rng } = data.platform.capsule(&parse::config::Index::RNG).unwrap() {
            let rng = Rc::clone(rng);
            assert_eq!([rng], chip.peripherals().rng().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::RNG).is_none());
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn temperature() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::TEMPERATURE as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::TEMPERATURE).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Temperature { temp } = data.platform.capsule(&parse::config::Index::TEMPERATURE).unwrap() {
            let temp = Rc::clone(temp);
            assert_eq!([temp], chip.peripherals().temp().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::TEMPERATURE).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Temperature { temp } = data.platform.capsule(&parse::config::Index::TEMPERATURE).unwrap() {
            let temp = Rc::clone(temp);
            assert_eq!([temp], chip.peripherals().temp().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::TEMPERATURE).is_none());
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn spi() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::SPI as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::SPI).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Spi { spi } = data.platform.capsule(&parse::config::Index::SPI).unwrap() {
            let spi = Rc::clone(spi);
            assert_eq!([spi], chip.peripherals().spi().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::SPI).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Spi { spi } = data.platform.capsule(&parse::config::Index::SPI).unwrap() {
            let spi = Rc::clone(spi);
            assert_eq!([spi], chip.peripherals().spi().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::SPI).is_none());
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn i2c() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::I2C as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::I2C).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::I2c { i2c } = data.platform.capsule(&parse::config::Index::I2C).unwrap() {
            let i2c = Rc::clone(i2c);
            assert_eq!([i2c], chip.peripherals().i2c().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::I2C).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::I2c { i2c } = data.platform.capsule(&parse::config::Index::I2C).unwrap() {
            let i2c = Rc::clone(i2c);
            assert_eq!([i2c], chip.peripherals().i2c().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::I2C).is_none());
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn kv_driver() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::KV_DRIVER as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::KV_DRIVER).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::KvDriver { flash } = data.platform.capsule(&parse::config::Index::KV_DRIVER).unwrap() {
            let flash = Rc::clone(flash);
            assert_eq!([flash], chip.peripherals().flash().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::KV_DRIVER).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::KvDriver { flash } = data.platform.capsule(&parse::config::Index::KV_DRIVER).unwrap() {
            let flash = Rc::clone(flash);
            assert_eq!([flash], chip.peripherals().flash().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::KV_DRIVER).is_none());
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn alarm() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::ALARM as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::ALARM).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Alarm { timer } = data.platform.capsule(&parse::config::Index::ALARM).unwrap() {
            let timer = Rc::clone(timer);
            assert_eq!([timer], chip.peripherals().timer().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::ALARM).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Alarm { timer } = data.platform.capsule(&parse::config::Index::ALARM).unwrap() {
            let timer = Rc::clone(timer);
            assert_eq!([timer], chip.peripherals().timer().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::ALARM).is_none());
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn ble() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::BLE as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::BLE).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::BleRadio { ble, timer } = data.platform.capsule(&parse::config::Index::BLE).unwrap() {
            let ble = Rc::clone(ble);
            let timer = Rc::clone(timer);
            assert_eq!([ble], chip.peripherals().ble().unwrap());
            assert_eq!([timer], chip.peripherals().timer().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::BLE).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::BleRadio { ble, timer } = data.platform.capsule(&parse::config::Index::BLE).unwrap() {
            let ble = Rc::clone(ble);
            let timer = Rc::clone(timer);
            assert_eq!([ble], chip.peripherals().ble().unwrap());
            assert_eq!([timer], chip.peripherals().timer().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::BLE).is_none());
    }

    tabs_and_enters(&mut s, 3, 3);
    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn lsm303agr() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::LSM303AGR as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);
    
    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::LSM303AGR).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Lsm303agr { i2c, ..} = data.platform.capsule(&parse::config::Index::LSM303AGR).unwrap() {
            let i2c = Rc::clone(i2c);
            assert_eq!([i2c], chip.peripherals().i2c().unwrap());
        } else {
            panic!("Wrong capsule set!");
        }
    }

    select_item(&mut s, SupportedCapsule::LSM303AGR as usize);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::LSM303AGR).is_none());
    }

    tabs_and_enters(&mut s, 2, 1);

    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn gpio() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::GPIO as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 43, 1);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::GPIO).is_some());
        if let parse::config::Capsule::Gpio { pins } = data.platform.capsule(&parse::config::Index::GPIO).unwrap() {
            assert_eq!(Vec::from([mock::gpio::PinIds::P0_00]), *pins);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);


    tabs_and_enters(&mut s, 42, 1);
    
    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::GPIO).is_none());
    }

    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn led() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::LED as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 43, 1);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::LED).is_some());
        if let parse::config::Capsule::Led { led_type, pins } = data.platform.capsule(&parse::config::Index::LED).unwrap() {
            assert_eq!(Vec::from([mock::gpio::PinIds::P0_00]), *pins);
            assert_eq!(*led_type, LedType::LedHigh);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 42, 1);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::LED).is_none());
    }

    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn gpio_and_led() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::GPIO as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 43, 1);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::GPIO).is_some());
        if let parse::config::Capsule::Gpio { pins } = data.platform.capsule(&parse::config::Index::GPIO).unwrap() {
            assert_eq!(Vec::from([mock::gpio::PinIds::P0_00]), *pins);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    go_up(&mut s, SupportedCapsule::GPIO as usize);

    select_item(&mut s, SupportedCapsule::LED as usize);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Enter);

    tabs_and_enters(&mut s, 41, 1);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::LED).is_some());
        if let parse::config::Capsule::Led { led_type, pins } = data.platform.capsule(&parse::config::Index::LED).unwrap() {
            assert_eq!(Vec::from([mock::gpio::PinIds::P0_01]), *pins);
            assert_eq!(*led_type, LedType::LedHigh);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn console() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::CONSOLE as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::CONSOLE).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Console { uart, baud_rate } = data.platform.capsule(&parse::config::Index::CONSOLE).unwrap() {
            let uart = Rc::clone(uart);
            assert_eq!([uart], chip.peripherals().uart().unwrap());
            assert_eq!(*baud_rate, 112500);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::CONSOLE).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Console { uart, baud_rate } = data.platform.capsule(&parse::config::Index::CONSOLE).unwrap() {
            let uart = Rc::clone(uart);
            assert_eq!([uart], chip.peripherals().uart().unwrap());
            assert_eq!(*baud_rate, 112500);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::CONSOLE).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Console { uart, baud_rate } = data.platform.capsule(&parse::config::Index::CONSOLE).unwrap() {
            let uart = Rc::clone(uart);
            assert_eq!([uart], chip.peripherals().uart().unwrap());
            assert_eq!(*baud_rate, 115200);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.write_char('t');
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::CONSOLE).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Console { uart, baud_rate } = data.platform.capsule(&parse::config::Index::CONSOLE).unwrap() {
            let uart = Rc::clone(uart);
            assert_eq!([uart], chip.peripherals().uart().unwrap());
            assert_eq!(*baud_rate, 115200);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::CONSOLE).is_none());
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn aes() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::AES as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::AES).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Aes { aes, number_of_blocks } = data.platform.capsule(&parse::config::Index::AES).unwrap() {
            let aes = Rc::clone(aes);
            assert_eq!([aes], chip.peripherals().aes().unwrap());
            assert_eq!(*number_of_blocks, 7);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::AES).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Aes { aes, number_of_blocks } = data.platform.capsule(&parse::config::Index::AES).unwrap() {
            let aes = Rc::clone(aes);
            assert_eq!([aes], chip.peripherals().aes().unwrap());
            assert_eq!(*number_of_blocks, 7);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::AES).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Aes { aes, number_of_blocks } = data.platform.capsule(&parse::config::Index::AES).unwrap() {
            let aes = Rc::clone(aes);
            assert_eq!([aes], chip.peripherals().aes().unwrap());
            assert_eq!(*number_of_blocks, 7);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Backspace);
    s.write_char('t');
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::AES).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Aes { aes, number_of_blocks } = data.platform.capsule(&parse::config::Index::AES).unwrap() {
            let aes = Rc::clone(aes);
            assert_eq!([aes], chip.peripherals().aes().unwrap());
            assert_eq!(*number_of_blocks, 7);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::AES).is_none());
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn hmac() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::HMAC as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::HMAC).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Hmac { hmac, length } = data.platform.capsule(&parse::config::Index::HMAC).unwrap() {
            let hmac = Rc::clone(hmac);
            assert_eq!([hmac], chip.peripherals().hmac().unwrap());
            assert_eq!(*length, 16);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::HMAC).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Hmac { hmac, length } = data.platform.capsule(&parse::config::Index::HMAC).unwrap() {
            let hmac = Rc::clone(hmac);
            assert_eq!([hmac], chip.peripherals().hmac().unwrap());
            assert_eq!(*length, 16);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::HMAC).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Hmac { hmac, length } = data.platform.capsule(&parse::config::Index::HMAC).unwrap() {
            let hmac = Rc::clone(hmac);
            assert_eq!([hmac], chip.peripherals().hmac().unwrap());
            assert_eq!(*length, 16);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.write_char('t');
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::HMAC).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Hmac { hmac, length } = data.platform.capsule(&parse::config::Index::HMAC).unwrap() {
            let hmac = Rc::clone(hmac);
            assert_eq!([hmac], chip.peripherals().hmac().unwrap());
            assert_eq!(*length, 16);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::HMAC).is_none());
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn flash() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Capsules as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, SupportedCapsule::FLASH as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::FLASH).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Flash { flash, buffer_size } = data.platform.capsule(&parse::config::Index::FLASH).unwrap() {
            let flash = Rc::clone(flash);
            assert_eq!([flash], chip.peripherals().flash().unwrap());
            assert_eq!(*buffer_size, 512);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    tabs_and_enters(&mut s, 2, 1);

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::FLASH).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Flash { flash, buffer_size } = data.platform.capsule(&parse::config::Index::FLASH).unwrap() {
            let flash = Rc::clone(flash);
            assert_eq!([flash], chip.peripherals().flash().unwrap());
            assert_eq!(*buffer_size, 512);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::FLASH).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Flash { flash, buffer_size } = data.platform.capsule(&parse::config::Index::FLASH).unwrap() {
            let flash = Rc::clone(flash);
            assert_eq!([flash], chip.peripherals().flash().unwrap());
            assert_eq!(*buffer_size, 512);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.write_char('t');
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::FLASH).is_some());
        let chip = Rc::clone(&data.chip);
        if let parse::config::Capsule::Flash { flash, buffer_size } = data.platform.capsule(&parse::config::Index::FLASH).unwrap() {
            let flash = Rc::clone(flash);
            assert_eq!([flash], chip.peripherals().flash().unwrap());
            assert_eq!(*buffer_size, 512);
        } else {
            panic!("Wrong capsule set!");
        }
    }

    go_up(&mut s, 1);
    s.hit_keystroke(Key::Enter);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert!(data.platform.capsule(&parse::config::Index::FLASH).is_none());
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}


#[test]
fn scheduler() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::KernelResources as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, KernelResources::Scheduler as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert_eq!(data.platform.scheduler, SchedulerType::RoundRobin);
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn syscall_filter() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::SysCallFilter as usize);
    s.hit_keystroke(Key::Enter);

    select_item(&mut s, 1);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert_eq!(data.platform.syscall_filter, SyscallFilterType::TbfHeaderFilterDefaultAllow);
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn processes() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::Processes as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Backspace);

    s.write_string("12");

    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert_eq!(data.platform.process_count, 12);
    }

    select_item(&mut s, ConfigurationField::Processes as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);

    s.write_string("12");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert_eq!(data.platform.process_count, 12);
    }

    select_item(&mut s, ConfigurationField::Processes as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);

    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        assert_eq!(data.platform.process_count, 4);
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn stack_memory() {
    let mut s = BasicSetup::new();

    s.hit_keystroke(Key::Enter);

    select_item(&mut s, ConfigurationField::StackMem as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);

    s.write_string("12");
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        unsafe {
            assert_eq!(data.platform.stack_size, NonZeroUsize::new_unchecked(0x12_usize));
        }
    }

    select_item(&mut s, ConfigurationField::StackMem as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.write_string("12");
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        unsafe {
            assert_eq!(data.platform.stack_size, NonZeroUsize::new_unchecked(12_usize));
        }
    }

    select_item(&mut s, ConfigurationField::StackMem as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.write_string("12");
    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        unsafe {
            assert_eq!(data.platform.stack_size, NonZeroUsize::new_unchecked(12_usize));
        }
    }

    select_item(&mut s, ConfigurationField::StackMem as usize);
    s.hit_keystroke(Key::Enter);

    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);
    s.hit_keystroke(Key::Backspace);

    s.hit_keystroke(Key::Enter);

    if let Some(data) = s.siv.user_data::<Data<mock::Chip>>() {
        unsafe {
            assert_eq!(data.platform.stack_size, NonZeroUsize::new_unchecked(0x900_usize));
        }
    }

    tabs_and_enters(&mut s, 2, 1);
    s.write_string("microbit");

    s.hit_keystroke(Key::Tab);
    s.hit_keystroke(Key::Enter);
}

#[test]
fn init_test() {
    let _ = init_configurator();
}