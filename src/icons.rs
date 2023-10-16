use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

pub type Icons = HashMap<String, Vec<String>>;

pub fn get_icons() -> Icons {
    // Returns a HashMap with the icon names as keys and a vector of possible folder names as values
    // Consider the folder names as tokens, so if a folder name is "folder name" then the tokens are "folder" and "name"
    let mut icons: Icons = HashMap::new();
    let file: File = File::open(Path::new("icons.txt")).unwrap();
    let reader: BufReader<File> = BufReader::new(file);
    for line in reader.lines() {
        let line: String = line.unwrap();
        let line: Vec<&str> = line.split(":").collect();
        let icon_name: String = line[0].to_string().trim().to_string();
        let folder_names: Vec<String> = line[1].split(",").map(|s| s.to_string().trim().to_string()).collect();
        icons.insert(icon_name, folder_names);
    }
    icons
}
