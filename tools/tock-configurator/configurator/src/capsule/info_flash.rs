use std::rc::Rc;

use crate::menu::capsule_popup;
use crate::state::Data;
use parse::peripherals::{Chip, DefaultPeripherals};

const PERIPHERAL: &str = "FLASH";

#[derive(Debug)]
pub(crate) struct InfoFlashConfig;

/// Menu for configuring the Flash capsule.
impl super::ConfigMenu for InfoFlashConfig {
    fn config<C: Chip + 'static + serde::ser::Serialize>(
        chip: Rc<C>,
    ) -> cursive::views::LinearLayout {
        match chip.peripherals().flash() {
            // If we have at least one flash peripheral, we make a list with it.
            Ok(flash_peripherals) => capsule_popup::<C, _>(crate::views::radio_group_with_null(
                Vec::from(flash_peripherals),
                on_flash_submit::<C>,
            )),
            // If we don't have any flash peripheral, we show a popup
            // with an error describing this.
            Err(_) => capsule_popup::<C, _>(crate::menu::no_support(PERIPHERAL)),
        }
    }
}

/// Configure a Flash info capsule based on the submitted Flash peripheral.
fn on_flash_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &Option<Rc<<<C as Chip>::Peripherals as DefaultPeripherals>::Flash>>,
) {
    if let Some(data) = siv.user_data::<Data<C>>() {
        match submit {
            Some(flash) => data.platform.update_info_flash(Rc::clone(&flash)),
            None => data.platform.remove_info_flash(),
        }
    }
}
