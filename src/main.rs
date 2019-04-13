use std::io::Result;

mod cli;

mod dgpu;
mod perf;
mod latch;


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
    let dgpu_power = dgpu::Device::open()?.get_power()?;
    let perf_mode  = perf::Device::open()?.get_mode()?;

    println!("System Status:");
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
    let state = value_t_or_exit!(m, "state", dgpu::PowerState);

    dgpu::Device::open()?.set_power(state)
}

fn cmd_dgpu_get(_: &clap::ArgMatches) -> Result<()> {
    println!("{}", dgpu::Device::open()?.get_power()?);
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
    let state = value_t_or_exit!(m, "mode", perf::Mode);

    perf::Device::open()?.set_mode(state)
}

fn cmd_perf_get(_: &clap::ArgMatches) -> Result<()> {
    println!("{}", perf::Device::open()?.get_mode()?);
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
    latch::Device::open()?.latch_lock()
}

fn cmd_latch_unlock(_: &clap::ArgMatches) -> Result<()> {
    latch::Device::open()?.latch_unlock()
}

fn cmd_latch_request(_: &clap::ArgMatches) -> Result<()> {
    latch::Device::open()?.latch_request()
}

fn cmd_latch_get_opmode(_: &clap::ArgMatches) -> Result<()> {
    println!("{}", latch::Device::open()?.get_opmode()?);
    Ok(())
}
