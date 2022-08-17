use std::{path::PathBuf, process::Command};
use directories::UserDirs;

pub fn pick_folder() -> Option<PathBuf> {
    let dir = desktop_dir_or_empty();

    rfd::FileDialog::new()
        .set_directory(dir)
        .pick_folder()
}

pub fn pick_csv() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Text File", &["txt", "csv"])
        .pick_file()
}

pub fn path_to_string(path: &Option<PathBuf>) -> Option<String> {
    path.as_ref().map(|path| path.display().to_string())
}

pub fn system_open_directory(output_directory: &str) {
    if cfg!(windows) {
        Command::new("explorer")
            .arg(output_directory) // <- Specify the directory you'd like to open.
            .spawn()
            .unwrap();
    } else if cfg!(unix) {
        Command::new("open")
            .arg(output_directory) // <- Specify the directory you'd like to open.
            .spawn()
            .unwrap();
    }
}

pub fn desktop_dir_or_empty() -> String {
    let mut dir = String::from("");
    if let Some(user_dirs) = UserDirs::new() {
        if let Some(desktop_dir) = user_dirs.desktop_dir() {
            dir = desktop_dir.display().to_string();
        }
    }
    dir
}