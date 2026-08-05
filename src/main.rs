mod cli;
mod commands;
mod git;
mod model;
mod skill;
mod store;
mod util;

fn main() {
    if let Err(error) = commands::run(cli::parse(std::env::args().skip(1).collect())) {
        eprintln!("tincan: {error}");
        std::process::exit(1);
    }
}
