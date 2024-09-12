// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

/// Types of ssyscall filters available for Tock platforms.
#[derive(serde::Serialize, serde::Deserialize, Default, Debug, Clone, Copy, PartialEq)]
pub enum SyscallFilterType {
    #[default]
    None,
    TbfHeaderFilterDefaultAllow,
}

impl std::fmt::Display for SyscallFilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyscallFilterType::None => write!(f, "None"),
            SyscallFilterType::TbfHeaderFilterDefaultAllow => {
                write!(f, "TbfHeaderFilterDefaultAllow")
            }
        }
    }
}
