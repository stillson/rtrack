#[macro_use]
extern crate log;
extern crate text_diff;

use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use text_diff::print_diff;

#[derive(Debug, Clone)]
pub enum TrackCommand {
    Commit(PathBuf),
    Diff(PathBuf),
}

const TRACK_DIR: &str = ".track";

pub fn handle_track(args: TrackCommand) -> Result<i32, &'static str> {
    debug!("{:?}", args);

    match args {
        TrackCommand::Commit(path) => handle_checkin(&path),
        TrackCommand::Diff(path) => handle_diff(&path),
    }
}

fn handle_checkin(file_name: &PathBuf) -> Result<i32, &'static str> {
    // create .track directory if needed
    let mut track_dir = PathBuf::new();
    track_dir.push(TRACK_DIR);

    if !track_dir.exists() {
        if let Err(_e) = fs::create_dir(track_dir) {
            return Err("error creating dir");
        }
    }

    let pre_f_name = file_name
        .file_name()
        .expect("bad filename")
        .to_str()
        .expect("bad file name;")
        .to_owned();

    for i in 1..1000 {
        let mut f_name = pre_f_name.clone();
        f_name.push_str(format!(".{:03}", i).as_str());
        let mut check_path = PathBuf::new();
        check_path.push(TRACK_DIR);
        check_path.push(f_name);
        if !check_path.exists() {
            fs::copy(file_name, check_path).expect("copy failed");
            return Ok(0);
        }
    }

    Err("Unable to copy file.")
}

fn handle_diff(file_name: &PathBuf) -> Result<i32, &'static str> {
    let mut track_dir = PathBuf::new();
    track_dir.push(TRACK_DIR);

    if !track_dir.exists() {
        return Ok(0);
    }

    let mut f_name = file_name
        .file_name()
        .expect("bad filename")
        .to_str()
        .expect("bad file name;")
        .to_owned();

    f_name.push_str(".*");
    let mut check_path = PathBuf::new();
    check_path.push(TRACK_DIR);
    check_path.push(f_name);

    // need to sort this somehow....
    let f = glob::glob(check_path.to_str().expect("oops"))
        .unwrap()
        .last()
        .unwrap();

    debug!("{:?}", f);

    let mut orig = File::open(file_name).unwrap();
    let mut orig_cont = String::new();
    if let Err(res) = orig.read_to_string(&mut orig_cont) {
        eprintln!("Scream and shout and run about!");
        eprintln!("{}", res.to_string());
        return Err("Scream and shout and run about"); // FIXME
    }

    let mut edit = File::open(f.unwrap()).unwrap();
    let mut edit_cont = String::new();
    if let Err(res) = edit.read_to_string(&mut edit_cont) {
        eprintln!("{}", res.to_string());
        return Err("Shout and Scream and never dream!"); // Fixme
    }

    print_diff(orig_cont.as_str(), edit_cont.as_str(), "\n");

    Ok(0)
}
