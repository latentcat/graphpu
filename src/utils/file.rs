use std::{path::PathBuf, process::Command};

pub fn pick_folder() -> Option<PathBuf> {
    let desktop_path = if cfg!(window) {
        "%USERPROFILE%/Desktop"
    } else {
        "~/Desktop"
    };

    rfd::FileDialog::new()
        .set_directory(desktop_path)
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