use crate::cli::Command as DynCommand;
use crate::error::Result;
use crate::sys;


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
            .subcommand(SubCommand::with_name("get-opmode")
                .about("Query the current device operation mode")
                .display_order(4))
    }

    fn execute(&self, m: &clap::ArgMatches) -> Result<()> {
        match m.subcommand() {
            ("lock",       Some(m)) => self.lock(m),
            ("unlock",     Some(m)) => self.unlock(m),
            ("request",    Some(m)) => self.request(m),
            ("get-opmode", Some(m)) => self.get_opmode(m),
            _                       => unreachable!(),
        }
    }
}

impl Command {
    fn lock(&self, m: &clap::ArgMatches) -> Result<()> {
        sys::dtx::Device::open()?.latch_lock()?;

        if !m.is_present("quiet") {
            println!("Clipboard latch locked");
        }

        Ok(())
    }

    fn unlock(&self, m: &clap::ArgMatches) -> Result<()> {
        sys::dtx::Device::open()?.latch_unlock()?;

        if !m.is_present("quiet") {
            println!("Clipboard latch unlocked");
        }

        Ok(())
    }

    fn request(&self, m: &clap::ArgMatches) -> Result<()> {
        sys::dtx::Device::open()?.latch_request()?;

        if !m.is_present("quiet") {
            println!("Clipboard latch request executed");
        }

        Ok(())
    }

    fn get_opmode(&self, m: &clap::ArgMatches) -> Result<()> {
        let opmode = sys::dtx::Device::open()?.get_opmode()?;

        if !m.is_present("quiet") {
            println!("Device is in '{}' mode", opmode);
        } else {
            println!("{}", opmode);
        }

        Ok(())
    }
}
