pub mod parser;
mod types;

pub use types::*;

mod glsl;

#[derive(Default)]
pub struct PlayoutModule {
    pub descriptor_sets: Vec<SetLayout>,
    pub data_structs: Vec<DataStruct>,
}
