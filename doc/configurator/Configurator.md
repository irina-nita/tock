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
- `menu.rs`: provides general (as in not for capsules) menus to be used in the configuration of Tock (details about its functions can be found [here](#menurs-functions)).
- `state.rs`: has the functions that handle the internal state of the configurator (details about its functions can be found [here](#staters-functions)).
- The `capsule` submodule: contains the configuration menus and logic for each Tock capsule (more details can be found [here](#the-capsule-submodule)).
- The `utils` submodule: contains different macros and items used for the TUI (more details can be found [here](#the-utils-submodule)).

## Implementation details

### Configurator submodules interactions
![](/doc/configurator/assets/configurator_interactions.png)

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

### `menu.rs` functions

The `menu.rs` functions create Cursive views that serve as configuration menus for everything except the capsules (they have a separate submodule, `capsule`, that was created to facilitate adding new capsules)

- #### `chip_select` 
```rust
/// Select menu of supported chips.
pub(crate) fn chip_select() -> cursive::views::SelectView<items::SupportedChip>
```

The `chip_select` function creates a menu for selecting the chip for which the configuration is done.

- #### `capsules_menu`
```rust
/// Menu for configuring the **capsules** the board will implement.
pub(crate) fn capsules_menu<C: Chip + 'static + serde::ser::Serialize>(
) -> cursive::views::ResizedView<cursive::views::LinearLayout>
```

The `capsules_menu` function creates a menu with all the capsules for the user to choose which one to add and configure.

- #### `capsule_popup`
```rust
/// Menu for configuring a capsule.
pub(crate) fn capsule_popup<
    C: Chip + 'static + serde::ser::Serialize,
    V: cursive::view::IntoBoxedView + 'static,
>(
    view: V,
) -> cursive::views::LinearLayout
```

The `capsule_popup` function is a helper one, offering a standard way to create a capsule configuration menu.

- #### `checkbox_popup`
```rust
/// A popup with a checkbox.
pub fn checkbox_popup<
    V: cursive::view::IntoBoxedView + 'static,
    F: 'static + Fn(&mut cursive::Cursive),
    G: 'static + Fn(&mut cursive::Cursive),
>(
    view: V,
    submit_callback: F,
    quit_callback: G,
) -> cursive::views::LinearLayout
```

The `checkbox_popup` function is a helper one, offering a standard way to create a popup menu with a checkbox.

- #### `no_support`
```rust
/// Popup in case of a peripheral not being supported.
pub(crate) fn no_support(peripheral: &'static str) -> cursive::views::TextView
```

The `no_support` function creates a popup telling the user that the chip does not have support for a peripheral.

- #### `capsule_not_configured`
```rust
/// Popup in case of a dependency capsule not being configured.
pub(crate) fn capsule_not_configured(capsule: &'static str) -> cursive::views::TextView
```

The `capsule_not_configured` function creates a popup telling the user that the capsule he wants to enable needs another capsule to be enabled.

- #### `pin_list_disabled`
```rust
/// A checkbox list that has disabled entries if they can't be used.
pub(crate) fn pin_list_disabled<C: Chip>(
    pin_list: Vec<(
        <<<C as Chip>::Peripherals as DefaultPeripherals>::Gpio as Gpio>::PinId,
        PinFunction,
    )>,
    gpio_use: PinFunction,
    name: &str,
) -> ScrollView<LinearLayout>
```

The `pin_list_disabled` function creates a checkbox list for the pins in which the pins that are used for the same reason as `gpio_use` appear as checked and the pins that are used for other reasons are disabled to avoid using the same pin for multiple functions.

- #### `kernel_resources_menu` 
```rust
/// Menu for configuring the **kernel resources** the board will use.
pub(crate) fn kernel_resources_menu<C: Chip + 'static + serde::ser::Serialize>(
) -> cursive::views::ResizedView<cursive::views::LinearLayout>
```

The `kernel_resources_menu` function creates a menu with all the kernel resources for the user to choose which one to configure.

- #### `scheduler_menu`
```rust
/// Scheduler configuration menu.
pub(crate) fn scheduler_menu<C: Chip + 'static + serde::ser::Serialize>(
    current_scheduler: SchedulerType,
) -> cursive::views::ResizedView<cursive::views::LinearLayout>
```

The `scheduler_menu` function provides a configuration menu in which the user can choose the scheduler type.

- #### `syscall_filter_menu`
```rust
/// Syscall filter configuration menu.
pub(crate) fn syscall_filter_menu<C: Chip + 'static + serde::ser::Serialize>(
    current_filter: SyscallFilterType,
) -> cursive::views::ResizedView<cursive::views::LinearLayout>
```

The `syscall_filter_menu` function provides a configuration menu in which the user can choose the syscall filter.

- #### `processes_menu`
```rust
/// Process count configuration menu.
pub(crate) fn processes_menu<C: Chip + 'static + serde::ser::Serialize>(
    proc_count: usize,
) -> cursive::views::Dialog
```

The `processes_menu` function provides a configuration menu in which the user can choose the number of processes.

- #### `stack_menu`
```rust
/// Stack memory size configuration menu.
pub(crate) fn stack_menu<C: Chip + 'static + serde::ser::Serialize>(
    current_stack_size: usize,
) -> cursive::views::Dialog
```

The `stack_menu` function provides a configuration menu in which the user can choose the stack memory size.

- #### `status_bar`
```rust
/// Status bar at top.
pub(crate) fn status_bar() -> LinearLayout
```

The `status_bar` function builds the status bar that stays at the top of the screen during the configuration process.

- #### `board_config_menu`
```rust
/// Board configuration menu.
pub(crate) fn board_config_menu<C: Chip + 'static + serde::ser::Serialize>(
) -> cursive::views::ResizedView<cursive::views::LinearLayout>
```

The `board_config_menu` function creates a menu with all the configuration options.

- #### `init_configurator`
```rust
/// Build the configurator by adding the layers defined in [`crate::menu::layers`]
/// and initalizing [`crate::menu::builder::CONFIGURATION_BUILDER`].
pub fn init_configurator() -> cursive::CursiveRunnable
```

The `init_configurator` function builds the configurator by creating the initial Cursive views and displaying them.

- #### `save_dialog`
```rust
/// Menu used for saving the configuration to a JSON file.
pub fn save_dialog<C: parse::peripherals::Chip + 'static + serde::ser::Serialize>(
) -> cursive::views::LinearLayout
```

The `save_dialog` function provides the menu for saving the configuration to a JSON file.

### The `utils` submodule

The `utils` submodule contains different macros and items used for the TUI. They are grouped in the following submodules:

- #### `items.rs`

This submodule exposes an interface for the configurator menu items with the `ToMenuItem` trait:

```rust
/// Trait for a type (usually an `enum`) that can be converted to a menu
/// item as defined by cursive's `SelectView`.
pub(crate) trait ToMenuItem {
    type Item;
    fn to_menu_item(self) -> (String, Self::Item);
}
```

The `to_menu_item` function takes an item and converts it into a pair consisting of a label and the original item.

The submodule also has enums for all of the main configurator menus.

```rust
/// Enum for the top-level configuration options for a board.
#[derive(Clone, Copy)]
pub(crate) enum ConfigurationField {
    Capsules,
    KernelResources,
    SysCallFilter,
    Processes,
    StackMem,
}

/// Enum for the kernel resources for a board.
#[derive(Clone, Copy)]
pub(crate) enum KernelResources {
    Scheduler,
}

/// Enum for supported chips by the configurator.
#[derive(Clone, Copy)]
pub(crate) enum SupportedChip {
    MicroBit,
}
```

- #### `macros.rs`

This submodule exports 1 macro that is used to add an arrow (`⎯⎯>`) at the end of a given string. This is done to let the user know that the menu item will open another menu after selecting it.

```rust
/// The [`submenu!`] macro adds the submenu symbol `⎯⎯>` at the end of the given string.
#[macro_export]
macro_rules! submenu {
    ($name:literal) => {
        std::format!("{} -->", $name)
    };
}
```

- #### `views.rs`

This submodule provides generic helper functions that create views used by the configurator:

```rust
/// Create a select menu generic over the options.
pub(crate) fn select_menu<T, R, S, F>(items: Vec<(S, T)>, on_submit: F) -> SelectView<T>
where
    T: 'static,
    S: Into<String>,
    F: Fn(&mut cursive::Cursive, &T) -> R + 'static,

/// Create a list of radio buttons with the `None` option (checked).
pub(crate) fn radio_group_with_null<T, F>(items: Vec<T>, on_change: F) -> LinearLayout
where
    T: 'static + std::fmt::Display,
    F: 'static + Fn(&mut cursive::Cursive, &Option<T>),

/// Create a list of radio buttons with the `None` option.
/// This variant has one of the other options checked.
pub(crate) fn radio_group_with_null_known<T, F, U>(
    items: Vec<T>,
    on_change: F,
    known: U,
) -> LinearLayout
where
    T: 'static + std::fmt::Display,
    U: 'static + std::fmt::Display,
    F: 'static + Fn(&mut cursive::Cursive, &Option<T>),

/// Create a list of radio buttons.
/// This variant has one of the other options checked.
pub(crate) fn radio_group_with_known<T, F, I, U>(items: I, on_change: F, known: U) -> LinearLayout
where
    T: 'static + std::fmt::Display,
    U: 'static + std::fmt::Display,
    F: 'static + Fn(&mut cursive::Cursive, &T),
    I: IntoIterator<Item = T>,

/// Create a dialog window with a `Quit` button.
pub(crate) fn dialog<
    V: cursive::view::IntoBoxedView + 'static,
    F: 'static + Fn(&mut cursive::Cursive),
    Q: 'static + Fn(&mut cursive::Cursive),
>(
    name: &'static str,
    prompt: &'static str,
    child_view: V,
    exit_cb: Option<F>,
    quit_cb: Option<Q>,
) -> LinearLayout

/// The main dialog component that will be reused for multiple layers.
pub(crate) fn main_dialog<
    V: cursive::view::IntoBoxedView + 'static,
    F: 'static + Fn(&mut cursive::Cursive),
    Q: 'static + Fn(&mut cursive::Cursive),
>(
    child_view: V,
    exit_cb: Option<F>,
    quit_cb: Option<Q>,
) -> LinearLayout
```

- The `select_menu` function creates a selection menu over the provided generic options.
- The `radio_group_with_null` function creates a list of radio buttons with the `None` option. This variant has the `None` option checked.
- The `radio_group_with_null_known` function creates a list of radio buttons with the `None` option This variant has one of the other options checked.
- The `radio_group_with_known` function creates a list of radio buttons without the `None` option. This variant has one of the other options checked.
- The `dialog` function creates a dialog window with a `Quit` button.
- The `main_dialog` function creates the main dialog component that will be reused for multiple layers.

### The `capsule` submodule

The `capsule` submodule contains the configuration menus and logic for each Tock capsule. Each capsule has it own submodule where it implements their configuration process and must have a public function that can be used from `state.rs`. At the moment, there is a lot of boilerplate code in this submodule. In the future, we want to refactor this part to use generics or macros.
