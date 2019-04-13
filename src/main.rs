use std::io::Result;
use clap;
use indoc::indoc;

mod dgpu;
mod perf;
mod latch;


fn app() -> clap::App<'static, 'static> {
    use clap::{App, AppSettings, Arg, SubCommand};

    let settings = [
        AppSettings::InferSubcommands,
        AppSettings::VersionlessSubcommands,
    ];

    let status = SubCommand::with_name("status")
        .about("Query the current system status");

    let perf = SubCommand::with_name("performance")
        .about("Control or query the current performance-mode")
        .long_about(indoc!("
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
            .about("Get the current performance-mode"));

    let dgpu = SubCommand::with_name("dgpu")
        .about("Control or query the dGPU power state")
        .long_about(indoc!("
            Control or query the dGPU power state

            Supported values are: 'on', 'off'.
            "))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("set")
            .about("Set the current dGPU power state")
            .arg(Arg::with_name("state")
                .help("The power-state to be set")
                .possible_values(&["on", "off", "1", "0"])
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("get")
            .about("Get the current dGPU power state"));

    let latch = SubCommand::with_name("latch")
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
            .display_order(4));

    App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!("\n"))
        .about("Control various aspects of Microsoft Surface devices")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .global_settings(&settings)
        .subcommand(status)
        .subcommand(perf)
        .subcommand(dgpu)
        .subcommand(latch)
}

fn main() {
    let matches = app().get_matches();

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
