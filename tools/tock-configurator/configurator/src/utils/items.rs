// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

pub(crate) use parse::config::Index as SupportedCapsule;

/// Trait for a type (usually an `enum`) that can be converted to a menu
/// item as defined by cursive's `SelectView`.
pub(crate) trait ToMenuItem {
    type Item;
    fn to_menu_item(self) -> (String, Self::Item);
}

/// Enum for the top-level configuration options for a board.
#[derive(Clone, Copy)]
pub(crate) enum ConfigurationField {
    Capsules,
    KernelResources,
    SysCallFilter,
    Processes,
    StackMem,
}

impl ToMenuItem for ConfigurationField {
    type Item = Self;
    fn to_menu_item(self) -> (String, Self::Item) {
        (
            match self {
                ConfigurationField::Capsules => crate::submenu!("capsules"),
                ConfigurationField::KernelResources => crate::submenu!("kernel resources"),
                ConfigurationField::Processes => crate::submenu!("processes"),
                ConfigurationField::StackMem => crate::submenu!("stack memory"),
                ConfigurationField::SysCallFilter => crate::submenu!("syscall filter"),
            },
            self,
        )
    }
}

/// Enum for the kernel resources for a board.
#[derive(Clone, Copy)]
pub(crate) enum KernelResources {
    Scheduler,
}

impl ToMenuItem for KernelResources {
    type Item = Self;
    fn to_menu_item(self) -> (String, Self::Item) {
        (
            match self {
                KernelResources::Scheduler => crate::submenu!("scheduler"),
            },
            self,
        )
    }
}

/// Enum for supported chips by the configurator.
#[cfg(not(test))]
#[derive(Clone, Copy)]
pub(crate) enum SupportedChip {
    MicroBit,
}

/// Enum for supported chips by the configurator with the mock chip added.
/// Only for testing purposes.
#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) enum SupportedChip {
    Mock,
    MicroBit,
}

#[cfg(not(test))]
impl ToMenuItem for SupportedChip {
    type Item = Self;
    fn to_menu_item(self) -> (String, Self::Item) {
        (
            match self {
                SupportedChip::MicroBit => crate::submenu!("microbit"),
            },
            self,
        )
    }
}

#[cfg(test)]
impl ToMenuItem for SupportedChip {
    type Item = Self;
    fn to_menu_item(self) -> (String, Self::Item) {
        (
            match self {
                SupportedChip::Mock => crate::submenu!("mock"),
                SupportedChip::MicroBit => crate::submenu!("microbit"),
            },
            self,
        )
    }
}

impl<T> ToMenuItem for T
where
    T: std::fmt::Debug,
{
    type Item = Self;

    fn to_menu_item(self) -> (String, Self::Item) {
        (format!("{:?}", self).to_lowercase(), self)
    }
}
