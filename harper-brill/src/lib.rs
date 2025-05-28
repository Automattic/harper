use lazy_static::lazy_static;
use std::sync::Arc;

pub use harper_pos_utils::{FreqDict, UPOS};

const FREQ_DICT_SOURCE: &str = include_str!(concat!(env!("OUT_DIR"), "/freq_dict.json"));

lazy_static! {
    static ref FREQ_DICT: Arc<FreqDict> = Arc::new(uncached_prebuilt_freq_dict());
}

fn uncached_prebuilt_freq_dict() -> FreqDict {
    serde_json::from_str(FREQ_DICT_SOURCE).unwrap()
}

pub fn prebuilt_freq_dict() -> Arc<FreqDict> {
    (*FREQ_DICT).clone()
}
