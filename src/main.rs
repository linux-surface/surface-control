use std::io::Result;
use clap;

mod dgpu;
mod perf;


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
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("set")
            .about("Set the current performance-mode")
            .arg(Arg::with_name("mode")
                .possible_values(&["1", "2", "3", "4"])
                .required(true)
                .index(1)))
        .subcommand(SubCommand::with_name("get")
            .about("Get the current performance-mode"));

    let dgpu_power = SubCommand::with_name("power")
        .about("Control or query the dGPU power state")
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

    let dgpu = SubCommand::with_name("dgpu")
        .about("Control the dGPU")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(dgpu_power);

    App::new("surfacectl")
        .version(clap::crate_version!())
        .author("Maximilian Luz <luzmaximilian@gmail.com>")
        .about("Control various aspects of Microsoft Surface devices")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .global_settings(&settings)
        .subcommand(status)
        .subcommand(perf)
        .subcommand(dgpu)
}

fn main() {
    let matches = app().get_matches();

    let result = match matches.subcommand() {
        ("status",      Some(m)) => cmd_status(m),
        ("dgpu",        Some(m)) => cmd_dgpu(m),
        ("performance", Some(m)) => cmd_perf(m),
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
        ("power", Some(m)) => cmd_dgpu_power(m),
        _                  => unreachable!(),
    }
}

fn cmd_dgpu_power(m: &clap::ArgMatches) -> Result<()> {
    match m.subcommand() {
        ("set", Some(m)) => cmd_dgpu_power_set(m),
        ("get", Some(m)) => cmd_dgpu_power_get(m),
        _                => unreachable!(),
    }
}

fn cmd_dgpu_power_set(m: &clap::ArgMatches) -> Result<()> {
    use clap::value_t_or_exit;
    let state = value_t_or_exit!(m, "state", dgpu::PowerState);

    dgpu::Device::open()?.set_power(state)
}

fn cmd_dgpu_power_get(_: &clap::ArgMatches) -> Result<()> {
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
