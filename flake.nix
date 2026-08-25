{
    inputs = {
        nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
        flake-utils.url = "github:numtide/flake-utils";
        rust-overlay = {
            url = "github:oxalica/rust-overlay";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };

    outputs =
        inputs@{
            self,
            nixpkgs,
            flake-utils,
            rust-overlay,
            ...
        }:
        flake-utils.lib.eachDefaultSystem (
            system:
            let
                # Import nixpkgs
                pkgs = nixpkgs.legacyPackages.${system};
                inherit (pkgs) lib;
                # Setup rust toolchain
                rust-bin = rust-overlay.lib.mkRustBin { } pkgs;
                rust' = (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml);
                # winit
                libPath = with pkgs; lib.makeLibraryPath [
                    libGL
                    libxkbcommon
                    wayland
                    vulkan-loader
                    vulkan-validation-layers
                ];
            in
            {
                # Provide a development environment with rust, cargo-v5, and the formatter
                devShells.default = pkgs.mkShell {
                    packages = [
                        rust'
                    ]
                    ++ (with pkgs; [
                        # General rust development tools
                        evcxr
                        cargo-watch
                        cargo-nextest
                        # Vex brain development
                        cargo-v5
                        # Display development
                        SDL2 # Required by embedded-graphics-simulator
                        imagemagick
                        # Simulator requirements
                        pkg-config
                        fontconfig
                        glibc
                        clang
                        llvmPackages.libclang
                        cmake
                        udev
                        # Pico development
                        probe-rs-tools # Interfacing with pico probe
                        picotool       # Flashing over BOOTSEL
                        pioasm         # Official PIO compiler
                        tio            # Allows reading serial devices (like USB)
                        mpremote
                        # PCB development
                        kicad
                    ]);

                    env = {
                        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
                        LD_LIBRARY_PATH = libPath;
                        VULKAN_SDK = "${pkgs.vulkan-headers}";
                    };
                };
            }
        );
}
