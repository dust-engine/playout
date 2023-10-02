use std::fmt::Debug;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct ShaderStages: u32 {
        const VERTEX = 0x1;
        const TELLESLATION_CONTROL = 0x2;
        const TELLESLATION_EVALUATION = 0x4;
        const GEOMETRY = 0x8;
        const FRAGMENT = 0x10;
        const COMPUTE = 0x20;
        const RAYGEN = 0x100;
        const ANY_HIT = 0x200;
        const CLOSEST_HIT = 0x400;
        const MISS = 0x800;
        const INTERSECTION = 0x1000;
        const CALLABLE = 0x2000;
        const TASK = 0x40;
        const MESH = 0x80;
    }
}

pub enum DescriptorType {
    Sampler,
    StorageImage { format: ImageFormat },
    SampledImage,
    UniformBuffer { ty: Type },
    StorageBuffer { ty: Type },
    AccelerationStructure,
}

impl Debug for DescriptorType {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[derive(Debug)]
pub struct SetLayout {
    pub name: String,
    pub set: u32,
    pub bindings: Vec<Binding>,
}

#[derive(Default)]
pub struct PushConstantsLayout {
    pub name: String,
    pub fields: Vec<PushConstantField>,
}

pub struct PushConstantField {
    pub field: Field,
    pub stages: ShaderStages,
}

#[derive(Debug)]
pub struct Binding {
    pub ident: String,
    pub binding: u32,
    pub stages: ShaderStages,
    pub descriptor_type: DescriptorType,
    pub descriptor_count: u32, // ALso needs: binding id, immutable sampler, shader stage flags
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    RGBA32_Float,
    RGBA16_Float,
    RG32_Float,
    RG16_Float,
    R11G11B10_Float,
    R32_Float,
    R16_Float,

    RGBA16_UNorm,
    RGB10A2_UNorm,
    RBGA8_UNorm,
    RG16_UNorm,
    RG8_UNorm,
    R16_UNorm,
    R8_UNorm,

    RGBA16_SNorm,
    RBGA8_SNorm,
    RG16_SNorm,
    RG8_SNorm,
    R16_SNorm,
    R8_SNorm,

    RGBA32_SInt,
    RGBA16_SInt,
    RGBA8_SInt,
    RG32_SInt,
    RG16_SInt,
    RG8_SInt,
    R32_SInt,
    R16_SInt,
    R8_SInt,

    RGBA32_UInt,
    RGBA16_UInt,
    RGB10A2_UInt,
    RGBA8_UInt,
    RG32_UInt,
    RG16_UInt,
    RG8_UInt,
    R32_UInt,
    R16_UInt,
    R8_UInt,
}

impl ImageFormat {
    pub fn data_mode(&self) -> ImageFormatDataMode {
        use ImageFormat::*;
        match self {
            RGBA32_Float | RGBA16_Float | RG32_Float | RG16_Float | R11G11B10_Float | R32_Float
            | R16_Float => ImageFormatDataMode::Float,

            RGBA16_UNorm | RGB10A2_UNorm | RBGA8_UNorm | RG16_UNorm | RG8_UNorm | R16_UNorm
            | R8_UNorm => ImageFormatDataMode::UNorm,

            RGBA16_SNorm | RBGA8_SNorm | RG16_SNorm | RG8_SNorm | R16_SNorm | R8_SNorm => {
                ImageFormatDataMode::SNorm
            }

            RGBA32_SInt | RGBA16_SInt | RGBA8_SInt | RG32_SInt | RG16_SInt | RG8_SInt
            | R32_SInt | R16_SInt | R8_SInt => ImageFormatDataMode::SInt,

            RGBA32_UInt | RGBA16_UInt | RGB10A2_UInt | RGBA8_UInt | RG32_UInt | RG16_UInt
            | RG8_UInt | R32_UInt | R16_UInt | R8_UInt => ImageFormatDataMode::UInt,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ImageFormatDataMode {
    Float,
    UNorm,
    SNorm,
    SInt,
    UInt,
}

pub struct DataStruct {
    pub ident: String,
    pub fields: Vec<Field>,
}

pub struct Field {
    pub ident: Option<String>,
    pub ty: Type,
}

pub enum Type {
    Array { ty: Box<Type>, size: usize },
    Primitive(PrimitiveType),
    Path(String),
    Slice { ty: Box<Type> },
    //Path, for nested structs
}

pub enum PrimitiveTypeSingle {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F16,
    F32,
    F64,
    Bool,
}

pub enum PrimitiveType {
    Single(PrimitiveTypeSingle),
    Vec {
        ty: PrimitiveTypeSingle,
        length: u8,
    },
    Mat {
        ty: PrimitiveTypeSingle,
        rows: u8,
        columns: u8,
    },
}
