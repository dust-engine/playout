#[cfg(feature = "parser")]
pub mod parser;
mod types;

use std::collections::HashMap;

pub use types::*;

#[cfg(feature = "glsl")]
mod glsl;

#[derive(Default)]
pub struct PlayoutModule {
    pub descriptor_sets: Vec<SetLayout>,
    pub push_constants: PushConstantsLayout,
    pub data_structs: HashMap<String, DataStruct>,
}
