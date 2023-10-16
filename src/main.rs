use std::{path::Path, time::Duration};

extern crate gio;
use gio::prelude::*;

use notify::{Config, Watcher, RecommendedWatcher, RecursiveMode, Result};
use users::get_current_username;

use icons::{Icons, get_icons};
mod icons;


fn set_folder_icon(folder_path: &str, icon_path: &str) {
    println!("Now setting the icon: {} to the folder: {}", icon_path, folder_path);
    let file: gio::File = gio::File::for_path(folder_path);
    let file_info: gio::FileInfo = file.query_info("standard::icon", gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS, gio::Cancellable::NONE).unwrap();
    println!("file_info: {:?}", file_info.list_attributes(None));
    let icon: gio::Icon = gio::Icon::for_string(format!("file://{}", icon_path).as_str()).expect("Failed to create icon");
    file_info.set_icon(&icon);
    file.set_attribute_string("metadata::custom-icon", format!("file://{}", icon_path).as_str(), gio::FileQueryInfoFlags::NONE, gio::Cancellable::NONE).unwrap();
    // file.set_attributes_from_info(&file_info, gio::FileQueryInfoFlags::NOFOLLOW_SYMLINKS, gio::Cancellable::NONE).unwrap();
}

fn check_set_folder_icon(icons: &Icons, filename: &str, folder_path: &str, username: &str) {
    icons.iter().for_each(|(icon_name, folder_names)| {
        folder_names.iter().for_each(|folder_name| {
            if filename.contains(folder_name) {
                println!("{} contains {} -> We set the icon: {}", filename, folder_name, icon_name);
                set_folder_icon(folder_path, format!("/home/{}/gnome_folder_icons/{}", username, icon_name).as_str());
            }
        });
    });
}

fn recursively_look_for_matching_folders(icons: &Icons, username: &str, cpath: &str) {
    // This method iterates for each folder inside of /home/username/
    // get all the folders inside of path
    let path: std::path::PathBuf = std::path::PathBuf::from(cpath);
    for entry in std::fs::read_dir(path).unwrap() {
        let entry: std::fs::DirEntry = entry.unwrap();
        let path: std::path::PathBuf = entry.path();
        if path.is_dir() {
            let path_str: &str = path.to_str().unwrap().trim();
            if !path_str.split("/").last().unwrap().starts_with(".") {
                println!("First Sweep on path -> {}", path_str);
                // if the directory has a name that is in the icons hashmap, then set the icon
                check_set_folder_icon(icons, path_str.split("/").last().unwrap(), path_str, username);
                // Make also a first time recursive sweep of the directory
                recursively_look_for_matching_folders(icons, username, path_str);
            }
        }
    }
}

fn main() -> Result<()> {
    let username: std::ffi::OsString = get_current_username().unwrap();
    let userstring: String = username.to_str().unwrap().to_string();
    // We need to download the gnome_folder_icons from github
    // We will place them in /usr/share/icons/gnome_folder_icons
    // Use the system terminal to execute git clone
    let output: std::process::Output = std::process::Command::new("git")
        .arg("clone")
        .arg("https://github.com/Samu01Tech/gnome-folder-icons.git")
        .arg(format!("/home/{}/gnome_folder_icons", username.to_str().unwrap()))
        .output()
        .expect("failed to execute process");
    println!("output: {:?}", output);
    // We load the icons hashmap
    let icons: Icons = get_icons();
    let icons_clone: Icons = icons.clone();

    // Automatically select the best implementation for your platform.
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res: std::result::Result<notify::Event, notify::Error>| {
        match res {
           Ok(event) => {
                if event.kind.is_create() || (event.kind.is_modify() && event.kind == notify::EventKind::Modify(notify::event::ModifyKind::Name(notify::event::RenameMode::Both)) && event.paths.len() == 2) {
                    let filename: String = event.paths[event.paths.len() - 1].to_str().unwrap().to_string().split("/").last().unwrap().to_string();
                    let folderpath: &str = event.paths[event.paths.len() - 1].to_str().unwrap();
                    check_set_folder_icon(&icons, filename.as_str(), folderpath, userstring.as_str());
                }
           },
           Err(err) => {
                println!("watch error: {:?}", err);
           }
        }
    })?;
    let config: Config = Config::default()
        .with_poll_interval(Duration::from_secs(2))
        .with_compare_contents(true);
    let res = watcher.configure(config)?;
    println!("Watcher: {:?}", watcher);
    println!("watcher.configure: {:?}", res);

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    // Read the names of the paths inside of /home/username/
    // Then make a watch path for each directory within /home/username/ which is not hidden
    for entry in std::fs::read_dir(format!("/home/{}", username.to_str().unwrap())).unwrap() {
        let entry: std::fs::DirEntry = entry.unwrap();
        let path: std::path::PathBuf = entry.path();
        if path.is_dir() {
            let path_str: &str = path.to_str().unwrap().trim();
            if !path_str.split("/").last().unwrap().starts_with(".") {
                println!("Watching path: {}", path_str);
                watcher.watch(Path::new(path_str), RecursiveMode::Recursive)?;
                // if the directory has a name that is in the icons hashmap, then set the icon
                check_set_folder_icon(&icons_clone, path_str.split("/").last().unwrap(), path_str, username.clone().to_str().unwrap());
                // Make also a first time recursive sweep of the directory
                recursively_look_for_matching_folders(&icons_clone, username.clone().to_str().unwrap(), path_str.clone());
            }
        }
    }
    // This is a simple loop, but you may want to use more complex logic here,
    // for example to handle I/O.
    loop { std::thread::sleep(Duration::from_secs(1)); }
}
