# Fedora / RHEL-family package, built from source. Needs Fedora 43+
# (GTK 4.18+, libadwaita 1.8+). Build with packaging/fedora/build-rpm.sh.
Name:           channex
Version:        0.2.0
Release:        1%{?dist}
Summary:        Native GTK/libadwaita viewer for chan-style imageboards
License:        LicenseRef-unknown
URL:            https://github.com/niruxx/Channex
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust
BuildRequires:  gcc
BuildRequires:  cmake
BuildRequires:  pkgconfig(gtk4) >= 4.18
BuildRequires:  pkgconfig(libadwaita-1) >= 1.8
BuildRequires:  blueprint-compiler
BuildRequires:  desktop-file-utils
BuildRequires:  libappstream-glib

Requires:       gtk4 >= 4.18
Requires:       libadwaita >= 1.8
Recommends:     gstreamer1-plugins-good
Recommends:     gstreamer1-plugin-libav

%global appid io.github.niruxx.Channex
# Rust binaries don't produce useful debuginfo packages here.
%global debug_package %{nil}

%description
Browse 4chan, 8kun, Lainchan and other 4chan-API-compatible or LynxChan
imageboards in a native GNOME-style app: catalogs, threads with quote
previews, bookmarks, downloads, a media viewer, and classic skins.

%prep
%autosetup

%build
cargo build --release --locked

%install
install -Dm755 target/release/channex %{buildroot}%{_bindir}/channex
install -Dm644 data/%{appid}.desktop %{buildroot}%{_datadir}/applications/%{appid}.desktop
install -Dm644 data/%{appid}.metainfo.xml %{buildroot}%{_metainfodir}/%{appid}.metainfo.xml
install -Dm644 data/icons/hicolor/scalable/apps/%{appid}.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/%{appid}.svg

%check
desktop-file-validate %{buildroot}%{_datadir}/applications/%{appid}.desktop
cargo test --release --locked

%files
%{_bindir}/channex
%{_datadir}/applications/%{appid}.desktop
%{_metainfodir}/%{appid}.metainfo.xml
%{_datadir}/icons/hicolor/scalable/apps/%{appid}.svg

%changelog
* Wed Nov 04 2026 niruxxdaboi <perezkyle6@gmail.com> - 0.2.0-1
- GTK 4 + libadwaita rewrite in Rust
