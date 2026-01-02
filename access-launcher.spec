Name:           access-launcher
Version:        0.2.1
Release:        1%{?dist}
Summary:        GTK4 app launcher that groups installed apps by category

License:        GPL-3.0-or-later
URL:            https://github.com/boo15mario/access-launcher
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  pkgconfig(gtk4)
BuildRequires:  desktop-file-utils

Requires:       gtk4

%description
Access Launcher is a Rust + GTK4 desktop app that lists installed
applications by category and launches them from a two-pane interface.

%prep
%autosetup -n %{name}-%{version}

%build
CARGO_NET_OFFLINE=1 cargo build --release --locked

%install
install -Dm0755 target/release/access-launcher \
    %{buildroot}%{_bindir}/access-launcher
desktop-file-install --dir=%{buildroot}%{_datadir}/applications \
    access-launcher.desktop

%check
CARGO_NET_OFFLINE=1 cargo test --release --locked
desktop-file-validate access-launcher.desktop

%files
%license LICENSE
%doc README.md
%{_bindir}/access-launcher
%{_datadir}/applications/access-launcher.desktop
