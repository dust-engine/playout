#![feature(alloc_layout_extra)]

#[cfg(feature = "parser")]
pub mod parser;
mod types;

use std::collections::BTreeMap;

pub use types::*;

#[cfg(feature = "glsl")]
mod glsl;

#[derive(Default)]
pub struct PlayoutModule {
    pub descriptor_sets: Vec<SetLayout>,
    pub push_constants: PushConstantsLayout,
    pub data_structs: BTreeMap<String, DataStruct>,
}
