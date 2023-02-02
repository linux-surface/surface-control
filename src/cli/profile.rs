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
    }

    fn execute(&self, m: &clap::ArgMatches) -> Result<()> {
        match m.subcommand() {
            Some(("set",  m))  => self.profile_set(m),
            Some(("get",  m))  => self.profile_get(m),
            Some(("list", m)) => self.profile_list(m),
            _                => unreachable!(),
        }
    }
}

impl Command {
    fn profile_set(&self, m: &clap::ArgMatches) -> Result<()> {
        let profile = m.value_of("profile").unwrap();

        let dev = sys::profile::Device::open()
            .context("Failed to open platform profile device")?;

        let supported = dev.get_supported()
            .context("Failed to get supported platform profiles")?;

        if !supported.iter().any(|p| p == profile) {
            anyhow::bail!("Platform profile '{profile}' is not supported");
        }

        let current_profile = dev.get()
            .context("Failed to get current platform profile")?;

        if profile != current_profile {
            dev.set(profile)
                .context("Failed to set platform profile")?;

            if !m.is_present("quiet") {
                println!("Platform profile set to '{profile}'");
            }

        } else if !m.is_present("quiet") {
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

        if !m.is_present("quiet") {
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
}
