mod error;
mod cli;
mod sys;

use crate::error::CliError;


fn main() -> Result<(), CliError> {
    let cmdr = cli::build();

    let matches = cmdr.cli().get_matches();
    match matches.subcommand() {
        (cmd, Some(m)) => cmdr.execute(cmd, m)?,
        _              => unreachable!(),
    }

    Ok(())
}
