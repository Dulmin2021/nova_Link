Name:           nova-link
Version:        0.1.0
Release:        1%{?dist}
Summary:        Secure local-first Android and Linux connectivity platform

License:        MIT
URL:            https://github.com/Dulmin2021/nova_Link
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust-toolchain
BuildRequires:  cargo
BuildRequires:  pkgconfig(openssl)
BuildRequires:  pkgconfig(gtk4)
BuildRequires:  pkgconfig(libadwaita-1)
BuildRequires:  pkgconfig(avahi-client)
BuildRequires:  systemd-rpm-macros

Requires:       avahi
Requires:       openssl

%description
NOVA-Link allows an Android smartphone and Linux computer to communicate
seamlessly over a local network with end-to-end encryption, streaming file
transfers, clipboard synchronization, and instant link sharing.

%prep
%autosetup

%build
cd linux
cargo build --release --workspace

%install
install -D -m 0755 linux/target/release/nova-daemon %{buildroot}%{_bindir}/nova-daemon
install -D -m 0755 linux/target/release/nova-desktop %{buildroot}%{_bindir}/nova-desktop
install -D -m 0644 linux/packaging/nova-link.service %{buildroot}%{_userunitdir}/nova-link.service
install -D -m 0644 linux/packaging/com.novalink.NovaLink.desktop %{buildroot}%{_datadir}/applications/com.novalink.NovaLink.desktop

%files
%license LICENSE
%doc README.md ARCHITECTURE.md SECURITY.md
%{_bindir}/nova-daemon
%{_bindir}/nova-desktop
%{_userunitdir}/nova-link.service
%{_datadir}/applications/com.novalink.NovaLink.desktop

%changelog
* Tue Aug 18 2026 NOVA-Link Team <dev@novalink.org> - 0.1.0-1
- Initial release of NOVA-Link for Fedora
