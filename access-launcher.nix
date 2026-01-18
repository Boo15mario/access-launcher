{ pkgs ? import <nixpkgs> {} }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "access-launcher";
  version = "0.1.0";

  # IMPORTANT: Replace these with your actual GitHub repository details
  src = pkgs.fetchFromGitHub {
    owner = "your-github-owner";
    repo = "access-launcher";
    rev = "main"; # Or a specific commit hash, e.g., "abcdef1234567890abcdef1234567890abcdef12"
    sha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="; # You'll need to update this after the first build (nix will tell you the correct hash)
  };

  # cargoLock is not needed when fetching from a git repo with Cargo.lock present
  # The lock file will be part of the fetched source.

  nativeBuildInputs = with pkgs; [
    pkg-config
    glib
    gtk4
  ];

  buildInputs = with pkgs; [
    glib
    gtk4
  ];

  buildPhase = ''
    cargo build --release
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp target/release/access-launcher $out/bin
    mkdir -p $out/share/applications
    cp access-launcher.desktop $out/share/applications
    mkdir -p $out/share/icons/hicolor/scalable/apps
    cp access-launcher.svg $out/share/icons/hicolor/scalable/apps
  '';

  meta = with pkgs.lib; {
    description = "A simple application launcher";
    homepage = "https://github.com/your-github-owner/access-launcher"; # Update this
    license = licenses.mit; # Or your specific license
    maintainers = [ ]; # Add your GitHub username here, e.g., pkgs.lib.maintainers.your-github-username
    platforms = platforms.linux;
  };
}
