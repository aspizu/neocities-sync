use std::{fs, path::PathBuf};

pub struct Data {
    config_dir: PathBuf,
    default_username: Option<String>,
}

impl Data {
    pub fn new() -> Self {
        let dirs =
            directories::ProjectDirs::from("org", "aspizu", "neocities-sync").unwrap();
        let config_dir = dirs.config_dir().to_path_buf();
        let default_username_path = config_dir.join("default_username.txt");
        fs::create_dir_all(&config_dir).unwrap();
        let default_username = fs::read_to_string(default_username_path).ok();
        Self { config_dir, default_username }
    }

    pub fn get_default_username(&self) -> Option<&str> {
        self.default_username.as_deref()
    }

    pub fn set_default_username(&mut self, username: String) {
        let default_username_path = self.config_dir.join("default_username.txt");
        fs::write(default_username_path, &username).unwrap();
        self.default_username = Some(username);
    }

    pub fn remove_default_username(&mut self) {
        let default_username_path = self.config_dir.join("default_username.txt");
        fs::remove_file(default_username_path).unwrap();
        self.default_username = None;
    }
}
