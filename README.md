# Linux Surface Control

![CI](https://github.com/linux-surface/surface-control/workflows/CI/badge.svg)

Control various aspects of Microsoft Surface devices on Linux from the Command-Line.
Aims to provide a unified front-end to the various sysfs-attributes and special devices.

## Usage

```
USAGE:
    surface [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Keep output quiet
    -V, --version    Prints version information

SUBCOMMANDS:
    dgpu           Control the discrete GPU
    dtx            Control the latch/dtx-system on the Surface Book 2
    help           Prints this message or the help of the given subcommand(s)
    performance    Control or query the current performance-mode
    status         Show an overview of the current system status
```

See `surface <subcommand> help` for more details.

_Hint:_ You can specify the subcommand by any unabiguous prefix of it, i.e. `surface perf` and `surface p` will both evaluate to `surface performance`.

## Prequisites

For this tool to work, you need a recent version of the [surface-sam module][surface-sam] e.g. via the [linux-surface kernel][surface-kernel].

## Installing

Have a look at the [releases](https://github.com/linux-surface/surface-control/releases) page.
Pre-built packages are available for Debian (Ubuntu, ...), whereas PKGBUILDs for Arch Linux are in the AUR (`surface-control`).

_Hint_: Add the following udev rule to change performance mode as a normal user
```
KERNEL=="01:03:01:00:01", SUBSYSTEM=="surface_aggregator", RUN+="/usr/bin/chmod 666 /sys/bus/surface_aggregator/devices/01:03:01:00:01/perf_mode"
```

## Building from Source

Building this application from source follows the standard rust procedure, i.e. simply call `cargo build --release --locked` for a release-ready executable.
Completion files are automatically generated and can be found in the corresponding `target/release/build/surface-<hash>/out/` directory.

### Arch Linux

Simply install `surface-control` from AUR or have a look at its PKGBUILD.

### Debian-based Distributions (Ubuntu, ...)

Generating a Debian package can be done via [`cargo deb`](https://github.com/mmstick/cargo-deb).
Specifically, you need to run
```
env CARGO_TARGET_DIR=target CARGO_INCREMENTAL=0 cargo deb
```
where setting `CARGO_TARGET_DIR` is required to output the generated auto-completion files at the correct location for cargo-deb to pick up.

The final package can be found in `target/debian`.

[surface-sam]: https://github.com/linux-surface/surface-aggregator-module
[surface-kernel]: https://github.com/linux-surface/linux-surface
