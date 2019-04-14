use std::io::Result;

mod cli;
mod sys;


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
        eprintln!("Error: {}", e);
    }
}


fn cmd_status(_: &clap::ArgMatches) -> Result<()> {
    let opmode     = sys::latch::Device::open()?.get_opmode()?;
    let perf_mode  = sys::perf::Device::open()?.get_mode()?;
    let dgpu_power = sys::dgpu::Device::open()?.get_power()?;

    println!("System Status:");
    println!("  Device-Mode:      {}", opmode);
    println!("  Performance-Mode: {}", perf_mode);
    println!("  dGPU-Power:       {}", dgpu_power);

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

    sys::dgpu::Device::open()?.set_power(state)
}

fn cmd_dgpu_get(_: &clap::ArgMatches) -> Result<()> {
    println!("{}", sys::dgpu::Device::open()?.get_power()?);
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
    let state = value_t_or_exit!(m, "mode", sys::perf::Mode);

    sys::perf::Device::open()?.set_mode(state)
}

fn cmd_perf_get(_: &clap::ArgMatches) -> Result<()> {
    println!("{}", sys::perf::Device::open()?.get_mode()?);
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

fn cmd_latch_lock(_: &clap::ArgMatches) -> Result<()> {
    sys::latch::Device::open()?.latch_lock()
}

fn cmd_latch_unlock(_: &clap::ArgMatches) -> Result<()> {
    sys::latch::Device::open()?.latch_unlock()
}

fn cmd_latch_request(_: &clap::ArgMatches) -> Result<()> {
    sys::latch::Device::open()?.latch_request()
}

fn cmd_latch_get_opmode(_: &clap::ArgMatches) -> Result<()> {
    println!("{}", sys::latch::Device::open()?.get_opmode()?);
    Ok(())
}
