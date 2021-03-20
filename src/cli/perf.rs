use crate::cli::Command as DynCommand;
use crate::sys;

use anyhow::{Context, Result};


pub struct Command;

impl DynCommand for Command {
    fn name(&self) -> &'static str {
        "performance"
    }

    fn build(&self) -> clap::App<'static, 'static> {
        use clap::{AppSettings, Arg, SubCommand};

        SubCommand::with_name(self.name())
            .about("Control or query the current performance-mode")
            .long_about(indoc::indoc!("
                Control or query the current performance-mode

                Supported performance-mode values are:

                    Value  Name
                    ---------------------------
                        1  Normal (Default)
                        2  Battery Saver
                        3  Better Performance
                        4  Best Performance
                "))
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(SubCommand::with_name("set")
                .about("Set the current performance-mode")
                .arg(Arg::with_name("mode")
                    .possible_values(&["1", "2", "3", "4"])
                    .required(true)
                    .index(1)))
            .subcommand(SubCommand::with_name("get")
                .about("Get the current performance-mode"))
    }

    fn execute(&self, m: &clap::ArgMatches) -> Result<()> {
        match m.subcommand() {
            ("set", Some(m)) => self.perf_set(m),
            ("get", Some(m)) => self.perf_get(m),
            _                => unreachable!(),
        }
    }
}

impl Command {
    fn perf_set(&self, m: &clap::ArgMatches) -> Result<()> {
        use clap::value_t_or_exit;
        let mode = value_t_or_exit!(m, "mode", sys::perf::Mode);

        let dev = sys::perf::Device::open()
            .context("Failed to open performance mode device")?;

        let current_mode = dev.get_mode()
            .context("Failed to get current performance mode")?;

        if mode != current_mode {
            dev.set_mode(mode)
                .context("Failed to set performance mode")?;

            if !m.is_present("quiet") {
                println!("Performance-mode set to '{}'", mode);
            }

        } else if !m.is_present("quiet") {
            println!("Performance-mode already set to '{}', not changing", mode);
        }

        Ok(())
    }

    fn perf_get(&self, m: &clap::ArgMatches) -> Result<()> {
        let mode = sys::perf::Device::open()
            .context("Failed to open performance mode device")?
            .get_mode()
            .context("Failed to get current performance mode")?;

        if !m.is_present("quiet") {
            println!("Performance-mode is '{}' ({})", mode, mode.short_str());
        } else {
            println!("{}", mode.short_str());
        }

        Ok(())
    }
}
