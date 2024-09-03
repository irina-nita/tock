// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

/// The [`submenu!`] macro adds the submenu symbol `⎯⎯>` at the end of the given string.
#[macro_export]
macro_rules! submenu {
    ($name:literal) => {
        std::format!("{} -->", $name)
    };
}
