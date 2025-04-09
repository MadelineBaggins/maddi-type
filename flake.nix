# SPDX-FileCopyrightText: 2025 Madeline Baggins <declanbaggins@gmail.com>
#
# SPDX-License-Identifier: CC0-1.0

{
  description = "A TUI program to practice typing";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    oxalica.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    oxalica,
    flake-utils,
    ...
  }: flake-utils.lib.eachDefaultSystem (system:
    let
      overlays = [ (import oxalica) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      latest-stable = pkgs.rust-bin.stable.latest.default.override {
        extensions = [
          "rust-src"
          "rust-analyzer"
        ];
      };
      stable-platform = with pkgs; makeRustPlatform {
        rustc = latest-stable;
        cargo = latest-stable;
      };
    in with pkgs; {
      defaultPackage = stable-platform.buildRustPackage {
        pname = "maddi-type";
        version = "0.1.0";
        src = ./.;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };
      };
      devShells.default = mkShell {
        buildInputs = [
          latest-stable
        ];
      };
    }
  );
}
