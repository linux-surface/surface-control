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
            .subcommand(SubCommand::with_name("heartbeat")
                .about("Send heartbeat if detachment in progress")
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
        sdtx::Device::open()
            .context("Failed to open DTX device")?
            .latch_lock()
            .context("Failed to lock latch")?;

        if !m.is_present("quiet") {
            println!("Clipboard latch locked");
        }

        Ok(())
    }

    fn unlock(&self, m: &clap::ArgMatches) -> Result<()> {
        sdtx::Device::open()
            .context("Failed to open DTX device")?
            .latch_unlock()
            .context("Failed to unlock latch")?;

        if !m.is_present("quiet") {
            println!("Clipboard latch unlocked");
        }

        Ok(())
    }

    fn request(&self, m: &clap::ArgMatches) -> Result<()> {
        sdtx::Device::open()
            .context("Failed to open DTX device")?
            .latch_request()
            .context("Failed to send latch request")?;

        if !m.is_present("quiet") {
            println!("Clipboard latch request executed");
        }

        Ok(())
    }

    fn confirm(&self, m: &clap::ArgMatches) -> Result<()> {
        sdtx::Device::open()
            .context("Failed to open DTX device")?
            .latch_confirm()
            .context("Failed to send confirmation")?;

        if !m.is_present("quiet") {
            println!("Clipboard detachment confirmed");
        }

        Ok(())
    }

    fn heartbeat(&self, m: &clap::ArgMatches) -> Result<()> {
        sdtx::Device::open()
            .context("Failed to open DTX device")?
            .latch_heartbeat()
            .context("Failed to send heartbeat")?;

        if !m.is_present("quiet") {
            println!("Clipboard detachment heartbeat sent");
        }

        Ok(())
    }

    fn cancel(&self, m: &clap::ArgMatches) -> Result<()> {
        sdtx::Device::open()
            .context("Failed to open DTX device")?
            .latch_cancel()
            .context("Failed to cancel detachment")?;

        if !m.is_present("quiet") {
            println!("Clipboard detachment canceled");
        }

        Ok(())
    }

    fn get_base_info(&self, m: &clap::ArgMatches) -> Result<()> {
        let info = sdtx::Device::open()
            .context("Failed to open DTX device")?
            .get_base_info()
            .context("Failed to get base info")?;

        if !m.is_present("quiet") {
            println!("State: {}", info.state);
            println!("Type:  {}", info.device_type);
            println!("ID:    {:#04x}", info.id);

        } else {
            let text = serde_json::to_string(&PrettyBaseInfo(info))
                .context("Failed to serialize data")?;

            println!("{}", text);
        }

        Ok(())
    }

    fn get_device_mode(&self, _m: &clap::ArgMatches) -> Result<()> {
        let mode = sdtx::Device::open()
            .context("Failed to open DTX device")?
            .get_device_mode()
            .context("Failed to get device mode")?;

        println!("{}", mode);
        Ok(())
    }

    fn get_latch_status(&self, _m: &clap::ArgMatches) -> Result<()> {
        let status = sdtx::Device::open()
            .context("Failed to open DTX device")?
            .get_latch_status()
            .context("Failed to get latch status")?;

        println!("{}", status);
        Ok(())
    }

    fn monitor(&self, m: &clap::ArgMatches) -> Result<()> {
        let mut device = sdtx::Device::open()
            .context("Failed to open DTX device")?;

        let events = device.events()
            .context("Failed to set up event stream")?;

        let quiet = m.is_present("quiet");

        for event in events {
            let event = event
                .map_err(|source| sys::Error::Io { source })
                .context("Error reading event")?;

            if !quiet {
                println!("{}", PrettyEvent(event));

            } else {
                let text = serde_json::to_string(&PrettyEvent(event))
                    .context("Failed to serialize data")?;

                println!("{}", text);
            }
        }

        Ok(())
    }
}


struct PrettyBaseInfo(sdtx::BaseInfo);

impl serde::Serialize for PrettyBaseInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        use serde::ser::SerializeStruct;

        let mut s = serializer.serialize_struct("BaseInfo", 3)?;

        match self.0.state {
            sdtx::BaseState::Attached    => s.serialize_field("state", "attached"),
            sdtx::BaseState::Detached    => s.serialize_field("state", "detached"),
            sdtx::BaseState::NotFeasible => s.serialize_field("state", "not-feasible"),
        }?;

        match self.0.device_type {
            sdtx::DeviceType::Hid        => s.serialize_field("type", "hid"),
            sdtx::DeviceType::Ssh        => s.serialize_field("type", "ssh"),
            sdtx::DeviceType::Unknown(x) => s.serialize_field("type", &x),
        }?;

        s.serialize_field("id", &self.0.id)?;
        s.end()
    }
}


struct PrettyEvent(sdtx::Event);

