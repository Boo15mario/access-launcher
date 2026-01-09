# Access Launcher

Access Launcher is a Rust + GTK4 desktop app that lists installed applications by category
and launches them from a two-pane interface.

## Features
- Scans system and user `.desktop` files.
- Groups apps into common categories (Internet, Office, Utilities, etc.).
- Keyboard-friendly list navigation with accessible labels.

## Requirements
- Rust toolchain (edition 2021).
- GTK4 development libraries available on your system.

## Install Dependencies
The commands below install the Rust toolchain, a C compiler, pkg-config, and GTK4
development headers needed to build this app.
```bash
# Fedora
sudo dnf install -y rust cargo gtk4-devel gcc pkgconf-pkg-config

# Debian/Ubuntu
sudo apt-get update
sudo apt-get install -y rustc cargo libgtk-4-dev build-essential pkg-config

# Arch Linux
sudo pacman -S --needed rust cargo gtk4 base-devel pkgconf

# Gentoo
sudo emerge --ask dev-lang/rust gui-libs/gtk:4 sys-devel/gcc pkgconf

# NixOS (temporary shell)
nix-shell -p rustc cargo pkg-config gtk4 gcc
# Or:
nix shell nixpkgs#rustc nixpkgs#cargo nixpkgs#pkg-config nixpkgs#gtk4 nixpkgs#gcc
```
If you do not have Rust installed, see `https://rustup.rs` for a standard setup.

NixOS runtime note: Access Launcher scans `XDG_DATA_HOME`, `XDG_DATA_DIRS`, and common
Nix profile paths for `.desktop` files. If your apps do not appear, ensure your
profile `share/applications` paths are present in `XDG_DATA_DIRS` (or `NIX_PROFILES`).

### Optional Tools
- rustfmt and clippy (recommended):
  - rustup: `rustup component add rustfmt clippy`
  - Distro packages (verify names with your package manager):
    - Fedora: `sudo dnf install -y rustfmt clippy`
    - Debian/Ubuntu: `sudo apt-get install -y rustfmt clippy`
    - Arch Linux: `sudo pacman -S --needed rustfmt clippy`
    - Gentoo: `sudo emerge --ask dev-util/rustfmt dev-util/clippy`
- Auto-rebuild on changes: `cargo install cargo-watch`
- Debugging (distro packages):
  - Fedora: `sudo dnf install -y gdb lldb`
  - Debian/Ubuntu: `sudo apt-get install -y gdb lldb`
  - Arch Linux: `sudo pacman -S --needed gdb lldb`
  - Gentoo: `sudo emerge --ask sys-devel/gdb sys-devel/lldb`

## Build and Run
```bash
cargo build
cargo run
```

## Install (NixOS)
Add a local build to `environment.systemPackages` or `home.packages`:
```nix
# configuration.nix or home.nix
{
  environment.systemPackages = [
    (pkgs.rustPlatform.buildRustPackage {
      pname = "access-launcher";
      version = "git";
      src = /path/to/access-launcher;
      cargoLock.lockFile = /path/to/access-launcher/Cargo.lock;
      nativeBuildInputs = [ pkgs.pkg-config ];
      buildInputs = [ pkgs.gtk4 ];
    })
  ];
}
```
Home Manager variant:
```nix
# home.nix
{
  home.packages = [
    (pkgs.rustPlatform.buildRustPackage {
      pname = "access-launcher";
      version = "git";
      src = /path/to/access-launcher;
      cargoLock.lockFile = /path/to/access-launcher/Cargo.lock;
      nativeBuildInputs = [ pkgs.pkg-config ];
      buildInputs = [ pkgs.gtk4 ];
    })
  ];
}
```
Flake example:
```nix
{
  description = "Access Launcher";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs";

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
    in {
      packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
        pname = "access-launcher";
        version = "git";
        src = self;
        cargoLock.lockFile = ./Cargo.lock;
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = [ pkgs.gtk4 ];
      };
    };
}
```
Usage:
```bash
nix build
./result/bin/access-launcher
```
Or:
```bash
nix run
```

## Usage
- Select a category in the left pane to filter applications.
- Activate an app in the right pane to launch it.
- Print the version: `access-launcher -v`
- Show help: `access-launcher -h`
- Run without flags to start the application normally.

## Development Notes
- Entry point: `src/main.rs`.
- Shared modules: `src/lib.rs`, `src/desktop.rs`, `src/ui.rs`.
- Integration tests: `tests/desktop.rs`.
- Formatting: `cargo fmt`
- Linting: `cargo clippy`
- Tests: `cargo test`
