# Maddi-Type: TUI Typing Practice

This is a project that I threw together in a day to help
myself learn some alternate keyboard layouts.

## Installation

`maddi-type` can be found on [crates.io](https://crates.io/)
and so can be installed with `cargo install maddi-type`.

Note that `maddi-type` uses **Rust 2024** so requires a
relatively modern Rust compiler to build.

Additionally, a `flake.nix` file is present in the root of
the file if you'd like to use `maddi-type` in your NixOS or
`home-manager` configurations.

## Usage

`maddi-type <FILE>.txt` is enough to get you started. This
will create a `<FILE>.progress.json` file that will track
your progress if you want to leave and return later.

Additional options can be listed with `maddi-type help` but
are currently limited to specifying the progress file
position.

## Stability

In its current form, the application will crash at very low
terminal sizes. This will be resolved in the future. Note
that currently a crash will result in your progress not
being saved.

## License

`maddi-type` is licensed under the GPLv3. As I am the sole
contributor, I'd be happy to relicense the code
permissively if you'd like to use it in your own
open-source project. If this is the case, please email me
to discuss.

`maddi-type` is compliant with version 3.2 of the [REUSE
specification](https://reuse.software/spec-3.2/).
