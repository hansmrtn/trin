use std::{env, fs};

use directories::ProjectDirs;
use discv5::enr::NodeId;
use rocksdb::{Options, DB};
use std::path::PathBuf;
use std::time::SystemTime;
use crate::cli::TrinConfig;

const TRIN_DATA_ENV_VAR: &str = "TRIN_DATA_PATH";

pub fn get_data_dir(node_id: NodeId) -> String {
    let application_path = env::var(TRIN_DATA_ENV_VAR).unwrap_or_else(|_| get_default_data_dir());
    let trin_config = TrinConfig::from_cli();

    fs::create_dir_all(&application_path).expect("Unable to create data directory folder");
    
    // Append first 8 characters of Node ID
    let node_id_string = hex::encode(node_id.raw());
    let suffix = &node_id_string[..8];
    let path = format!("{}/Trin_{}", &application_path, &suffix);
    
    if trin_config.ephemeral {
        fs::create_dir_all(&path).expect("Unable to create ephemeral data directory");
        let paths = fs::read_dir(&application_path).unwrap();
        // NOTE: SystemTime is machine dependent
        let mut min_timestamp = SystemTime::now();
        let mut min_path = PathBuf::new();
        let mut n_paths = 0;
        for p in paths {
            n_paths += 1;
            if let Ok(p) = p {
                // Get oldest last-modified Trin_nodeId directory timestamp and path
                if let Ok(metadata) = p.metadata() {
                    if metadata.modified().unwrap() < min_timestamp {
                        min_timestamp = metadata.modified().unwrap();
                        min_path = p.path();
                    }
                } else {
                    println!("Couldn't get metadata for {:?}", p.path());
                }
            }
        }
        // Remove the oldest last-modified Trin_nodeId directory 
        // TODO: Twelve is arbitrary and should probably be a user preference
        if n_paths > 12 {
            fs::remove_dir_all(&min_path)
                .expect("Failed to remove oldest modified ephemeral data directory");
        }
    }

    path
}

pub fn get_default_data_dir() -> String {
    //  Linux:	    $XDG_DATA_HOME/Trin or $HOME/.local/share/Trin	
    //  macOS:	    $HOME/Library/Application Support/Trin	
    //  Windows:    C:\Users\Username\AppData\Roaming\Trin\data
    let application_path = "Trin".to_owned();

    match ProjectDirs::from("", "", &application_path) {
        Some(proj_dirs) => proj_dirs.data_local_dir().to_str().unwrap().to_string(),
        None => panic!("Unable to find data directory"),
    }
}

pub fn setup_overlay_db(node_id: NodeId) -> DB {
    let data_path = get_data_dir(node_id);
    let mut db_opts = Options::default();
    db_opts.create_if_missing(true);
    DB::open(&db_opts, data_path).unwrap()
}
