#![deny(clippy::all)]
//! Extra simple source code control.
//!
//!  Rtrack is a extremly simple source control system. Basically, it
//!  automates the process of making a copy of the file with a different name.
//!  All the copies are kept in the `.track` directory. Restoring is done by manually
//!  copying files, but the process of backing up is very simple, just `rtrack <file>`.
//!
//!  Also, writen as a library with a main.rs, it is easy to include this functionality
//!  in another program.
//!
//!  # examples
//! ```
//!  # use std::path::Path;
//!  # use std::fs::File;
//!  # use std::env::set_current_dir;
//!  # use tempfile::tempdir;
//!  # use std::io::Write;
//!  # use rtrack::TrackCommand::Commit;
//!  # use rtrack::handle_track;
//!  # let dir = tempdir().expect("tempdir failed");
//!  # let file_path = dir.path().join("filename");
//!  # let mut file = File::create(file_path.clone()).expect("file create failed");
//!  # writeln!(file, "0123456789").expect("writeln failed");
//!  # set_current_dir(dir.path()).expect("set_current_dir failed");
//!  use std::path::PathBuf;
//!
//!  let file_path = PathBuf::from("filename");
//!  let command = Commit(file_path);
//!  let res = handle_track(command).unwrap();
//!  ```

extern crate log;
extern crate text_diff;

use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use log::{error, trace};
use text_diff::{diff, print_diff, Difference};

use crate::TrackReturn::{DiffRet, Rval};

#[derive(Debug, Clone)]
pub enum TrackCommand {
    Commit(PathBuf),
    Diff(PathBuf),
}

#[derive(Debug)]
pub enum TrackReturn {
    Rval(i32),
    DiffRet(i32, Vec<Difference>),
}

const TRACK_DIR: &str = ".track";

/// Central dispatch function
pub fn handle_track(args: TrackCommand) -> Result<TrackReturn, &'static str> {
    trace!("handle_track: {:?}", args);

    match args {
        TrackCommand::Commit(path) => match handle_checkin(&path) {
            Ok(i) => Ok(Rval(i)),
            Err(e) => {
                error!("error in handle_track()/commit: {}", e);
                Err(e)
            }
        },
        TrackCommand::Diff(path) => match handle_diff(&path) {
            Ok((i, v)) => Ok(DiffRet(i, v)),
            Err(e) => {
                error!("error in handle_track()/diff: {}", e);
                Err(e)
            }
        },
    }
}

/// Function to handle a checkin
fn handle_checkin(file_name: &Path) -> Result<i32, &'static str> {
    trace!("handle_checkin() file:   {:?}", file_name);
    trace!("handle_checkin() cwd():{:?}", std::env::current_dir());

    // create .track directory if needed
    if !file_name.exists() {
        error!(
            "handle_checkin: file doesn't exist {} in {}",
            file_name.to_str().unwrap(),
            std::env::current_dir().unwrap().to_str().unwrap()
        );
        return Err("File doesn't exist.");
    }

    let mut track_dir = PathBuf::new();
    track_dir.push(TRACK_DIR);

    if !track_dir.exists() {
        if let Err(e) = fs::create_dir(&track_dir) {
            error!(
                "cannot create track dir at {:?}, error: {}",
                &track_dir,
                e.to_string()
            );
            return Err("error creating dir");
        }
    }

    let pre_f_name = file_name
        .file_name()
        .expect("handle_checkin(): bad filename 1")
        .to_str()
        .expect("handle_checkin bad filename 2")
        .to_owned();

    for i in 1..1000 {
        let mut f_name = pre_f_name.clone();
        f_name.push_str(format!(".{:03}", i).as_str());
        let mut check_path = PathBuf::new();
        check_path.push(TRACK_DIR);
        check_path.push(f_name);
        trace!("{:?}", check_path);
        if !check_path.exists() {
            fs::copy(file_name, check_path).expect("handle_checkin(): copy failed");
            return Ok(0);
        }
    }

    error!("handle_checkin(): Unable to copy file: {}", pre_f_name);
    Err("Unable to copy file.")
}

