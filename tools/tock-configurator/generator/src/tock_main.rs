// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright OxidOS Automotive SRL 2024
//
// Author: Irina Nita <irina.nita@oxidos.io>
// Author: Darius Jipa <darius.jipa@oxidos.io>

use parse::{context::Context, Chip, Component, Ident};
use proc_macro2::Literal;
use quote::{format_ident, quote};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::rc::Rc;

/// Wrapper struct for the context of the Tock platform's `main.rs`, used
/// for parsing the configuration file and generating the code.
///
/// Each main is generic over the chip that the platform is built on.
pub struct TockMain<C: Chip> {
    context: Context<C>,
}

impl<C: Chip + 'static> TockMain<C> {
    /// Create a new [`TockMain`] instance from a serialized configuration, given the path for
    /// the JSON file and the chip that the configuration is based on.
    pub fn from_json<P: AsRef<Path>>(chip: C, path: P) -> Result<Self, Box<dyn Error>> {
        let mut buf = String::new();
        File::open(path)?.read_to_string(&mut buf)?;

        // Retrieve the base user-provided configuration.
        let config = serde_json::from_str::<parse::Configuration<C::Peripherals>>(&buf)?;

        // Parse the resulted configuration for generating a context.
        let context = Context::from_config(chip, config)?;

        Ok(Self { context })
    }

    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn Error>> {
        let syntax_tree = syn::parse2(self.main_rs()?)?;
        std::fs::File::create(path)?.write_all(prettyplease::unparse(&syntax_tree).as_bytes())?;

        Ok(())
    }

    /// The token stream of the `main.rs` file for the platform.
    fn main_rs(&self) -> Result<proc_macro2::TokenStream, Box<dyn Error>> {
        let imports = self.imports();
        let global_declarations = self.global_declarations()?;
        let board_struct_def = self.struct_definition()?;
        let setup = self.setup()?;
        let main = self.main();

        Ok(quote! {
            //! GENERATED BY TOCKOS CONFIGURATOR.
            #imports
            #global_declarations
            #board_struct_def
            #setup
            #main
        })
    }

    /// Minimum needed imports.
    fn imports(&self) -> proc_macro2::TokenStream {
        quote! {
            #![no_std]
            #![cfg_attr(not(doc), no_main)]

            use kernel::component::Component;
            use kernel::platform::{KernelResources, SyscallDriverLookup};

            pub mod io;
            // use kernel::utilities::registers::interfaces::ReadWriteable as _;
        }
    }

    /// The `main` function of the module. The whole logic is moved to the `setup`
    /// function.
    fn main(&self) -> proc_macro2::TokenStream {
        let process_count = Literal::usize_unsuffixed(self.context.process_count);
        quote! {
            #[no_mangle]
            pub unsafe fn main() {
                let __main_loop_capability = kernel::create_capability!(kernel::capabilities::MainLoopCapability);
                let (board_kernel, platform, chip) = setup();
                board_kernel.kernel_loop(
                    &platform,
                    chip,
                    None::<&kernel::ipc::IPC<#process_count>>,
                    &__main_loop_capability,
                );
            }
        }
    }

    /// Constants and statics.
    fn global_declarations(&self) -> Result<proc_macro2::TokenStream, Box<dyn Error>> {
        let (chip_type, process_count, stack_size) = (
            self.context.chip.ty()?,
            self.context.process_count,
            self.context.stack_size,
        );

        Ok(quote! {
            pub const NUM_PROCS: usize = #process_count;
            const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy = capsules_system::process_policies::PanicFaultPolicy {};
            static mut PROCESSES: [Option<&'static dyn kernel::process::Process>; NUM_PROCS] = [None; NUM_PROCS];
            static mut PROCESS_PRINTER: Option<&'static capsules_system::process_printer::ProcessPrinterText> = None;
            static mut CHIP: Option<&'static #chip_type> = None;

            #[no_mangle]
            #[link_section = ".stack_buffer"]
            pub static mut STACK_MEMORY: [u8; #stack_size] = [0; #stack_size];
        })
    }

    /// Board's setup function.
    fn setup(&self) -> Result<proc_macro2::TokenStream, Box<dyn Error>> {
        let (platform_ty, platform_ident): (_, proc_macro2::TokenStream) = (
            self.context.platform.ty()?,
            self.context.platform.ident()?.parse().unwrap(),
        );

        let chip_ty = self.context.chip.ty()?;
        let chip_ident = format_ident!("{}", self.context.chip.ident()?);

        // Stack for the traversed nodes.
        // Contains the "roots" of the graph, i.e. the platform, the chip.
        let mut stack: Vec<Rc<dyn Component>> =
            vec![self.context.platform.clone(), self.context.chip.clone()];

        // Sort the initialization dependencies.
        let initializations = crate::util::topological_sort(&mut stack)?;

        // Inject the custom code for a chip/arch? Like enabling interrupts?
        Ok(quote! {
            unsafe fn setup() -> (
            &'static kernel::Kernel,
            #platform_ty,
            &'static #chip_ty) {
                let board_kernel = kernel::static_init!(kernel::Kernel, kernel::Kernel::new(&*core::ptr::addr_of!(PROCESSES)));
                #(#initializations)*

            let __process_management_capability =
                kernel::create_capability!(kernel::capabilities::ProcessManagementCapability);
                extern "C" {
                    static _sapps: u8;
                    static _eapps: u8;
                    static mut _sappmem: u8;
                    static _eappmem: u8;
                }

                kernel::process::load_processes(
                    board_kernel,
                    #chip_ident,
                    core::slice::from_raw_parts(
                        core::ptr::addr_of!(_sapps),
                        core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
                    ),
                    core::slice::from_raw_parts_mut(
                        core::ptr::addr_of_mut!(_sappmem),
                        core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
                    ),
                    &mut *core::ptr::addr_of_mut!(PROCESSES),
                    &FAULT_RESPONSE,
                    &__process_management_capability,
                )
                .unwrap_or_else(|err| {
                    kernel::debug!("Error loading processes!");
                    kernel::debug!("{:?}", err);
                });

                (board_kernel, #platform_ident, #chip_ident)
            }
        })
    }

    /// Definition of the board's struct and implementations of the needed traits.
    fn struct_definition(&self) -> Result<proc_macro2::TokenStream, Box<dyn Error>> {
        let (scheduler_timer_id, scheduler_timer_type) = (
            format_ident!("{}", self.context.chip.systick()?.ident()?),
            self.context.chip.systick()?.ty()?,
        );
        let board_ty = self.context.platform.ty()?;
        let (scheduler_id, scheduler_ty) = (
            format_ident!("{}", self.context.platform.scheduler.ident()?),
            self.context.platform.scheduler.ty()?,
        );

        let chip_ty = self.context.chip.ty()?;

        let capsules = &self.context.platform.capsules;

        let mut capsules_identifiers = Vec::new();
        let mut capsules_types = Vec::new();
        let mut capsules_driver_nums = Vec::new();

        for capsule in capsules.iter() {
            capsules_identifiers.push(format_ident!("{}", capsule.ident()?.to_string()));
            capsules_types.push(capsule.ty()?);
            capsules_driver_nums.push(capsule.driver_num());
        }

        Ok(quote! {
            struct #board_ty {
                #(#capsules_identifiers: &'static #capsules_types,)*
                #scheduler_id: &'static #scheduler_ty,
                #scheduler_timer_id: #scheduler_timer_type
            }

            impl SyscallDriverLookup for #board_ty {
                fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
                where
                    F: FnOnce(Option<&dyn kernel::syscall::SyscallDriver>) -> R,
                {
                    match driver_num {
                        #( #capsules_driver_nums => f(Some(self.#capsules_identifiers)),)*
                        _ => f(None),
                    }
                }
            }

            impl KernelResources<#chip_ty> for #board_ty {
                type SyscallDriverLookup = Self;
                type SyscallFilter = ();
                type ProcessFault = ();
                type Scheduler = #scheduler_ty;
                type SchedulerTimer = #scheduler_timer_type;
                type WatchDog = ();
                type ContextSwitchCallback = ();

                fn syscall_driver_lookup(&self) -> &Self::SyscallDriverLookup {
                    self
                }
                fn syscall_filter(&self) -> &Self::SyscallFilter {
                    &()
                }
                fn process_fault(&self) -> &Self::ProcessFault {
                    &()
                }
                fn scheduler(&self) -> &Self::Scheduler {
                    &self.#scheduler_id
                }
                fn scheduler_timer(&self) -> &Self::SchedulerTimer {
                    &self.#scheduler_timer_id
                }
                fn watchdog(&self) -> &Self::WatchDog {
                    &()
                }
                fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
                    &()
                }
            }
        })
    }
}