impl serde::Serialize for PrettyEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        use serde::ser::SerializeStruct;
        use sdtx::{DeviceType, Event, HardwareError, RuntimeError, uapi};
        use sdtx::event::{BaseState, CancelReason, DeviceMode, LatchStatus};

        match &self.0 {
            Event::Request => {
                let mut s = serializer.serialize_struct("Event", 1)?;
                s.serialize_field("type", "request")?;
                s.end()
            },

            Event::Cancel { reason } => {
                let mut s = serializer.serialize_struct("Event", 2)?;
                s.serialize_field("type", "cancel")?;

                match reason {
                    CancelReason::Hardware(err) => match err {
                        HardwareError::FailedToOpen       => s.serialize_field("reason", "failed-to-open"),
                        HardwareError::FailedToRemainOpen => s.serialize_field("reason", "failed-to-remain-open"),
                        HardwareError::FailedToClose      => s.serialize_field("reason", "failed-to-close"),
                        HardwareError::Unknown(x) => {
                            s.serialize_field("reason", &(*x as u16 | uapi::SDTX_CATEGORY_HARDWARE_ERROR))
                        },
                    },
                    CancelReason::Runtime(err) => match err {
                        RuntimeError::NotFeasible => s.serialize_field("reason", "not-feasible"),
                        RuntimeError::Timeout     => s.serialize_field("reason", "timeout"),
                        RuntimeError::Unknown(x) => {
                            s.serialize_field("reason", &(*x as u16 | uapi::SDTX_CATEGORY_RUNTIME_ERROR))
                        },
                    },
                    CancelReason::Unknown(x) => s.serialize_field("reason", x),
                }?;

                s.end()
            },

            Event::BaseConnection { state, device_type, id } => {
                let mut s = serializer.serialize_struct("Event", 4)?;
                s.serialize_field("type", "base-connection")?;

                match state {
                    BaseState::Attached    => s.serialize_field("state", "attached"),
                    BaseState::Detached    => s.serialize_field("state", "detached"),
                    BaseState::NotFeasible => s.serialize_field("state", "not-feasible"),
                    BaseState::Unknown(x)  => s.serialize_field("state", x),
                }?;

                match device_type {
                    DeviceType::Hid        => s.serialize_field("device-type", "hid"),
                    DeviceType::Ssh        => s.serialize_field("device-type", "ssh"),
                    DeviceType::Unknown(x) => s.serialize_field("device-type", &x),
                }?;

                s.serialize_field("id", id)?;
                s.end()
            },

            Event::LatchStatus { status } => {
                let mut s = serializer.serialize_struct("Event", 2)?;
                s.serialize_field("type", "latch-status")?;

                match status {
                    LatchStatus::Closed     => s.serialize_field("status", "closed"),
                    LatchStatus::Opened     => s.serialize_field("status", "opened"),
                    LatchStatus::Error(err) => match err {
                        HardwareError::FailedToOpen       => s.serialize_field("status", "failed-to-open"),
                        HardwareError::FailedToRemainOpen => s.serialize_field("status", "failed-to-remain-open"),
                        HardwareError::FailedToClose      => s.serialize_field("status", "failed-to-close"),
                        HardwareError::Unknown(x) => {
                            s.serialize_field("status", &(*x as u16 | uapi::SDTX_CATEGORY_HARDWARE_ERROR))
                        },
                    },
                    LatchStatus::Unknown(x) => s.serialize_field("status", x),
                }?;

                s.end()
            },

            Event::DeviceMode { mode } => {
                let mut s = serializer.serialize_struct("Event", 2)?;
                s.serialize_field("type", "device-mode")?;

                match mode {
                    DeviceMode::Tablet     => s.serialize_field("mode", "tablet"),
                    DeviceMode::Laptop     => s.serialize_field("mode", "laptop"),
                    DeviceMode::Studio     => s.serialize_field("mode", "studio"),
                    DeviceMode::Unknown(x) => s.serialize_field("mode", x),
                }?;

                s.end()
            },

            Event::Unknown { code, data } => {
                let mut s = serializer.serialize_struct("Event", 2)?;
                s.serialize_field("type", code)?;
                s.serialize_field("data", data)?;
                s.end()
            },
        }
    }
}

impl std::fmt::Display for PrettyEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use sdtx::{DeviceType, Event};
        use sdtx::event::{BaseState, CancelReason, DeviceMode, LatchStatus};

        match &self.0 {
            Event::Request => {
                write!(f, "Request")
            },

            Event::Cancel { reason } => {
                write!(f, "Cancel         {{ Reason: ")?;

                match reason {
                    CancelReason::Hardware(err) => write!(f, "\"{}\"", err),
                    CancelReason::Runtime(err)  => write!(f, "\"{}\"", err),
                    CancelReason::Unknown(err)  => write!(f, "{:#04x}", err),
                }?;

                write!(f, " }}")
            },

            Event::BaseConnection { state, device_type, id } => {
                write!(f, "BaseConnection {{ State: ")?;

                match state {
                    BaseState::Detached    => write!(f, "Detached"),
                    BaseState::Attached    => write!(f, "Attached"),
                    BaseState::NotFeasible => write!(f, "NotFeasible"),
                    BaseState::Unknown(x)  => write!(f, "{:#04x}", x),
                }?;

                write!(f, ", DeviceType: ")?;

                match device_type {
                    DeviceType::Hid        => write!(f, "Hid"),
                    DeviceType::Ssh        => write!(f, "Ssh"),
                    DeviceType::Unknown(x) => write!(f, "{:#04x}", x),
                }?;

                write!(f, ", Id: {:#04x} }}", id)
            },

            Event::LatchStatus { status } => {
                write!(f, "LatchStatus    {{ Status: ")?;

                match status {
                    LatchStatus::Closed     => write!(f, "Closed"),
                    LatchStatus::Opened     => write!(f, "Opened"),
                    LatchStatus::Error(err) => write!(f, "\"Error: {}\"", err),
                    LatchStatus::Unknown(x) => write!(f, "{:#04x}", x),
                }?;

                write!(f, " }}")
            },

            Event::DeviceMode { mode } => {
                write!(f, "DeviceMode     {{ Status: ")?;

                match mode {
                    DeviceMode::Tablet     => write!(f, "Tablet"),
                    DeviceMode::Laptop     => write!(f, "Laptop"),
                    DeviceMode::Studio     => write!(f, "Studio"),
                    DeviceMode::Unknown(x) => write!(f, "{:#04x}", x),
                }?;

                write!(f, " }}")
            },

            Event::Unknown { code, data } => {
                write!(f, "Unknown        {{ Code: {:#04x}, Data: {:02x?} }}", code, data)
            },
        }
    }
}
