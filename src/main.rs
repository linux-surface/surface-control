mod cli;
mod sys;

use anyhow::Result;


fn main() -> Result<()> {
    let cmdr = cli::build();

    let matches = cmdr.cli().get_matches();
    match matches.subcommand() {
        Some((cmd, m)) => cmdr.execute(cmd, m)?,
        _              => unreachable!(),
    }

    Ok(())
}
