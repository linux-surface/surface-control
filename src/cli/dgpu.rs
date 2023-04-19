use std::convert::TryFrom;

use crate::cli::Command as DynCommand;
use crate::sys::pci::PciDevice;
use crate::sys::Error;
use crate::sys;

use anyhow::{Context, Result};
use clap::ValueEnum;
use clap::builder::PossibleValue;


pub struct Command;

impl DynCommand for Command {
    fn name(&self) -> &'static str {
        "dgpu"
    }

    fn build(&self) -> clap::Command {
        use clap::Arg;

        clap::Command::new(self.name())
            .about("Control the discrete GPU")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .infer_subcommands(true)
            .subcommand(clap::Command::new("id")
                .alias("get-id")
                .about("Get the dGPU PCI device ID")
                .display_order(1))
            .subcommand(clap::Command::new("get-power-state")
                .aliases(["ps", "get-ps", "power-state"])
                .about("Get the dGPU PCI power state")
                .display_order(2))
            .subcommand(clap::Command::new("get-runtime-pm")
                .aliases(["rpm", "get-rpm"])
                .about("Get the dGPU runtime PM control")
                .display_order(3))
            .subcommand(clap::Command::new("set-runtime-pm")
                .alias("set-rpm")
                .about("Set the dGPU runtime PM control")
                .arg(Arg::new("mode")
                    .value_parser(clap::value_parser!(sys::pci::RuntimePowerManagement))
                    .required(true)
                    .index(1))
                .display_order(4))
    }

    fn execute(&self, m: &clap::ArgMatches) -> Result<()> {
        match m.subcommand() {
            Some(("id",              m)) => self.get_id(m),
            Some(("get-power-state", m)) => self.get_power_state(m),
            Some(("get-runtime-pm",  m)) => self.get_runtime_pm(m),
            Some(("set-runtime-pm",  m)) => self.set_runtime_pm(m),
            _                            => unreachable!(),
        }
    }
}

impl Command {
    fn get_id(&self, m: &clap::ArgMatches) -> Result<()> {
        let dgpu = find_dgpu_device()
            .context("Failed to look up discrete GPU device")?
            .ok_or_else(|| anyhow::anyhow!("No discrete GPU found"))?;

        let vendor_id = dgpu.vendor_id()
            .context("Failed to get vendor ID")?;

        let device_id = dgpu.device_id()
            .context("Failed to get device ID")?;

        if !m.get_flag("quiet") {
            println!("Vendor: {vendor_id:04x}");
            println!("Device: {device_id:04x}");
        } else {
            println!("{vendor_id:04x}:{device_id:04x}");
        }

        Ok(())
    }

    fn get_power_state(&self, _m: &clap::ArgMatches) -> Result<()> {
        let dgpu = find_dgpu_device()
            .context("Failed to look up discrete GPU device")?
            .ok_or_else(|| anyhow::anyhow!("No discrete GPU found"))?;

        let pstate = dgpu.get_power_state()
            .context("Failed to get device power state")?;

         println!("{pstate}");

        Ok(())
    }

    fn get_runtime_pm(&self, _m: &clap::ArgMatches) -> Result<()> {
        let dgpu = find_dgpu_device()
            .context("Failed to look up discrete GPU device")?
            .ok_or_else(|| anyhow::anyhow!("No discrete GPU found"))?;

        let mode = dgpu.get_runtime_pm()
            .context("Failed to get runtime PM mode")?;

        println!("{mode}");

        Ok(())
    }

    fn set_runtime_pm(&self, m: &clap::ArgMatches) -> Result<()> {
        let mode: sys::pci::RuntimePowerManagement = *m.get_one("mode").unwrap();

        let mut dgpu = find_dgpu_device()
            .context("Failed to look up discrete GPU device")?
            .ok_or_else(|| anyhow::anyhow!("No discrete GPU found"))?;

        dgpu.set_runtime_pm(mode)
            .context("Failed to set runtime PM mode")?;

        if !m.get_flag("quiet") {
            println!("Discrete GPU runtime PM set to '{mode}'");
        }

        Ok(())
    }
}

pub fn find_dgpu_device() -> crate::sys::Result<Option<PciDevice>> {
    let mut enumerator = udev::Enumerator::new()
        .map_err(|source| Error::Io { source })?;

    enumerator.match_subsystem("pci")
        .map_err(|source| Error::Io { source })?;

    let devices = enumerator.scan_devices()
        .map_err(|source| Error::Io { source })?;

    for device in devices {
        let device = PciDevice::try_from(device)
            .map_err(|source| Error::SysFs { source })?;

        let vendor_id = device.vendor_id()
            .map_err(|source| Error::SysFs { source })?;

        if vendor_id != sys::pci::VENDOR_ID_NVIDIA {
            continue;
        }

        let class = device.class()
            .map_err(|source| Error::SysFs { source })?;

        if class.base != sys::pci::BASE_CLASS_DISPLAY {
            continue;
        }

        return Ok(Some(device));
    }

    Ok(None)
}

impl ValueEnum for sys::pci::RuntimePowerManagement {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::On, Self::Off]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        match self {
            Self::On => Some(PossibleValue::new("on").help("Enable runtime power management")),
            Self::Off => Some(PossibleValue::new("off").help("Disable runtime power management")),
        }
    }
}
