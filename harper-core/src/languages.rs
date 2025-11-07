use std::default::Default;

pub enum Language {
    English,
    Portuguese,
}

impl Default for Language {
    fn default() -> Self {
        Self::English
    }
}
