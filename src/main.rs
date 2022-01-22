#![deny(clippy::all)]
extern crate env_logger;
#[macro_use]
extern crate log;

use std::path::PathBuf;

use structopt::StructOpt;

use rtrack::{handle_track, TrackCommand, TrackReturn};

#[derive(Debug, StructOpt)]
#[structopt(name = "rtrack", about = "minimal localized version control system")]
pub struct Cli {
    /// diff file
    #[structopt(short, long)]
    diff: bool,

    /// Add a file to rhe repo
    #[structopt(parse(from_os_str))]
    file_name: PathBuf,
}

impl Cli {
    /// get_local_cli converts from a structopt based Cli (based on the command line arguments)
    /// to a rtrack::TrackCommand which is an enum the library uses.
    fn get_local_cli(&self) -> TrackCommand {
        if self.diff {
            return TrackCommand::Diff(self.file_name.clone());
        }
        TrackCommand::Commit(self.file_name.clone())
    }
}

fn main() {
    env_logger::init();
    info!("rtrack 0.2.0 started");

    let args = Cli::from_args();

    match handle_track(args.get_local_cli()) {
        Ok(TrackReturn::Rval(rv)) => rv,
        Ok(TrackReturn::DiffRet(rv, _)) => rv,
        Err(e) => {
            error!("error in main: {}", e);
            eprintln!("{}", e);
            -1
        }
    };
}
