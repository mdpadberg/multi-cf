use std::path::PathBuf;

pub struct Options {
    pub cf_binary_name: String,
    pub mcf_home: String,
}

impl Options {
    pub fn get_mcf_home_path_buf(&self) -> PathBuf {
        PathBuf::from(&self.mcf_home)
    }

    pub fn new(cf_binary_name: Option<String>, mcf_home: Option<String>) -> Options {
        let mut options = Options::default();

        if let Some(some) = mcf_home {
            options.mcf_home = some;
        }

        if let Some(some) = cf_binary_name {
            options.cf_binary_name = some;
        }

        options
    }
}

impl Default for Options {
    fn default() -> Options {
        Options {
            cf_binary_name: "cf".to_string(),
            mcf_home: dirs::home_dir()
                .expect("OS not supported")
                .to_str()
                .unwrap()
                .to_string(),
        }
    }
}

//TODO: write tests