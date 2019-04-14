mod error;
mod cli;
mod sys;

use crate::error::Result;


fn main() {
    let matches = cli::app().get_matches();

    let result = match matches.subcommand() {
        ("status",      Some(m)) => cmd_status(m),
        ("dgpu",        Some(m)) => cmd_dgpu(m),
        ("performance", Some(m)) => cmd_perf(m),
        ("latch",       Some(m)) => cmd_latch(m),
        _                        => unreachable!(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}.", e.kind());

        for cause in e.iter_causes() {
            eprintln!("       {}.", cause);
        }
    }
}


fn cmd_status(_: &clap::ArgMatches) -> Result<()> {
    let opmode     = sys::latch::Device::open()?.get_opmode()?;
    let perf_mode  = sys::perf::Device::open()?.get_mode()?;
    let dgpu_power = sys::dgpu::Device::open()?.get_power()?;

    println!("       Device-Mode: {}", opmode);
    println!("  Performance-Mode: {} ({})", perf_mode, perf_mode.short_str());
    println!("        dGPU-Power: {}", dgpu_power);

    Ok(())
}


fn cmd_dgpu(m: &clap::ArgMatches) -> Result<()> {
    match m.subcommand() {
        ("set", Some(m)) => cmd_dgpu_set(m),
        ("get", Some(m)) => cmd_dgpu_get(m),
        _                => unreachable!(),
    }
}

fn cmd_dgpu_set(m: &clap::ArgMatches) -> Result<()> {
    use clap::value_t_or_exit;
    let state = value_t_or_exit!(m, "state", sys::dgpu::PowerState);

    sys::dgpu::Device::open()?.set_power(state)?;

    if !m.is_present("quiet") {
        println!("dGPU power set to '{}'", state);
    }

    Ok(())
}

fn cmd_dgpu_get(m: &clap::ArgMatches) -> Result<()> {
    let state = sys::dgpu::Device::open()?.get_power()?;

    if !m.is_present("quiet") {
        println!("dGPU power is '{}'", state);
    } else {
        println!("{}", state);
    }

    Ok(())
}


fn cmd_perf(m: &clap::ArgMatches) -> Result<()> {
    match m.subcommand() {
        ("set", Some(m)) => cmd_perf_set(m),
        ("get", Some(m)) => cmd_perf_get(m),
        _                => unreachable!(),
    }
}

fn cmd_perf_set(m: &clap::ArgMatches) -> Result<()> {
    use clap::value_t_or_exit;
    let mode = value_t_or_exit!(m, "mode", sys::perf::Mode);

    sys::perf::Device::open()?.set_mode(mode)?;

    if !m.is_present("quiet") {
        println!("Performance-mode set to '{}'", mode);
    }

    Ok(())
}

fn cmd_perf_get(m: &clap::ArgMatches) -> Result<()> {
    let mode = sys::perf::Device::open()?.get_mode()?;

    if !m.is_present("quiet") {
        println!("Performance-mode is '{}' ({})", mode, mode.short_str());
    } else {
        println!("{}", mode.short_str());
    }

    Ok(())
}


fn cmd_latch(m: &clap::ArgMatches) -> Result<()> {
    match m.subcommand() {
        ("lock",       Some(m)) => cmd_latch_lock(m),
        ("unlock",     Some(m)) => cmd_latch_unlock(m),
        ("request",    Some(m)) => cmd_latch_request(m),
        ("get-opmode", Some(m)) => cmd_latch_get_opmode(m),
        _                       => unreachable!(),
    }
}

fn cmd_latch_lock(m: &clap::ArgMatches) -> Result<()> {
    sys::latch::Device::open()?.latch_lock()?;

    if !m.is_present("quiet") {
        println!("Clipboard latch locked");
    }

    Ok(())
}

fn cmd_latch_unlock(m: &clap::ArgMatches) -> Result<()> {
    sys::latch::Device::open()?.latch_unlock()?;

    if !m.is_present("quiet") {
        println!("Clipboard latch unlocked");
    }

    Ok(())
}

fn cmd_latch_request(m: &clap::ArgMatches) -> Result<()> {
    sys::latch::Device::open()?.latch_request()?;

    if !m.is_present("quiet") {
        println!("Clipboard latch request executed");
    }

    Ok(())
}

fn cmd_latch_get_opmode(m: &clap::ArgMatches) -> Result<()> {
    let opmode = sys::latch::Device::open()?.get_opmode()?;

    if !m.is_present("quiet") {
        println!("Device is in '{}' mode", opmode);
    } else {
        println!("{}", opmode);
    }

    Ok(())
}
