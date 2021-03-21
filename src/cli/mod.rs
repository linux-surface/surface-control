pub mod dgpu;
pub mod dtx;
pub mod perf;
pub mod status;

use anyhow::Result;

use std::collections::HashMap;


pub trait Command {
    fn name(&self) -> &'static str;
    fn build(&self) -> clap::App<'static, 'static>;
    fn execute(&self, matches: &clap::ArgMatches) -> Result<()>;
}


pub struct Registry {
    commands: HashMap<&'static str, Box<dyn Command>>,
}

impl Registry {
    pub fn build() -> Self {
        let list: Vec<Box<dyn Command>> = vec![
            Box::new(status::Command),
            Box::new(perf::Command),
            Box::new(dtx::Command),
            Box::new(dgpu::Command),
        ];

        Registry {
            commands: list.into_iter().map(|c| (c.name(), c)).collect(),
        }
    }

    pub fn cli(&self) -> clap::App<'static, 'static> {
        use clap::{App, AppSettings, Arg};

        let settings = [
            AppSettings::ColoredHelp,
            AppSettings::InferSubcommands,
            AppSettings::VersionlessSubcommands,
        ];

        let mut app = App::new(clap::crate_name!())
            .version(clap::crate_version!())
            .author(clap::crate_authors!())
            .about("Control various aspects of Microsoft Surface devices")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .global_settings(&settings)
            .arg(Arg::with_name("quiet")
                .help("Keep output quiet")
                .short("q")
                .long("quiet")
                .global(true));

        for cmd in self.commands.values() {
            app = app.subcommand(cmd.build());
        }

        app
    }

    pub fn execute(&self, command: &str, matches: &clap::ArgMatches) -> Result<()> {
        self.commands[command].execute(matches)
    }
}

pub fn build() -> Registry {
    Registry::build()
}
