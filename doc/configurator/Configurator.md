The Configurator crate
======================

The `configurator/` crate contains the TUI (Terminal User Interface) menu used for visually configuring a platform.

This part of the configuration process is meant to be as agnostic as possible to the Tock-specific implementations.

The application saves the configuration into a JSON file named `.config.json`.

The TUI library that was chosen for the configurator is [`cursive`](https://github.com/gyscos/cursive) because it has great Linux compatibility and flexibility.

## Current status

The menu items are currently: 
- capsules (configuration menus for the Tock capsules)
- kernel resources (configuration menus for the resources of the Tock kernel)
- syscall filter (configuration menu to choose whether to use a syscall filter or not)
- processes (configuration menu for the number of processes)
- stack memory (configuration menu for the stack memory size)

## File structure

- `main.rs`: entry point of the configurator. It starts the TUI.
- `lib.rs`: exposes the modules.
- `menu.rs`: provides general (as in not for capsules) menus to be used in the configuration of Tock.
- `state.rs`: has the functions that handle the internal state of the configurator (details about its functions can be found [here](#staters-functions)).
- The `capsule` module: contains the configuration menus and logic for each Tock capsule.
- The `utils` module: contains different macros and items used for the TUI.

## Implementation details

### The `Data` struct

The main structure of the Configurator is the `Data` struct. This represent the inner data that needs to be kept by the Cursive instance during the configuration process.

```rust
/// Inner data to be kept by Cursive.
pub(crate) struct Data<C: Chip> {
    /// The platform configuration.
    pub(crate) platform: parse::Configuration<C::Peripherals>,

    /// The chip that the platform configuration is based on.
    pub(crate) chip: Rc<C>,

    /// The view stack.
    views: ViewStack,

    /// List of pins with their usage.
    pub gpio_list: Option<Vec<GpioHelper<C>>>,
}
```

The members of the struct are:

- `platform` is a struct that keeps the configuration details (TODO: maybe explain the struct here?).
- `chip` is a reference to the chip model that was chosen in the first step of the configuration. This will be later used in the configuration of the capsules to get a list of all the available peripherals so that the user can choose which to use.
- `views` represents a stack of the past views that can be used to go back in the configuration process.
- `gpio_list` represents a vector of `GpioHelper` instances (which will be detailed in this [section](#the-gpiohelper-struct)). This is an option because some chips might not have GPIO support, so this list would be useless.

#### Associated functions

```rust
pub(crate) fn new(chip: C) -> Data<C> 

/// Add a view to the view stack.
pub(crate) fn push_view(&mut self, view: Box<dyn cursive::View>)

/// Pop view from the view stack.
pub(crate) fn pop_view(&mut self) -> Option<Box<dyn cursive::View>>

/// Take the port and returns the helper struct for it.
pub fn gpio(
    &self,
    gpio: &<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio,
) -> Option<&GpioHelper<C>>

/// Change the pin status that is stored inside the configurator
/// inner state.
pub fn change_pin_status(
    &mut self,
    gpio: Rc<<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio>,
    searched_pin: <<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio as Gpio>::PinId,
    status: PinFunction,
)
```

- The `new` function creates a new `Data` instance from a chip.

- The `push_view` and `pop_view` functions operate on the `ViewStack` (which is detailed [here](#the-viewstack-type)), these being the most basic stack functions.

- The `gpio` function searches for a certain GPIO bus and returns an immutable reference to its `GpioHelper` (which is detailed [here](#the-gpiohelper-struct)).

- The `change_pin_status` function changes the usage of the selected pin by search all the pins from the `GpioHelper` until finding the right one.

#### The `ViewStack` type

`ViewStack` represents a vector of Cursive views that is used as a stack (by using `Vec::push` and `Vec::pop`):

```rust
pub(crate) type ViewStack = Vec<Box<dyn cursive::View>>
```

### The `GpioHelper` struct

We want to keep the state of the GPIO pins to avoid selecting the same pin for two different so we will use a struct to keep track of the usage:

```rust
#[derive(Debug)]
pub struct GpioHelper<C: Chip> {
    pub gpio: Rc<<C::Peripherals as DefaultPeripherals>::Gpio>,
    pub pins: GpioMap<C>,
}
```

The members of the struct are:

- `gpio` represents the GPIO bus that the helper is built around. It is used as an identifier when searching for a certain GPIO bus.
- `pins` represents a list of the pins from the GPIO bus, paired with their usage.

#### Associated functions

```rust
pub(crate) fn new(gpio: Rc<<C::Peripherals as DefaultPeripherals>::Gpio>) -> Self

pub fn pins(&self) -> &GpioMap<C>
```

- The `new` function creates a new `GpioHelper` instance from a `Gpio` reference by getting a list of all its pins and initializing them with the `PinFunction::None` variant.

- The `pins` function returns a reference to the pin vector.

#### The `GpioMap` type and the `PinFunction` enum

The `GpioMap` type represents an alias for a vector of pairs composed of a pin ID and its usage:

```rust
pub(crate) type GpioMap<C> = Vec<(
    <<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio as Gpio>::PinId,
    PinFunction,
)>
```

`PinFunction` is an enum that represents the possible usage scenarios for the pin:

```rust
pub enum PinFunction {
    None,
    Led,
    Button,
    Gpio,
}
```

### `state.rs` functions

- #### `push_layer`
```rust
/// Push a layer to the view stack.
pub(crate) fn push_layer<
    V: cursive::view::IntoBoxedView + 'static,
    C: Chip + 'static + serde::ser::Serialize,
>(
    siv: &mut cursive::Cursive,
    layer: V,
)
```

The `push_layer` function takes the current Cursive view, adds it to the ViewStack, then displays the view given as the `layer` parameter.

- #### `on_chip_submit`
```rust
/// Initialize a board configuration session based on the submitted chip.
pub(crate) fn on_chip_submit(siv: &mut cursive::Cursive, submit: &items::SupportedChip)
```

The `on_chip_submit` function initializes the inner Cursive data based on the chip that was selected at the start of the configuration process.

- #### `on_scheduler_submit`
```rust
/// Update the inner data based on the scheduler type that was selected.
pub(crate) fn on_scheduler_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &SyscallFilterType,
)
```

The `on_scheduler_submit` function updates the inner data that is stored in the Cursive instance based on the type of scheduler that was selected by the user.

- #### `on_syscall_filter_submit`
```rust
/// Update the inner data based on the syscall filter that was selected.
pub(crate) fn on_syscall_filter_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &SyscallFilterType,
)
```

The `on_syscall_filter_submit` function updates the inner data that is stored in the Cursive instance based on the type of scheduler that was selected by the user.

- #### `on_config_submit`
```rust
/// Open a new configuration window based on the submitted config field.
pub(crate) fn on_config_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &items::ConfigurationField,
)
```

The `on_config_submit` function opens a new configuration window based on the desired configuration field.

- #### `on_kernel_resource_submit`
```rust
/// Open the corresponding config window based on the submitted kernel resource.
pub(crate) fn on_kernel_resource_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &items::KernelResources,
)
```

The `on_kernel_resource_submit` function opens a new configuration window based on the chosen kernel resource to be configured.

- #### `on_capsule_submit`
```rust
/// Open the corresponding config window based on the submitted capsule.
pub(crate) fn on_capsule_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
    submit: &items::SupportedCapsule,
)
```

The `on_capsule_submit` function opens a new configuration window based on the chosen capsule to be configured.

- #### `on_exit_submit`
```rust
/// Exit the current window and go back to the previous one.
pub(crate) fn on_exit_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
)
```

The `on_exit_submit` function closes the current view and opens the previous view. It does this by using the `ViewStack`.

- #### `on_quit_submit`
```rust
/// Exit the current window and go to the "save to JSON" menu.
pub(crate) fn on_quit_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
)
```

The `on_quit_submit` function closes the current view and opens the menu for saving the configuration in the JSON format.

- #### `on_name_submit`
```rust
/// Write to the JSON file and quit the configurator.
pub(crate) fn on_name_submit<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    name: &str,
)
```

The `on_name_submit` function calls the [`write_json`](#write_json) function and quits the configurator.

- #### `on_count_submit_proc`
```rust
/// Save the process count to use in the JSON.
pub(crate) fn on_count_submit_proc<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    name: &str,
)
```

The `on_count_submit_proc` function updates the data that is stored inside the Cursive instance with the process count that was submitted by the user.

- #### `on_count_submit_stack`
```rust
/// Save the stack memory size to use in the JSON.
pub(crate) fn on_count_submit_stack<C: Chip + 'static + serde::Serialize>(
    siv: &mut cursive::Cursive,
    name: &str,
)
```

The `on_count_submit_stack` function updates the data that is stored inside the Cursive instance with the stack memory size that was submitted by the user. This function also provides the user with the possibility to enter the size as an hexadecimal number by appending the `0x` prefix to the number.

- #### `write_json`
```rust
/// Write the contents of the inner Data to a JSON file
pub(crate) fn write_json<C: Chip + 'static + serde::ser::Serialize>(data: &mut Data<C>)
```

The `write_json` function serializes the contents of `Data::platform` then writes the result to a JSON file named `.config.json`.

- #### `on_save_submit`

```rust
/// Take the board identifier name then save the configuration using it
pub(crate) fn on_save_submit<C: Chip + 'static + serde::ser::Serialize>(
    siv: &mut cursive::Cursive,
)
```

The `on_save_submit` function gets the submitted board identifier name and writes the configuration to a JSON file with it. 
