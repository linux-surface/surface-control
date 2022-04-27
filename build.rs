use std::env;
use clap_complete::shells;

#[allow(dead_code)]
#[path = "src/cli/mod.rs"]
mod cli;

#[allow(dead_code)]
#[path = "src/sys/mod.rs"]
mod sys;


fn main() {
    let outdir = env::var_os("CARGO_TARGET_DIR")
        .or_else(|| env::var_os("OUT_DIR"))
        .unwrap();

    let reg = cli::build();
    let mut app = reg.cli();

    clap_complete::generate_to(shells::Bash, &mut app, "surface", &outdir).unwrap();
    clap_complete::generate_to(shells::Zsh, &mut app, "surface", &outdir).unwrap();
    clap_complete::generate_to(shells::Fish, &mut app, "surface", &outdir).unwrap();
}
