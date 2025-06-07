
pub struct Config {
    templ_dir: Box<str>,
}

impl Config {
    pub fn templ_dir(&self) -> &str {
        &self.templ_dir
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            templ_dir: String::from("templates").into_boxed_str(),
        }
    }
}

