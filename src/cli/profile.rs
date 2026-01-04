use crate::cli::Command as DynCommand;
use crate::sys;

use anyhow::{Context, Result};


pub struct Command;

impl DynCommand for Command {
    fn name(&self) -> &'static str {
        "profile"
    }

    fn build(&self) -> clap::Command {
        use clap::Arg;

        clap::Command::new(self.name())
            .about("Control or query the current platform profile")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .infer_subcommands(true)
            .subcommand(clap::Command::new("set")
                .about("Set the current platform profile")
                .arg(Arg::new("profile")
                    .required(true)
                    .index(1)))
            .subcommand(clap::Command::new("get")
                .about("Get the current platform profile"))
            .subcommand(clap::Command::new("list")
                .about("List all available platform profiles"))
            .subcommand(clap::Command::new("next")
                .about("Cycle to the next platform profile"))
            .subcommand(clap::Command::new("prev")
                .about("Cycle to the previous platform profile"))
    }

    fn execute(&self, m: &clap::ArgMatches) -> Result<()> {
        match m.subcommand() {
            Some(("set",  m)) => self.profile_set(m),
            Some(("get",  m)) => self.profile_get(m),
            Some(("list", m)) => self.profile_list(m),
            Some(("next", m)) => self.profile_cycle_next(m),
            Some(("prev", m)) => self.profile_cycle_prev(m),
            _                 => unreachable!(),
        }
    }
}

impl Command {
    fn profile_set(&self, m: &clap::ArgMatches) -> Result<()> {
        let profile: &String = m.get_one("profile").unwrap();

        let dev = sys::profile::Device::open()
            .context("Failed to open platform profile device")?;

        let supported = dev.get_supported()
            .context("Failed to get supported platform profiles")?;

        if !supported.iter().any(|p| p == profile) {
            anyhow::bail!("Platform profile '{profile}' is not supported");
        }

        let current_profile = dev.get()
            .context("Failed to get current platform profile")?;

        if profile != &current_profile {
            dev.set(profile)
                .context("Failed to set platform profile")?;

            if !m.get_flag("quiet") {
                println!("Platform profile set to '{profile}'");
            }

        } else if !m.get_flag("quiet") {
            println!("Platform profile already set to '{profile}', not changing");
        }

        Ok(())
    }

    fn profile_get(&self, _m: &clap::ArgMatches) -> Result<()> {
        let dev = sys::profile::Device::open()
            .context("Failed to open platform profile device")?;

        let profile = dev.get()
            .context("Failed to get current platform profile")?;

        println!("{profile}");
        Ok(())
    }

    fn profile_list(&self, m: &clap::ArgMatches) -> Result<()> {
        let dev = sys::profile::Device::open()
            .context("Failed to open platform profile device")?;

        let supported = dev.get_supported()
            .context("Failed to get supported platform profiles")?;

        if !m.get_flag("quiet") {
            for profile in supported {
                println!("{profile}");
            }

        } else {
            let text = serde_json::to_string(&supported)
                .context("Failed to serialize data")?;

            println!("{text}");
        }

        Ok(())
    }

    fn profile_cycle_next(&self, m: &clap::ArgMatches) -> Result<()> {
        let dev = sys::profile::Device::open()
            .context("Failed to open platform profile device")?;

        let supported = dev.get_supported()
            .context("Failed to get supported platform profiles")?;

        if supported.is_empty() {
            anyhow::bail!("No platform profiles available");
        }

        let current_profile = dev.get()
            .context("Failed to get current platform profile")?;

        // Find the next profile in the list, wrapping around to the start
        let current_index = supported.iter()
            .position(|p| p == &current_profile)
            .unwrap_or(0);

        let next_index = (current_index + 1) % supported.len();
        let next_profile = &supported[next_index];

        dev.set(next_profile)
            .context("Failed to set platform profile")?;

        if !m.get_flag("quiet") {
            println!("Platform profile cycled from '{current_profile}' to '{next_profile}'");
        }

        Ok(())
    }

    fn profile_cycle_prev(&self, m: &clap::ArgMatches) -> Result<()> {
        let dev = sys::profile::Device::open()
            .context("Failed to open platform profile device")?;

        let supported = dev.get_supported()
            .context("Failed to get supported platform profiles")?;

        if supported.is_empty() {
            anyhow::bail!("No platform profiles available");
        }

        let current_profile = dev.get()
            .context("Failed to get current platform profile")?;

        // Find the previous profile in the list, wrapping around to the end
        let current_index = supported.iter()
            .position(|p| p == &current_profile)
            .unwrap_or(0);

        let prev_index = (current_index + supported.len() - 1) % supported.len();
        let prev_profile = &supported[prev_index];

        dev.set(prev_profile)
            .context("Failed to set platform profile")?;

        if !m.get_flag("quiet") {
            println!("Platform profile cycled from '{current_profile}' to '{prev_profile}'");
        }

        Ok(())
    }
}
