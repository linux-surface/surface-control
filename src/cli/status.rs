use crate::cli::Command as DynCommand;
use crate::sys;

use anyhow::Result;


pub struct Command;

impl DynCommand for Command {
    fn name(&self) -> &'static str {
        "status"
    }

    fn build(&self) -> clap::Command {
        clap::Command::new(self.name())
            .about("Show an overview of the current system status")
    }

    fn execute(&self, _: &clap::ArgMatches) -> Result<()> {
        let stats = Stats::load();

        if stats.available() {
            print!("{stats}");
            Ok(())
        } else {
            anyhow::bail!("No devices found")
        }
    }
}


struct Stats {
    prof: ProfileStats,
    dgpu: DgpuStats,
    dtx:  DtxStats,
}

struct ProfileStats {
    current: String,
    supported: Vec<String>,
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
        let prof = ProfileStats::load();
        let dgpu = DgpuStats::load();
        let dtx  = DtxStats::load();

        Stats { prof, dgpu, dtx }
    }

    fn available(&self) -> bool {
        self.prof.available()
            || self.dgpu.available()
            || self.dtx.available()
    }
}

impl ProfileStats {
    fn load() -> Self {
        let dev = sys::profile::Device::open().ok();

        let current = dev.as_ref().and_then(|d| d.get().ok());
        let supported = dev.as_ref().and_then(|d| d.get_supported().ok());

        ProfileStats {
            current: current.unwrap_or_default(),
            supported: supported.unwrap_or_default(),
        }
    }

    fn available(&self) -> bool {
        !self.supported.is_empty() && !self.current.is_empty()
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
        write!(f, "{}{}{}", self.prof, self.dgpu, self.dtx)
    }
}

impl std::fmt::Display for ProfileStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.available() {
            return Ok(());
        }

        let mut profiles = String::new();
        for (i, profile) in self.supported.iter().enumerate() {
            if i > 0 {
                profiles += " ";
            }

            if *profile == self.current {
                profiles += "[";
                profiles += profile;
                profiles += "]";
            } else {
                profiles += profile;
            }
        }

        writeln!(f, "Platform Profile: {profiles}\n")
    }
}

impl std::fmt::Display for DgpuStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.available() {
            return Ok(())
        }

        writeln!(f, "Discrete GPU:")?;

        if let Some(vendor) = self.vendor {
            writeln!(f, "  Vendor:         {vendor:04x}")?;
        }
        if let Some(device) = self.device {
            writeln!(f, "  Device:         {device:04x}")?;
        }
        if let Some(power_state) = self.power_state {
            writeln!(f, "  Power State:    {power_state}")?;
        }
        if let Some(runtime_pm) = self.runtime_pm {
            writeln!(f, "  Runtime PM:     {runtime_pm}")?;
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
            writeln!(f, "  Device Mode:    {device_mode}")?;
        }
        if let Some(base_state) = self.base_state {
            writeln!(f, "  Base State:     {base_state}")?;
        }
        if let Some(base_type) = self.base_type {
            writeln!(f, "  Base Type:      {base_type}")?;
        }
        if let Some(base_id) = self.base_id {
            writeln!(f, "  Base ID:        {base_id:#04x}")?;
        }
        if let Some(latch_status) = self.latch_status {
            writeln!(f, "  Latch Status:   {latch_status}")?;
        }

        writeln!(f)
    }
}
