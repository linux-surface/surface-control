use crate::cli::Command as DynCommand;
use crate::sys;

use anyhow::{Context, Result};


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

        let mode = sys::dtx::Device::open().and_then(|d| d.get_device_mode());
        let mode = match mode {
            Ok(x) => { found = true; Some(x) },
            Err(sys::Error::DeviceAccess { .. }) => None,
            Err(e) => return Err(e).context("Failed to access DTX device"),
        };

        let perf_mode = sys::perf::Device::open().and_then(|d| d.get_mode());
        let perf_mode = match perf_mode {
            Ok(x) => { found = true; Some(x) },
            Err(sys::Error::DeviceAccess { .. }) => None,
            Err(e) => return Err(e).context("Failed to access performance mode device"),
        };

        // TODO: print dGPU power state

        if found {
            if let Some(mode) = mode {
                println!("       Device-Mode: {}", mode);
            }
            if let Some(perf_mode) = perf_mode {
                println!("  Performance-Mode: {} ({})", perf_mode, perf_mode.short_str());
            }

            Ok(())

        } else {
            anyhow::bail!("No Surface control device found")
        }
    }
}
