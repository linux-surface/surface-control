# Linux Surface Control

Control various aspects of Microsoft Surface devices on Linux from the Command-Line.
Aims to provide a unified front-end to the various sysfs-attributes and special devices.

## Building from Source

Building this application from source follows the standard rust procedure, i.e. simply call `cargo build --release --locked` for a release-ready executable.
Completion files are automatically generated and can be found in the corresponding `target/release/build/surface-<hash>/out/` directory.

### Arch Linux

You can generate a package using the provided PKGBUILD in the `pkg/arch` directory.

### Debian

Generating a Debian package can be done via [`cargo deb`](https://github.com/mmstick/cargo-deb).
Specifically, you need to run
```
env CARGO_TARGET_DIR=target CARGO_INCREMENTAL=0 cargo deb
```
where setting `CARGO_TARGET_DIR` is required to output the generated auto-completion files at the correct location for cargo-deb to pick up.

The final package can be found in `target/debian`.