/// handle diffing a file in the repo
fn handle_diff(file_name: &Path) -> Result<(i32, Vec<Difference>), &'static str> {
    let mut track_dir = PathBuf::new();
    track_dir.push(TRACK_DIR);

    if !track_dir.exists() {
        error!("No .track directory");
        return Err("No track repository");
    }

    let mut f_name = file_name
        .file_name()
        .expect("handle_diff(): bad filename 1")
        .to_str()
        .expect("handle_diff(): bad filename 2")
        .to_owned();

    f_name.push_str(".*");
    let mut check_path = PathBuf::new();
    check_path.push(TRACK_DIR);
    check_path.push(f_name);

    trace!("handle_diff() check_path: {:?}", check_path);
    // need to sort this somehow....
    let f = glob::glob(check_path.to_str().expect("handle_diff() bad path to str"))
        .unwrap()
        .last()
        .unwrap();

    let mut orig = File::open(file_name).expect("handle_diff(): error opening file");
    let mut orig_cont = String::new();
    if let Err(res) = orig.read_to_string(&mut orig_cont) {
        error!("Unable to read file {:?}", file_name);
        eprintln!("Unable to read file {:?}", file_name);
        eprintln!("{}", res.to_string());
        return Err("handle_diff(): unable to read file 1");
    }

    let other_file = f.expect("Error generating file name");
    let mut edit = File::open(other_file.clone()).unwrap();
    let mut edit_cont = String::new();
    if let Err(res) = edit.read_to_string(&mut edit_cont) {
        error!("Unable to read file {:?}", other_file);
        eprintln!("Unable to read file {:?}", other_file);
        eprintln!("{}", res.to_string());
        return Err("Shout and Scream and never dream!"); // Fixme
    }

    print_diff(orig_cont.as_str(), edit_cont.as_str(), "\n");

    Ok(diff(orig_cont.as_str(), edit_cont.as_str(), "\n"))
}

// ----------------------------------------
// ---------------- testing ---------------
// ----------------------------------------

#[cfg(test)]
mod tests {
    use std::env::set_current_dir;
    use std::fs::File;
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::string::String;

    use tempfile::{tempdir, TempDir};

    // when testing use `cargo test -- --test-threads=1`
    // the current_dir is global state which can be a problem if multiple
    // threads are used
    use crate::{handle_checkin, handle_diff, handle_track, TrackCommand, TrackReturn};

    const TEST_FILE: &str = "rtrack_test";

    fn test_setup() -> (TempDir, PathBuf) {
        let dir = tempdir().expect("tempdir failed");
        let file_path = dir.path().join(TEST_FILE);
        {
            let mut file = File::create(file_path.clone()).expect("file create failed");
            writeln!(file, "0123456789").expect("writeln failed")
        }

        set_current_dir(dir.path()).expect("set_current_dir failed");

        (dir, file_path)
    }

    #[test]
    fn test_handle_track() {
        let (dir, file_path) = test_setup();
        match handle_track(TrackCommand::Commit(file_path.clone())) {
            Ok(rv) => {
                if let TrackReturn::Rval(i) = rv {
                    assert_eq!(i, 0)
                } else {
                    panic!()
                }
            }
            Err(e) => {
                // need to keep a reference to dir around, otherwise
                // it gets removed on drop
                eprintln!("dir: {:?}", dir);
                eprintln!("{}", e);
                panic!();
            }
        }

        match handle_track(TrackCommand::Diff(file_path.clone())) {
            Ok(rv) => {
                if let TrackReturn::DiffRet(i, _) = rv {
                    assert_eq!(i, 0)
                } else {
                    panic!()
                }
            }
            Err(_) => panic!(),
        }
    }

    #[test]
    fn test_handle_checkin() {
        let (dir, _file_path) = test_setup();
        let p = dir.path().to_path_buf();
        let t = p.clone().join(Path::new(".track"));
        let r1 = t.join(String::from(TEST_FILE) + ".001");
        let r2 = t.join(String::from(TEST_FILE) + ".002");

        assert!(!t.exists());
        let rv = handle_checkin(&Path::new(TEST_FILE));

        match rv {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                panic!();
            }
        }

        assert!(t.exists());
        assert!(r1.exists());
        assert!(!r2.exists());

        let rv = handle_checkin(Path::new(TEST_FILE));

        match rv {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
                panic!();
            }
        }

        assert!(r2.exists());

        let rv = handle_checkin(Path::new((String::from("bad_") + TEST_FILE).as_str()));

        match rv {
            Ok(_) => {
                panic!()
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    #[test]
    fn test_handle_diff() {
        let (_dir, _file_path) = test_setup();

        let rv1 = handle_diff(Path::new(TEST_FILE));
        if rv1.is_ok() {
            panic!()
        }

        let rv2 = handle_checkin(Path::new(TEST_FILE));
        if rv2.is_err() {
            panic!()
        }

        let rv3 = handle_diff(Path::new(TEST_FILE));
        match rv3 {
            Ok((i, _v)) => {
                if i != 0 {
                    panic!()
                }
            }
            Err(e) => {
                eprintln!("error: {}", e);
                panic!()
            }
        };
    }
}
