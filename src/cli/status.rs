use failure::ResultExt;

use crate::cli::Command as DynCommand;
use crate::error::{ErrorKind, Result};
use crate::sys;


pub struct Command;

impl DynCommand for Command {
    fn name(&self) -> &'static str {
        "status"
    }

    fn build(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Query the current system status")
    }

    fn execute(&self, _: &clap::ArgMatches) -> Result<()> {
        let mut found = false;

        let opmode = sys::dtx::Device::open().and_then(|d| d.get_opmode());
        let opmode = match opmode {
            Ok(x) => { found = true; Some(x) },
            Err(ref e) if e.kind() == ErrorKind::DeviceAccess => None,
            Err(e) => return Err(e),
        };

        let perf_mode = sys::perf::Device::open().and_then(|d| d.get_mode());
        let perf_mode = match perf_mode {
            Ok(x) => { found = true; Some(x) },
            Err(ref e) if e.kind() == ErrorKind::DeviceAccess => None,
            Err(e) => return Err(e),
        };

        if found {
            if let Some(opmode) = opmode {
                println!("       Device-Mode: {}", opmode);
            }
            if let Some(perf_mode) = perf_mode {
                println!("  Performance-Mode: {} ({})", perf_mode, perf_mode.short_str());
            }

            Ok(())

        } else {
            Err(failure::err_msg("No surface control device not found"))
                .context(ErrorKind::DeviceAccess)
                .map_err(Into::into)
        }
    }
}
