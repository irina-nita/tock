// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.
// Copyright OxidOS Automotive 2025.

//! Tock kernel for the Raspberry Pi Pico.
//!
//! It is based on RP2040SoC SoC (Cortex M0+).

#![no_std]
#![no_main]
#![deny(missing_docs)]

use capsules_core::i2c_master::I2CMasterDriver;
use components::led::LedsComponent;
use kernel::component::Component;
use kernel::hil::gpio::Configure;
use kernel::hil::gpio::FloatingState;
use kernel::hil::i2c::I2CMaster;
use kernel::hil::led::LedHigh;
use kernel::platform::{KernelResources, SyscallDriverLookup};
use kernel::syscall::SyscallDriver;
use kernel::{capabilities, create_capability, debug, static_init};
use rp2040::adc::{self, Adc};
use rp2040::chip::{Rp2040, Rp2040DefaultPeripherals};
use rp2040::gpio::{GpioFunction, RPGpio, RPGpioPin};
use rp2040::i2c::I2c;

mod io;

kernel::stack_size! {0x1500}

// State for loading and holding applications.
// How should the kernel respond when a process faults.
const FAULT_RESPONSE: capsules_system::process_policies::PanicFaultPolicy =
    capsules_system::process_policies::PanicFaultPolicy {};

type TemperatureRp2040Sensor = components::temperature_rp2040::TemperatureRp2040ComponentType<
    capsules_core::virtualizers::virtual_adc::AdcDevice<'static, rp2040::adc::Adc<'static>>,
>;
type TemperatureDriver = components::temperature::TemperatureComponentType<TemperatureRp2040Sensor>;

/// Supported drivers by the platform
pub struct RaspberryPiPico {
    base: raspberry_pi_pico::Platform,
    led: &'static capsules_core::led::LedDriver<'static, LedHigh<'static, RPGpioPin<'static>>, 1>,
    gpio: &'static capsules_core::gpio::GPIO<'static, RPGpioPin<'static>>,
    adc: &'static capsules_core::adc::AdcVirtualized<'static>,
    temperature: &'static TemperatureDriver,
    i2c: &'static capsules_core::i2c_master::I2CMasterDriver<'static, I2c<'static, 'static>>,
}

impl SyscallDriverLookup for RaspberryPiPico {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn SyscallDriver>) -> R,
    {
        match driver_num {
            capsules_core::led::DRIVER_NUM => f(Some(self.led)),
            capsules_core::gpio::DRIVER_NUM => f(Some(self.gpio)),
            capsules_core::adc::DRIVER_NUM => f(Some(self.adc)),
            capsules_extra::temperature::DRIVER_NUM => f(Some(self.temperature)),
            capsules_core::i2c_master::DRIVER_NUM => f(Some(self.i2c)),
            _ => self.base.with_driver(driver_num, f),
        }
    }
}

impl KernelResources<Rp2040<'static, Rp2040DefaultPeripherals<'static>>> for RaspberryPiPico {
    type SyscallDriverLookup = Self;
    type SyscallFilter = ();
    type ProcessFault = ();
    type Scheduler = raspberry_pi_pico::SchedulerInUse;
    type SchedulerTimer = cortexm0p::systick::SysTick;
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
        self.base.scheduler
    }
    fn scheduler_timer(&self) -> &Self::SchedulerTimer {
        &self.base.systick
    }
    fn watchdog(&self) -> &Self::WatchDog {
        &()
    }
    fn context_switch_callback(&self) -> &Self::ContextSwitchCallback {
        &()
    }
}

