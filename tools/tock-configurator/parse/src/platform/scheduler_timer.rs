// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use super::peripherals::timer;
use crate::Component;
use std::rc::Rc;

#[parse_macros::component(curr, ident = "scheduler_timer")]
pub struct SchedulerTimer<T: timer::Timer + 'static> {
    virtual_mux_alarm: Rc<timer::VirtualMuxAlarm<T>>,
}

impl<T: timer::Timer + 'static> SchedulerTimer<T> {
    pub fn get(virtual_mux_alarm: Rc<timer::VirtualMuxAlarm<T>>) -> Rc<Self> {
        Rc::new(Self::new(virtual_mux_alarm))
    }
}

impl<T: timer::Timer + 'static> Component for SchedulerTimer<T> {
    fn dependencies(&self) -> Option<Vec<Rc<dyn Component>>> {
        Some(vec![self.virtual_mux_alarm.clone()])
    }
}

impl<T: timer::Timer + 'static> SchedulerTimer<T> {
    pub fn virtual_mux_alarm(&self) -> Rc<timer::VirtualMuxAlarm<T>> {
        self.virtual_mux_alarm.clone()
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct DefaultSchedulerTimer;

impl DefaultSchedulerTimer {
    pub fn new() -> Self {
        Self {}
    }
}

impl crate::Ident for DefaultSchedulerTimer {
    fn ident(&self) -> Result<String, crate::error::Error> {
        Ok(String::from("scheduler_timer"))
    }
}

impl Component for DefaultSchedulerTimer {
    fn ty(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        Ok(quote::quote!(()))
    }

    fn init_expr(&self) -> Result<proc_macro2::TokenStream, crate::Error> {
        Ok(quote::quote!(&()))
    }
}
