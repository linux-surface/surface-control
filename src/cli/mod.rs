pub mod dgpu;
pub mod dtx;
pub mod profile;
pub mod status;

use anyhow::Result;

use std::collections::HashMap;


pub trait Command {
    fn name(&self) -> &'static str;
    fn build(&self) -> clap::Command;
    fn execute(&self, matches: &clap::ArgMatches) -> Result<()>;
}


pub struct Registry {
    commands: HashMap<&'static str, Box<dyn Command>>,
}

impl Registry {
    pub fn build() -> Self {
        let list: Vec<Box<dyn Command>> = vec![
            Box::new(status::Command),
            Box::new(profile::Command),
            Box::new(dtx::Command),
            Box::new(dgpu::Command),
        ];

        Registry {
            commands: list.into_iter().map(|c| (c.name(), c)).collect(),
        }
    }

    pub fn cli(&self) -> clap::Command {
        use clap::Arg;

        let mut app = clap::command!()
            .about("Control various aspects of Microsoft Surface devices")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .infer_subcommands(true)
            .arg(Arg::new("quiet")
                .help("Keep output quiet")
                .short('q')
                .long("quiet")
                .action(clap::ArgAction::SetTrue)
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
