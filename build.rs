use std::env;
use clap::Shell;

#[allow(dead_code)]
#[path = "src/error.rs"]
mod error;

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

    let mut app = cli::build().cli();
    app.gen_completions("surface", Shell::Bash, &outdir);
    app.gen_completions("surface", Shell::Zsh,  &outdir);
    app.gen_completions("surface", Shell::Fish, &outdir);
}