/// This is in a separate, inline(never) function so that its stack frame is
/// removed when this function returns. Otherwise, the stack space used for
/// these static_inits is wasted.
#[inline(never)]
pub unsafe fn start() -> (
    &'static kernel::Kernel,
    RaspberryPiPico,
    &'static rp2040::chip::Rp2040<'static, Rp2040DefaultPeripherals<'static>>,
) {
    // Initialize deferred calls very early.
    kernel::deferred_call::initialize_deferred_call_state_unsafe::<
        <raspberry_pi_pico::ChipHw as kernel::platform::chip::Chip>::ThreadIdProvider,
    >();

    let output = raspberry_pi_pico::Output::Cdc;

    // Uncomment this to use UART0 as output for console caspule/debug writer/process console
    // let output = raspberry_pi_pico::Output::Uart;

    let (board_kernel, base, peripherals, _, chip) = raspberry_pi_pico::setup(output);

    // Set the UART used for panic
    (*core::ptr::addr_of_mut!(io::WRITER)).set_uart(&peripherals.uart0);

    // LED
    let led = LedsComponent::new().finalize(components::led_component_static!(
        LedHigh<'static, RPGpioPin<'static>>,
        LedHigh::new(peripherals.pins.get_pin(RPGpio::GPIO25))
    ));

    // GPIO
    let gpio = components::gpio::GpioComponent::new(
        board_kernel,
        capsules_core::gpio::DRIVER_NUM,
        components::gpio_component_helper!(
            RPGpioPin,
            // Used for serial communication. Comment them in if you don't use serial.
            // 0 => peripherals.pins.get_pin(RPGpio::GPIO0),
            // 1 => peripherals.pins.get_pin(RPGpio::GPIO1),
            2 => peripherals.pins.get_pin(RPGpio::GPIO2),
            3 => peripherals.pins.get_pin(RPGpio::GPIO3),
            // Used for i2c. Comment them in if you don't use i2c.
            // 4 => peripherals.pins.get_pin(RPGpio::GPIO4),
            // 5 => peripherals.pins.get_pin(RPGpio::GPIO5),
            6 => peripherals.pins.get_pin(RPGpio::GPIO6),
            7 => peripherals.pins.get_pin(RPGpio::GPIO7),
            8 => peripherals.pins.get_pin(RPGpio::GPIO8),
            9 => peripherals.pins.get_pin(RPGpio::GPIO9),
            10 => peripherals.pins.get_pin(RPGpio::GPIO10),
            11 => peripherals.pins.get_pin(RPGpio::GPIO11),
            12 => peripherals.pins.get_pin(RPGpio::GPIO12),
            13 => peripherals.pins.get_pin(RPGpio::GPIO13),
            14 => peripherals.pins.get_pin(RPGpio::GPIO14),
            15 => peripherals.pins.get_pin(RPGpio::GPIO15),
            16 => peripherals.pins.get_pin(RPGpio::GPIO16),
            17 => peripherals.pins.get_pin(RPGpio::GPIO17),
            18 => peripherals.pins.get_pin(RPGpio::GPIO18),
            19 => peripherals.pins.get_pin(RPGpio::GPIO19),
            20 => peripherals.pins.get_pin(RPGpio::GPIO20),
            21 => peripherals.pins.get_pin(RPGpio::GPIO21),
            22 => peripherals.pins.get_pin(RPGpio::GPIO22),
            23 => peripherals.pins.get_pin(RPGpio::GPIO23),
            24 => peripherals.pins.get_pin(RPGpio::GPIO24),
            // LED pin
            // 25 => peripherals.pins.get_pin(RPGpio::GPIO25),

            // Uncomment to use these as GPIO pins instead of ADC pins
            // 26 => peripherals.pins.get_pin(RPGpio::GPIO26),
            // 27 => peripherals.pins.get_pin(RPGpio::GPIO27),
            // 28 => peripherals.pins.get_pin(RPGpio::GPIO28),
            // 29 => peripherals.pins.get_pin(RPGpio::GPIO29)
        ),
    )
    .finalize(components::gpio_component_static!(RPGpioPin<'static>));

    peripherals.adc.init();

    let adc_mux = components::adc::AdcMuxComponent::new(&peripherals.adc)
        .finalize(components::adc_mux_component_static!(Adc));

    let temp_sensor = components::temperature_rp2040::TemperatureRp2040Component::new(
        adc_mux,
        adc::Channel::Channel4,
        1.721,
        0.706,
    )
    .finalize(components::temperature_rp2040_adc_component_static!(
        rp2040::adc::Adc
    ));

    let temperature = components::temperature::TemperatureComponent::new(
        board_kernel,
        capsules_extra::temperature::DRIVER_NUM,
        temp_sensor,
    )
    .finalize(components::temperature_component_static!(
        TemperatureRp2040Sensor
    ));

    let adc_channel_0 = components::adc::AdcComponent::new(adc_mux, adc::Channel::Channel0)
        .finalize(components::adc_component_static!(Adc));

    let adc_channel_1 = components::adc::AdcComponent::new(adc_mux, adc::Channel::Channel1)
        .finalize(components::adc_component_static!(Adc));

    let adc_channel_2 = components::adc::AdcComponent::new(adc_mux, adc::Channel::Channel2)
        .finalize(components::adc_component_static!(Adc));

    let adc_channel_3 = components::adc::AdcComponent::new(adc_mux, adc::Channel::Channel3)
        .finalize(components::adc_component_static!(Adc));

    let adc =
        components::adc::AdcVirtualComponent::new(board_kernel, capsules_core::adc::DRIVER_NUM)
            .finalize(components::adc_syscall_component_helper!(
                adc_channel_0,
                adc_channel_1,
                adc_channel_2,
                adc_channel_3,
            ));

    let sda_pin = peripherals.pins.get_pin(RPGpio::GPIO4);
    let scl_pin = peripherals.pins.get_pin(RPGpio::GPIO5);

    sda_pin.set_function(GpioFunction::I2C);
    scl_pin.set_function(GpioFunction::I2C);

    sda_pin.set_floating_state(FloatingState::PullUp);
    scl_pin.set_floating_state(FloatingState::PullUp);

    let i2c_master_buffer = static_init!(
        [u8; capsules_core::i2c_master::BUFFER_LENGTH],
        [0; capsules_core::i2c_master::BUFFER_LENGTH]
    );

    let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);
    let i2c0 = &peripherals.i2c0;
    let i2c = static_init!(
        I2CMasterDriver<I2c<'static, 'static>>,
        I2CMasterDriver::new(
            i2c0,
            i2c_master_buffer,
            board_kernel.create_grant(
                capsules_core::i2c_master::DRIVER_NUM,
                &memory_allocation_capability
            ),
        )
    );
    i2c0.init(10 * 1000);
    i2c0.set_master_client(i2c);

    debug!("Initialization complete. Enter main loop");

    // These symbols are defined in the linker script.
    extern "C" {
        /// Beginning of the ROM region containing app images.
        static _sapps: u8;
        /// End of the ROM region containing app images.
        static _eapps: u8;
        /// Beginning of the RAM region for app memory.
        static mut _sappmem: u8;
        /// End of the RAM region for app memory.
        static _eappmem: u8;
    }

    let process_management_capability =
        create_capability!(capabilities::ProcessManagementCapability);
    kernel::process::load_processes(
        board_kernel,
        chip,
        core::slice::from_raw_parts(
            core::ptr::addr_of!(_sapps),
            core::ptr::addr_of!(_eapps) as usize - core::ptr::addr_of!(_sapps) as usize,
        ),
        core::slice::from_raw_parts_mut(
            core::ptr::addr_of_mut!(_sappmem),
            core::ptr::addr_of!(_eappmem) as usize - core::ptr::addr_of!(_sappmem) as usize,
        ),
        &FAULT_RESPONSE,
        &process_management_capability,
    )
    .unwrap_or_else(|err| {
        debug!("Error loading processes!");
        debug!("{:?}", err);
    });

    let platform = RaspberryPiPico {
        base,
        led,
        gpio,
        adc,
        temperature,
        i2c,
    };

    (board_kernel, platform, chip)
}

/// Main function called after RAM initialized.
#[no_mangle]
pub unsafe fn main() {
    let main_loop_capability = create_capability!(capabilities::MainLoopCapability);

    let (board_kernel, platform, chip) = start();

    board_kernel.kernel_loop(
        &platform,
        chip,
        Some(&platform.base.ipc),
        &main_loop_capability,
    );
}
