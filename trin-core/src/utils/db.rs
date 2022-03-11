use std::{env, fs};

use crate::cli::TrinConfig;
use directories::ProjectDirs;
use discv5::enr::NodeId;
use rocksdb::{Options, DB};
use std::path::PathBuf;
use tempdir::TempDir;

const TRIN_DATA_ENV_VAR: &str = "TRIN_DATA_PATH";

pub fn setup_temp_data_dir() {
    // NOTE: A TRIN_DATA_PATH will overwrite --ephemeral usage
    match env::var(TRIN_DATA_ENV_VAR) {
        Ok(_) => {}
        _ => {
            let temp_dir = TempDir::new("Trin").expect("Failed to create ephmeral data directory");
            env::set_var(TRIN_DATA_ENV_VAR, temp_dir.path());
        }
    }
}

pub fn get_data_dir(node_id: NodeId) -> String {
    let trin_config = TrinConfig::from_cli();

    if trin_config.ephemeral {
        setup_temp_data_dir();
    }

    let application_path = env::var(TRIN_DATA_ENV_VAR).unwrap_or_else(|_| get_default_data_dir());

    fs::create_dir_all(&application_path).expect("Unable to create data directory folder");

    // Append first 8 characters of Node ID to session path
    let node_id_string = hex::encode(node_id.raw());
    let suffix = &node_id_string[..8];
    let session_path = PathBuf::from(&application_path).join(format!("Trin_{}", &suffix));

    session_path.into_os_string().into_string().unwrap()
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
