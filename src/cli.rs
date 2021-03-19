use clap::{App, AppSettings, Arg, SubCommand};
use indoc::indoc;


pub fn app() -> App<'static, 'static> {
    let settings = [
        AppSettings::ColoredHelp,
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

    let dtx = SubCommand::with_name("dtx")
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
        .author("Maximilian Luz <luzmaximilian@gmail.com>")
        .about("Control various aspects of Microsoft Surface devices")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .global_settings(&settings)
        .subcommand(status)
        .subcommand(perf)
        .subcommand(dtx)
        .arg(Arg::with_name("quiet")
            .help("Keep output quiet")
            .short("q")
            .long("quiet")
            .global(true))
}
