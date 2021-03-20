use crate::cli::Command as DynCommand;
use crate::sys;

use anyhow::{Context, Result};


pub struct Command;

impl DynCommand for Command {
    fn name(&self) -> &'static str {
        "dtx"
    }

    fn build(&self) -> clap::App<'static, 'static> {
        use clap::{AppSettings, SubCommand};

        SubCommand::with_name(self.name())
            .about("Control the latch/dtx-system on the Surface Book 2")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(SubCommand::with_name("lock")
                .about("Lock the latch")
                .display_order(1))
            .subcommand(SubCommand::with_name("unlock")
                .about("Unlock the latch")
                .display_order(2))
            .subcommand(SubCommand::with_name("request")
                .about("Request latch-open or abort if already in progress")
                .display_order(3))
            .subcommand(SubCommand::with_name("confirm")
                .about("Confirm latch-open if detachment in progress")
                .display_order(4))
            .subcommand(SubCommand::with_name("hearbeat")
                .about("Send hearbeat if detachment in progress")
                .display_order(5))
            .subcommand(SubCommand::with_name("cancel")
                .about("Cancel any detachment in progress")
                .display_order(6))
            .subcommand(SubCommand::with_name("get-base")
                .about("Get information about the currently attached base")
                .display_order(7))
            .subcommand(SubCommand::with_name("get-devicemode")
                .about("Query the current device operation mode")
                .display_order(8))
            .subcommand(SubCommand::with_name("get-latchstatus")
                .about("Query the current latch status")
                .display_order(9))
            .subcommand(SubCommand::with_name("monitor")
                .about("Monitor DTX events")
                .display_order(10))
    }

    fn execute(&self, m: &clap::ArgMatches) -> Result<()> {
        match m.subcommand() {
            ("lock",            Some(m)) => self.lock(m),
            ("unlock",          Some(m)) => self.unlock(m),
            ("request",         Some(m)) => self.request(m),
            ("confirm",         Some(m)) => self.confirm(m),
            ("heartbeat",       Some(m)) => self.heartbeat(m),
            ("cancel",          Some(m)) => self.cancel(m),
            ("get-base",        Some(m)) => self.get_base_info(m),
            ("get-devicemode",  Some(m)) => self.get_device_mode(m),
            ("get-latchstatus", Some(m)) => self.get_latch_status(m),
            ("monitor",         Some(m)) => self.monitor(m),
            _                            => unreachable!(),
        }
    }
}

impl Command {
    fn lock(&self, m: &clap::ArgMatches) -> Result<()> {
        sys::dtx::Device::open()
            .context("Failed to open DTX device")?
            .latch_lock()
            .context("Failed to lock latch")?;

        if !m.is_present("quiet") {
            println!("Clipboard latch locked");
        }

        Ok(())
    }

    fn unlock(&self, m: &clap::ArgMatches) -> Result<()> {
        sys::dtx::Device::open()
            .context("Failed to open DTX device")?
            .latch_unlock()
            .context("Failed to unlock latch")?;

        if !m.is_present("quiet") {
            println!("Clipboard latch unlocked");
        }

        Ok(())
    }

    fn request(&self, m: &clap::ArgMatches) -> Result<()> {
        sys::dtx::Device::open()
            .context("Failed to open DTX device")?
            .latch_request()
            .context("Failed to send latch request")?;

        if !m.is_present("quiet") {
            println!("Clipboard latch request executed");
        }

        Ok(())
    }

    fn confirm(&self, m: &clap::ArgMatches) -> Result<()> {
        sys::dtx::Device::open()
            .context("Failed to open DTX device")?
            .latch_confirm()
            .context("Failed to send confirmation")?;

        if !m.is_present("quiet") {
            println!("Clipboard detachment confirmed");
        }

        Ok(())
    }

    fn heartbeat(&self, m: &clap::ArgMatches) -> Result<()> {
        sys::dtx::Device::open()
            .context("Failed to open DTX device")?
            .latch_heartbeat()
            .context("Failed to send heartbeat")?;

        if !m.is_present("quiet") {
            println!("Clipboard detachment heartbeat sent");
        }

        Ok(())
    }

    fn cancel(&self, m: &clap::ArgMatches) -> Result<()> {
        sys::dtx::Device::open()
            .context("Failed to open DTX device")?
            .latch_cancel()
            .context("Failed to cancel detachment")?;

        if !m.is_present("quiet") {
            println!("Clipboard detachment canceled");
        }

        Ok(())
    }

    fn get_base_info(&self, m: &clap::ArgMatches) -> Result<()> {
        let info = sys::dtx::Device::open()
            .context("Failed to open DTX device")?
            .get_base_info()
            .context("Failed to get base info")?;

        if !m.is_present("quiet") {
            println!("State:       {}", info.state);
            println!("Device-Type: {}", info.device_type);
            println!("ID:          {:#04x}", info.id);

        } else if let sys::dtx::DeviceType::Unknown(ty) = info.device_type {
            println!("{{ \"state\": \"{}\", \"type\": {}, \"id\": {} }}",
                     info.state, ty, info.id);

        } else {
            println!("{{ \"state\": \"{}\", \"type\": \"{}\", \"id\": {} }}",
                     info.state, info.device_type, info.id);
        }

        Ok(())
    }

    fn get_device_mode(&self, m: &clap::ArgMatches) -> Result<()> {
        let mode = sys::dtx::Device::open()
            .context("Failed to open DTX device")?
            .get_device_mode()
            .context("Failed to get device mode")?;

        if !m.is_present("quiet") {
            println!("Device is in '{}' mode", mode);
        } else {
            println!("{}", mode);
        }

        Ok(())
    }

    fn get_latch_status(&self, m: &clap::ArgMatches) -> Result<()> {
        let status = sys::dtx::Device::open()
            .context("Failed to open DTX device")?
            .get_latch_status()
            .context("Failed to get latch status")?;

        if !m.is_present("quiet") {
            println!("Latch has been '{}'", status);
        } else {
            println!("{}", status);
        }

        Ok(())
    }

    fn monitor(&self, _m: &clap::ArgMatches) -> Result<()> {
        let mut device = sys::dtx::Device::open()
            .context("Failed to open DTX device")?;

        let events = device.events()
            .context("Failed to set up event stream")?;

        for event in events {
            let event = event
                .map_err(|source| sys::Error::IoError { source })
                .context("Error reading event")?;

            println!("{:?}", event);
        }

        Ok(())
    }
}
