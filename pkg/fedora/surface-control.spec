Name:       surface-control
Version:    0.4.0
Release:    1%{?dist}
Summary:    Control various aspects of Microsoft Surface devices from the shell

License:    MIT
URL:        https://github.com/linux-surface/surface-control

Requires:       dbus libgcc libudev
BuildRequires:  rust cargo systemd-rpm-macros libudev-devel

%global debug_package %{nil}

%description
Control various aspects of Microsoft Surface devices on Linux from the shell.
Aims to provide a unified front-end to the various sysfs-attributes and special
devices.

%prep

%build
cd surface-control

export CARGO_TARGET_DIR="$PWD/target"
export CARGO_INCREMENTAL=0

cargo build --release --locked
strip --strip-all "target/release/surface"

%install
rm -rf %{buildroot}
install -D -m755 "surface-control/target/release/surface" "%{buildroot}/usr/bin/surface"
install -D -m644 "surface-control/etc/sysusers/surface-control.conf" "%{buildroot}%{_sysusersdir}/%{name}.conf"
install -D -m644 "surface-control/etc/udev/40-surface-control.rules" "%{buildroot}%{_udevrulesdir}/40-surface-control.rules"
install -D -m644 "surface-control/target/surface.bash" "%{buildroot}/usr/share/bash-completion/completions/surface"
install -D -m644 "surface-control/target/_surface" "%{buildroot}/usr/share/zsh/site-functions/_surface"
install -D -m644 "surface-control/target/surface.fish" "%{buildroot}/usr/share/fish/completions/surface.fish"

%pre
%sysusers_create_package %{name} "surface-control/etc/sysusers/surface-control.conf"

%files
%license surface-control/LICENSE
/usr/bin/surface
%{_sysusersdir}/%{name}.conf
%{_udevrulesdir}/40-surface-control.rules
/usr/share/bash-completion/completions/surface
/usr/share/zsh/site-functions/_surface
/usr/share/fish/completions/surface.fish

%changelog
* Sun Mar 21 2021 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.0-1
- Update dGPU interface
- Add more DTX commands
- Restructure status command

* Fri Mar 19 2021 Dorian Stoll <dorian.stoll@tmsp.io> - 0.3.1-2
- Bump release to build for Fedora 34

* Tue Sep 29 2020 Dorian Stoll <dorian.stoll@tmsp.io> - 0.2.8-2
- Bump release to build for Fedora 33

* Fri Jul 03 2020 Maximilian Luz <luzmaximilian@gmail.com>
- Update to version 0.2.6

* Tue Mar 31 2020 Dorian Stoll <dorian.stoll@tmsp.io> 0.2.5-3
- Bump pkgrel

* Sun Dec 01 2019 Dorian Stoll <dorian.stoll@tmsp.io>
- Update to version 0.2.5

* Fri Sep 27 2019 Dorian Stoll <dorian.stoll@tmsp.io>
- Update packaging

* Sat Sep 14 2019 Dorian Stoll <dorian.stoll@tmsp.io>
- Update to 0.2.4

* Fri May 17 2019 Dorian Stoll <dorian.stoll@tmsp.io>
- Initial version
