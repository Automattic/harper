pub use harper_pos_utils::*;

const FREQ_DICT: &str = include_str!(concat!(env!("OUT_DIR"), "/freq_dict.json"));

pub fn prebuilt_freq_dict() -> FreqDict {
    serde_json::from_str(FREQ_DICT).unwrap()
}
