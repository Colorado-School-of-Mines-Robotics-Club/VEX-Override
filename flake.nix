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
                        # Vex brain development
                        cargo-v5
                        # Display development
                        SDL2 # Required by embedded-graphics-simulator
                        imagemagick
                        # Pico development
                        probe-rs-tools # Interfacing with pico probe
                        picotool       # Flashing over BOOTSEL
                        pioasm         # Official PIO compiler
                        tio            # Allows reading serial devices (like USB)
                        # PCB development
                        kicad
                    ]);
                };
            }
        );
}
