# VEX Override

This is the codebase for VURC team ORDGR's 26-27 Override season.

## Development environment setup

A nix flake is provided with a `devShell` output providing all the necessary
tools plus some extra. If you have nix installed, you can use that. If you
don't, all the necessary tools are listed below:

1. `rustup`
    - Follow the instructions at https://www.rust-lang.org/tools/install
    - This project requires nightly and has a `rust-toolchain.toml` defining the specific toolchain version.
2. `cargo-v5`
    - Used for building and uploading to the brain
    - Can be installed with cargo: `cargo install cargo-v5`
3. `picotool`
    - Used for uploading code to the Pico microcontroller over BOOTSEL mode
    - If using a probe tool, this isn't strictly necessary
    - Can be downloaded from the official Pico SDK at `https://github.com/raspberrypi/pico-sdk-tools/releases`
    - Example: `picotool load -u -v -x -t elf <executable>`
4. `probe-rs`
    - Used for uploading code and debugging the Pico microcontroller with a probe tool
    - If no probe tool is available, use BOOTSEL mode and `picotool` instead
    - Can be installed with cargo: `cargo install probe-rs`

## Project layout

| Folder                                     | Description                                                                                                                   |
| ------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------- |
| [`pico/`](./pico/)                         | The code running on the Raspberry Pi Pico 2 microcontroller we use on our bots                                                |
| [`pico/src/lidar`](./pico/src/lidar)       | Rust code implementing the prococol for RPLIDAR S & C series devices, may be factored out into a seperate crate at some point |
| [`pcb/`](./pcb/)                           | The kicad files (schematics, layouts, etc) for the custom PCB connected to our Picos and v5 brains                            |
| [`brain/bots`](./brain/bots)               | Top-level code running on our bots' v5 brains, setting up subsystems and declaring autonomous routes                          |
| [`brain/common`](./brain/common)           | Bot-agnostic v5 brain code such as shared subsystems or display logic                                                         |
| [`brain/coprocessor`](./brain/coprocessor) | Brain-side logic for communicating with the Pico microcontroller                                                              |
