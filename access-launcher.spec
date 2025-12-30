Name:           access-launcher
Version:        0.1.0
Release:        1%{?dist}
Summary:        GTK4 app launcher that groups installed apps by category

License:        GPL-3.0-or-later
URL:            https://example.com/access-launcher
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
%cargo_build

%install
%cargo_install
install -Dpm 0644 access-launcher.desktop \
    %{buildroot}%{_datadir}/applications/access-launcher.desktop

%check
desktop-file-validate %{buildroot}%{_datadir}/applications/access-launcher.desktop

%files
%license LICENSE
%doc README.md
%{_bindir}/access-launcher
%{_datadir}/applications/access-launcher.desktop
