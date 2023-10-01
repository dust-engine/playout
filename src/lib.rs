pub mod parser;
mod types;

pub use types::*;

mod glsl;

pub struct PlayoutModule {
    pub descriptor_sets: Vec<SetLayout>,
}
