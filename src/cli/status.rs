use crate::cli::Command as DynCommand;
use crate::sys;

use anyhow::Result;


pub struct Command;

impl DynCommand for Command {
    fn name(&self) -> &'static str {
        "status"
    }

    fn build(&self) -> clap::App<'static, 'static> {
        clap::SubCommand::with_name(self.name())
            .about("Show an overview of the current system status")
    }

    fn execute(&self, _: &clap::ArgMatches) -> Result<()> {
        let stats = Stats::load();

        if stats.available() {
            print!("{}", stats);
            Ok(())
        } else {
            anyhow::bail!("No devices found")
        }
    }
}


struct Stats {
    perf: PerfStats,
    dgpu: DgpuStats,
    dtx:  DtxStats,
}

struct PerfStats {
    mode: Option<sys::perf::Mode>,
}

struct DgpuStats {
    vendor:      Option<u16>,
    device:      Option<u16>,
    power_state: Option<sys::pci::PowerState>,
    runtime_pm:  Option<sys::pci::RuntimePowerManagement>,
}

struct DtxStats {
    device_mode:  Option<sdtx::DeviceMode>,
    base_state:   Option<sdtx::BaseState>,
    base_type:    Option<sdtx::DeviceType>,
    base_id:      Option<u8>,
    latch_status: Option<sdtx::LatchStatus>,
}


impl Stats {
    fn load() -> Self {
        let perf = PerfStats::load();
        let dgpu = DgpuStats::load();
        let dtx  = DtxStats::load();

        Stats { perf, dgpu, dtx }
    }

    fn available(&self) -> bool {
        self.perf.available()
            || self.dgpu.available()
            || self.dtx.available()
    }
}

impl PerfStats {
    fn load() -> Self {
        let mode = sys::perf::Device::open().ok()
            .and_then(|device| device.get_mode().ok());

        PerfStats { mode }
    }

    fn available(&self) -> bool {
        self.mode.is_some()
    }
}

impl DgpuStats {
    fn load() -> Self {
        let dev = crate::cli::dgpu::find_dgpu_device().ok().and_then(|x| x);

        let vendor = dev.as_ref().and_then(|d| d.vendor_id().ok());
        let device = dev.as_ref().and_then(|d| d.device_id().ok());
        let power_state = dev.as_ref().and_then(|d| d.get_power_state().ok());
        let runtime_pm = dev.as_ref().and_then(|d| d.get_runtime_pm().ok());

        DgpuStats { vendor, device, power_state, runtime_pm }
    }

    fn available(&self) -> bool {
        self.vendor.is_some()
            || self.device.is_some()
            || self.power_state.is_some()
            || self.runtime_pm.is_some()
    }
}

impl DtxStats {
    fn load() -> Self {
        let dev = sdtx::Device::open().ok();

        let base = dev.as_ref().and_then(|d| d.get_base_info().ok());
        let base_state = base.map(|b| b.state);
        let base_type = base.map(|b| b.device_type);
        let base_id = base.map(|b| b.id);

        let device_mode = dev.as_ref().and_then(|d| d.get_device_mode().ok());
        let latch_status = dev.as_ref().and_then(|d| d.get_latch_status().ok());

        DtxStats {
            device_mode,
            base_state,
            base_type,
            base_id,
            latch_status,
        }
    }

    fn available(&self) -> bool {
        self.device_mode.is_some()
            || self.base_state.is_some()
            || self.base_type.is_some()
            || self.base_id.is_some()
            || self.latch_status.is_some()
    }
}


impl std::fmt::Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.perf, self.dgpu, self.dtx)
    }
}

impl std::fmt::Display for PerfStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(mode) = self.mode {
            writeln!(f, "Performance Mode: {}\n", mode)
        } else {
            Ok(())
        }
    }
}

impl std::fmt::Display for DgpuStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.available() {
            return Ok(())
        }

        writeln!(f, "Discrete GPU:")?;

        if let Some(vendor) = self.vendor {
            writeln!(f, "  Vendor:         {:04x}", vendor)?;
        }
        if let Some(device) = self.device {
            writeln!(f, "  Device:         {:04x}", device)?;
        }
        if let Some(power_state) = self.power_state {
            writeln!(f, "  Power State:    {}", power_state)?;
        }
        if let Some(runtime_pm) = self.runtime_pm {
            writeln!(f, "  Runtime PM:     {}", runtime_pm)?;
        }

        writeln!(f)
    }
}

impl std::fmt::Display for DtxStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.available() {
            return Ok(())
        }

        writeln!(f, "DTX:")?;

        if let Some(device_mode) = self.device_mode {
            writeln!(f, "  Device Mode:    {}", device_mode)?;
        }
        if let Some(base_state) = self.base_state {
            writeln!(f, "  Base State:     {}", base_state)?;
        }
        if let Some(base_type) = self.base_type {
            writeln!(f, "  Base Type:      {}", base_type)?;
        }
        if let Some(base_id) = self.base_id {
            writeln!(f, "  Base ID:        {:#04x}", base_id)?;
        }
        if let Some(latch_status) = self.latch_status {
            writeln!(f, "  Latch Status:   {}", latch_status)?;
        }

        writeln!(f)
    }
}
