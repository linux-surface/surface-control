Name:       surface-control
Version:    0.4.9
Release:    1%{?dist}
Summary:    Control various aspects of Microsoft Surface devices from the shell

License:    MIT
URL:        https://github.com/linux-surface/surface-control

Requires:       dbus libgcc systemd-libs
BuildRequires:  rust cargo systemd-rpm-macros systemd-devel

%global debug_package %{nil}

%description
Control various aspects of Microsoft Surface devices on Linux from the shell.
Aims to provide a unified front-end to the various sysfs-attributes and special
devices.

%prep

%build
export CARGO_TARGET_DIR="$PWD/target"
export CARGO_INCREMENTAL=0

cargo build --release --locked
strip --strip-all "target/release/surface"

%install
rm -rf %{buildroot}
install -D -m755 "target/release/surface" "%{buildroot}/usr/bin/surface"
install -D -m644 "target/surface.bash" "%{buildroot}/usr/share/bash-completion/completions/surface"
install -D -m644 "target/_surface" "%{buildroot}/usr/share/zsh/site-functions/_surface"
install -D -m644 "target/surface.fish" "%{buildroot}/usr/share/fish/vendor_completions.d/surface.fish"

%files
/usr/bin/surface
/usr/share/bash-completion/completions/surface
/usr/share/zsh/site-functions/_surface
/usr/share/fish/vendor_completions.d/surface.fish

%changelog
* Fri Oct 10 2025 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.9-1
- Update dependencies

* Sat Apr 19 2025 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.8-1
- Update dependencies

* Sat Sep 14 2024 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.7-1
- Update dependencies

* Thu Mar 14 2024 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.6-3
- Update dependencies

* Tue Oct 03 2023 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.6-2
- Bump release for Fedora 39

* Tue Oct 03 2023 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.6-1
- Update dependencies

* Mon Jun 12 2023 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.5-1
- Remove outdated udev rules
- Update dependencies

* Wed Apr 19 2023 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.4-1
- Add support for Fedora 38
- Update dependencies

* Fri Oct 14 2022 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.3-2
- Bump release to build for Fedora 36

* Sat Sep 17 2022 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.3-1
- Remove outdated performance-mode command
- Update dependencies

* Thu Apr 28 2022 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.2-1
- Update dependencies

* Wed Apr 27 2022 Dorian Stoll <dorian.stoll@tmsp.io> - 0.4.1-3
- Bump release to build for Fedora 36

* Wed Nov 03 2021 Dorian Stoll <dorian.stoll@tmsp.io> - 0.4.1-2
- Bump release to build for Fedora 35

* Wed May 06 2021 Maximilian Luz <luzmaximilian@gmail.com> - 0.4.1-1
- Add interface for platform profile

* Mon Mar 22 2021 Dorian Stoll <dorian.stoll@tmsp.io> - 0.4.0-2
- Fix libudev dependency

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
