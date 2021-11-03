use std::convert::TryFrom;

use crate::{cli::Command as DynCommand, sys};
use crate::sys::pci::PciDevice;

use anyhow::{Context, Result};
use sys::Error;


pub struct Command;

impl DynCommand for Command {
    fn name(&self) -> &'static str {
        "dgpu"
    }

    fn build(&self) -> clap::App<'static, 'static> {
        use clap::{AppSettings, Arg, SubCommand};

        clap::SubCommand::with_name(self.name())
            .about("Control the discrete GPU")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .setting(AppSettings::InferSubcommands)
            .subcommand(SubCommand::with_name("id")
                .alias("get-id")
                .about("Get the dGPU PCI device ID")
                .display_order(1))
            .subcommand(SubCommand::with_name("get-power-state")
                .aliases(&["ps", "get-ps", "power-state"])
                .about("Get the dGPU PCI power state")
                .display_order(2))
            .subcommand(SubCommand::with_name("get-runtime-pm")
                .aliases(&["rpm", "get-rpm"])
                .about("Get the dGPU runtime PM control")
                .display_order(3))
            .subcommand(SubCommand::with_name("set-runtime-pm")
                .alias("set-rpm")
                .about("Set the dGPU runtime PM control")
                .arg(Arg::with_name("mode")
                    .possible_values(&["on", "off"])
                    .required(true)
                    .index(1))
                .display_order(4))
    }

    fn execute(&self, m: &clap::ArgMatches) -> Result<()> {
        match m.subcommand() {
            ("id",              Some(m)) => self.get_id(m),
            ("get-power-state", Some(m)) => self.get_power_state(m),
            ("get-runtime-pm",  Some(m)) => self.get_runtime_pm(m),
            ("set-runtime-pm",  Some(m)) => self.set_runtime_pm(m),
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

        if !m.is_present("quiet") {
            println!("Vendor: {:04x}", vendor_id);
            println!("Device: {:04x}", device_id);
        } else {
            println!("{:04x}:{:04x}", vendor_id, device_id);
        }

        Ok(())
    }

    fn get_power_state(&self, _m: &clap::ArgMatches) -> Result<()> {
        let dgpu = find_dgpu_device()
            .context("Failed to look up discrete GPU device")?
            .ok_or_else(|| anyhow::anyhow!("No discrete GPU found"))?;

        let pstate = dgpu.get_power_state()
            .context("Failed to get device power state")?;

         println!("{}", pstate);

        Ok(())
    }

    fn get_runtime_pm(&self, _m: &clap::ArgMatches) -> Result<()> {
        let dgpu = find_dgpu_device()
            .context("Failed to look up discrete GPU device")?
            .ok_or_else(|| anyhow::anyhow!("No discrete GPU found"))?;

        let mode = dgpu.get_runtime_pm()
            .context("Failed to get runtime PM mode")?;

        println!("{}", mode);

        Ok(())
    }

    fn set_runtime_pm(&self, m: &clap::ArgMatches) -> Result<()> {
        use clap::value_t_or_exit;
        let mode = value_t_or_exit!(m, "mode", sys::pci::RuntimePowerManagement);

        let mut dgpu = find_dgpu_device()
            .context("Failed to look up discrete GPU device")?
            .ok_or_else(|| anyhow::anyhow!("No discrete GPU found"))?;

        dgpu.set_runtime_pm(mode)
            .context("Failed to set runtime PM mode")?;

        if !m.is_present("quiet") {
            println!("Discrete GPU runtime PM set to '{}'", mode);
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
