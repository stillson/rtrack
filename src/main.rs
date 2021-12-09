#![deny(clippy::all)]
use std::path::PathBuf;
use structopt::StructOpt;

#[macro_use]
extern crate log;

#[derive(Debug, StructOpt)]
#[structopt(name = "rtrack", about = "minimal localized version control system")]
pub struct Cli {
    /// diff file
    #[structopt(short, long)]
    diff: bool,

    #[structopt(parse(from_os_str))]
    file_name: PathBuf,
}

impl Cli {
    /// get_local_cli converts from a structopt based Cli (based on the command line arguments)
    /// to a rtrack::TrackCommand which is an enum the library uses.
    fn get_local_cli(&self) -> rtrack::TrackCommand {
        if self.diff {
            return rtrack::TrackCommand::Diff(self.file_name.clone());
        }
        rtrack::TrackCommand::Commit(self.file_name.clone())
    }
}

fn main() {
    info!("rtrack started");

    let args = Cli::from_args();

    match rtrack::handle_track(args.get_local_cli()) {
        Ok(rv) => rv,
        Err(e) => {
            eprintln!("{}", e);
            0
        }
    };
}
