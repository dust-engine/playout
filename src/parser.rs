use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
};

use crate::{
    Binding, DataStruct, DescriptorType, Field, ImageFormat, PlayoutModule, PrimitiveType,
    PrimitiveTypeSingle, PushConstantField, PushConstantsLayout, SetLayout, ShaderStages, Type,
};

impl Parse for DescriptorType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty = input.parse::<syn::Ident>()?;
        let ty = match ty.to_string().as_str() {
            "StorageImage" => {
                let _left: syn::Token![<] = input.parse()?;
                let format: ImageFormat = input.parse()?;
                let _right: syn::Token![>] = input.parse()?;
                Self::StorageImage { format }
            }
            "SampledImage" => Self::SampledImage,
            "AccelerationStructure" => Self::AccelerationStructure,
            "UniformBuffer" => {
                let _left: syn::Token![<] = input.parse()?;
                let ty: Type = input.parse()?;
                let _right: syn::Token![>] = input.parse()?;
                Self::UniformBuffer { ty }
            }
            "StorageBuffer" => {
                let _left: syn::Token![<] = input.parse()?;
                let ty: Type = input.parse()?;
                let _right: syn::Token![>] = input.parse()?;
                Self::StorageBuffer { ty }
            }
            _ => return Err(syn::Error::new(input.span(), "Invalid descriptor type")),
        };
        Ok(ty)
    }
}

impl Parse for ImageFormat {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        let value = match ident.to_string().as_str() {
            "RGBA32_Float" => Self::RGBA32_Float,
            "RGBA16_Float" => Self::RGBA16_Float,
            "RG32_Float" => Self::RG32_Float,
            "RG16_Float" => Self::RG16_Float,
            "R11G11B10_Float" => Self::R11G11B10_Float,
            "R32_Float" => Self::R32_Float,
            "R16_Float" => Self::R16_Float,
            "RGBA16_UNorm" => Self::RGBA16_UNorm,
            "RGB10A2_UNorm" => Self::RGB10A2_UNorm,
            "RBGA8_UNorm" => Self::RBGA8_UNorm,
            "RG16_UNorm" => Self::RG16_UNorm,
            "RG8_UNorm" => Self::RG8_UNorm,
            "R16_UNorm" => Self::R16_UNorm,
            "R8_UNorm" => Self::R8_UNorm,
            "RGBA16_SNorm" => Self::RGBA16_SNorm,
            "RBGA8_SNorm" => Self::RBGA8_SNorm,
            "RG16_SNorm" => Self::RG16_SNorm,
            "RG8_SNorm" => Self::RG8_SNorm,
            "R16_SNorm" => Self::R16_SNorm,
            "R8_SNorm" => Self::R8_SNorm,
            "RGBA32_SInt" => Self::RGBA32_SInt,
            "RGBA16_SInt" => Self::RGBA16_SInt,
            "RGBA8_SInt" => Self::RGBA8_SInt,
            "RG32_SInt" => Self::RG32_SInt,
            "RG16_SInt" => Self::RG16_SInt,
            "RG8_SInt" => Self::RG8_SInt,
            "R32_SInt" => Self::R32_SInt,
            "R16_SInt" => Self::R16_SInt,
            "R8_SInt" => Self::R8_SInt,
            "RGBA32_UInt" => Self::RGBA32_UInt,
            "RGBA16_UInt" => Self::RGBA16_UInt,
            "RGBA8_UInt" => Self::RGBA8_UInt,
            "RG32_UInt" => Self::RG32_UInt,
            "RG16_UInt" => Self::RG16_UInt,
            "RG8_UInt" => Self::RG8_UInt,
            "R32_UInt" => Self::R32_UInt,
            "R16_UInt" => Self::R16_UInt,
            "R8_UInt" => Self::R8_UInt,
            _ => return Err(syn::Error::new(ident.span(), "Invalid image layout")),
        };
        Ok(value)
    }
}

impl Parse for ShaderStages {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut result = ShaderStages::empty();
        let stages: syn::punctuated::Punctuated<syn::Ident, syn::Token![|]> =
            input.parse_terminated(syn::Ident::parse, syn::Token![|])?;
        for stage in stages {
            let stage = match stage.to_string().as_str() {
                "VERTEX" => Self::VERTEX,
                "TELLESLATION_CONTROL" => Self::TELLESLATION_CONTROL,
                "TELLESLATION_EVALUATION" => Self::TELLESLATION_EVALUATION,
                "GEOMETRY" => Self::GEOMETRY,
                "FRAGMENT" => Self::FRAGMENT,
                "COMPUTE" => Self::COMPUTE,
                "RAYGEN" => Self::RAYGEN,
                "ANY_HIT" => Self::ANY_HIT,
                "CLOSEST_HIT" => Self::CLOSEST_HIT,
                "MISS" => Self::MISS,
                "INTERSECTION" => Self::INTERSECTION,
                "CALLABLE" => Self::CALLABLE,
                "TASK" => Self::TASK,
                "MESH" => Self::MESH,
                _ => return Err(syn::Error::new(stage.span(), "Invalid shader stage")),
            };
            result |= stage;
        }
        Ok(result)
    }
}

fn parse_shader_stage_attribute(input: ParseStream) -> syn::Result<ShaderStages> {
    let _pound: syn::Token![#] = input.parse()?;
    let _bang: syn::Token![!] = input.parse()?;

    let content;
    let _bracket: syn::token::Bracket = syn::bracketed!(content in input);

    let ident = content.parse::<syn::Ident>()?;
    if ident == "stage" {
        let inside;
        let _paren = syn::parenthesized!(inside in content);
        inside.parse::<ShaderStages>()
    } else {
        Err(syn::Error::new(ident.span(), "unknown attribute"))
    }
}

impl Parse for Binding {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut binding: u32 = 0;
        let mut layout: Option<String> = None;
        loop {
            if !input.peek(syn::Token![#]) {
                break;
            }
            let _pound: syn::Token![#] = input.parse()?;
            let content;
            let _bracket: syn::token::Bracket = syn::bracketed!(content in input);
            let ident = content.parse::<syn::Ident>()?;
            if ident == "binding" {
                let _eq = content.parse::<syn::Token![=]>()?;
                let binding_literal: syn::LitInt = content.parse::<syn::LitInt>()?;
                binding = binding_literal.base10_parse()?;
            } else if ident == "layout" {
                let _eq = content.parse::<syn::Token![=]>()?;
                let layout_ident: syn::Ident = content.parse::<syn::Ident>()?;
                layout = Some(layout_ident.to_string());
            } else {
                return Err(syn::Error::new(ident.span(), "unknown attribute"))
            }
        }
        let unnamed_field = input.peek(syn::Token![_]);
        let ident = if unnamed_field {
            input.call(syn::Ident::parse_any)
        } else {
            input.parse()
        }?;

        let _colon: syn::Token![:] = input.parse()?;

        let mut descriptor_count = 1;
        let descriptor_type: DescriptorType = if input.peek(syn::token::Bracket) {
            let content;
            let _bracket: syn::token::Bracket = syn::bracketed!(content in input);
            let ty: DescriptorType = content.parse()?;
            let _semicolon: syn::Token![;] = content.parse()?;
            let length = content.parse::<syn::LitInt>()?;
            descriptor_count = length.base10_parse()?;
            ty
        } else {
            input.parse()?
        };

        Ok(Binding {
            ident: ident.to_string(),
            binding,
            stages: ShaderStages::empty(),
            descriptor_type,
            descriptor_count,
            layout,
        })
    }
}

impl Parse for PushConstantsLayout {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _struct = input.parse::<syn::Token![struct]>()?;
        let name = input.parse::<syn::Ident>()?;
        let lookahead = input.lookahead1();
        if !lookahead.peek(syn::token::Brace) {
            return Err(lookahead.error());
        }
        let content;
        let _paren: syn::token::Brace = syn::braced!(content in input);

        let mut current_shader_stages = ShaderStages::empty();
        let mut fields = Vec::new();
        loop {
            if content.peek(syn::Token![#]) && content.peek2(syn::Token![!]) {
                current_shader_stages = parse_shader_stage_attribute(&content)?;
                continue;
            }
            if content.is_empty() {
                break;
            }
            if current_shader_stages.is_empty() {
                return Err(syn::Error::new(
                    content.span(),
                    "No shader stages specified for this value",
                ));
            }
            fields.push(PushConstantField {
                field: content.parse()?,
                stages: current_shader_stages,
            });
            if content.is_empty() {
                break;
            }
            let _comma: syn::Token![,] = content.parse()?;
        }
        Ok(PushConstantsLayout {
            fields,
            name: name.to_string(),
        })
    }
}

impl Parse for SetLayout {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _struct = input.parse::<syn::Token![struct]>()?;
        let name = input.parse::<syn::Ident>()?;
        let lookahead = input.lookahead1();
        if lookahead.peek(syn::token::Brace) {
            let content;
            let _paren: syn::token::Brace = syn::braced!(content in input);

            let mut current_shader_stages = ShaderStages::empty();
            let mut current_binding = 0;
            let mut bindings = Vec::new();
            loop {
                if content.peek(syn::Token![#]) && content.peek2(syn::Token![!]) {
                    current_shader_stages = parse_shader_stage_attribute(&content)?;
                    continue;
                }
                if content.is_empty() {
                    break;
                }
                if current_shader_stages.is_empty() {
                    return Err(syn::Error::new(
                        content.span(),
                        "No shader stages specified for this binding",
                    ));
                }
                let mut binding: Binding = content.parse()?;
                if binding.binding == 0 {
                    // If the binding number wasn't specified, use the automatically tracked number
                    binding.binding = current_binding;
                } else {
                    // If the binding number was specified, update the tracked binding number
                    current_binding = binding.binding;
                }
                binding.stages = current_shader_stages;
                bindings.push(binding);
                if content.is_empty() {
                    break;
                }
                let _comma: syn::Token![,] = content.parse()?;
                current_binding += 1;
            }
            Ok(SetLayout {
                bindings,
                name: name.to_string(),
                set: 0,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl Parse for PlayoutModule {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut module = PlayoutModule::default();
        let mut current_set_id: u32 = 0;
        loop {
            if input.is_empty() {
                break;
            }

            let mut is_descriptor_set = None;
            let mut is_push_constants = false;
            if input.peek(syn::Token![#]) {
                let _pound: syn::Token![#] = input.parse()?;
                let content;
                let _bracket: syn::token::Bracket = syn::bracketed!(content in input);
                let ident = content.parse::<syn::Ident>()?;
                if ident == "set" {
                    if content.peek(syn::Token![=]) {
                        let _eq: syn::Token![=] = content.parse()?;
                        current_set_id = content.parse::<syn::LitInt>()?.base10_parse()?;
                    }
                    is_descriptor_set = Some(current_set_id);
                    current_set_id += 1;
                } else if ident == "push_constants" {
                    is_push_constants = true;
                } else {
                    return Err(syn::Error::new(ident.span(), "unknown attribute"));
                }
            }
            if input.is_empty() {
                break;
            }
            if input.peek(syn::Token![struct]) {
                if let Some(set_id) = is_descriptor_set {
                    let mut set_layout = input.parse::<SetLayout>()?;
                    set_layout.set = set_id;
                    module.descriptor_sets.push(set_layout);
                } else if is_push_constants {
                    module.push_constants = input.parse()?;
                } else {
                    let data_struct = input.parse::<DataStruct>()?;
                    module
                        .data_structs
                        .insert(data_struct.ident.clone(), data_struct);
                }
            }
        }
        Ok(module)
    }
}

impl Parse for PrimitiveTypeSingle {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<syn::Ident>()?;
        match ident.to_string().as_str() {
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            "i8" => Ok(Self::I8),
            "i16" => Ok(Self::I16),
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            "bool" => Ok(Self::Bool),
            "f16" => Ok(Self::F16),
            "f32" => Ok(Self::F32),
            "f64" => Ok(Self::F64),
            _ => Err(syn::Error::new(ident.span(), "Invalid primitive type")),
        }
    }
}

impl Parse for PrimitiveType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let old_input = input.fork();

        let ident = input.parse::<syn::Ident>()?;
        let mut result = match ident.to_string().as_str() {
            "Vec4" => Self::Vec {
                ty: PrimitiveTypeSingle::F32,
                length: 4,
            },
            "Vec3" => Self::Vec {
                ty: PrimitiveTypeSingle::F32,
                length: 3,
            },
            "Vec2" => Self::Vec {
                ty: PrimitiveTypeSingle::F32,
                length: 2,
            },
            "UVec4" => Self::Vec {
                ty: PrimitiveTypeSingle::U32,
                length: 4,
            }, // Shorthands
            "UVec3" => Self::Vec {
                ty: PrimitiveTypeSingle::U32,
                length: 3,
            },
            "UVec2" => Self::Vec {
                ty: PrimitiveTypeSingle::U32,
                length: 2,
            },
            "IVec4" => Self::Vec {
                ty: PrimitiveTypeSingle::I32,
                length: 4,
            },
            "IVec3" => Self::Vec {
                ty: PrimitiveTypeSingle::I32,
                length: 3,
            },
            "IVec2" => Self::Vec {
                ty: PrimitiveTypeSingle::I32,
                length: 2,
            },
            "Mat4" => Self::Mat {
                ty: PrimitiveTypeSingle::F32,
                rows: 4,
                columns: 4,
            },
            "Mat3" => Self::Mat {
                ty: PrimitiveTypeSingle::F32,
                rows: 3,
                columns: 3,
            },
            "Mat2" => Self::Mat {
                ty: PrimitiveTypeSingle::F32,
                rows: 2,
                columns: 2,
            },
            _ => Self::Single(old_input.parse()?),
        };
        if input.peek(syn::Token![<]) {
            // Attempt to resolve generic arg
            if let Self::Vec { ty, .. } = &mut result {
                if matches!(ty, PrimitiveTypeSingle::F32) {
                    let _left = input.parse::<syn::Token![<]>()?;
                    *ty = input.parse()?;
                    let _right = input.parse::<syn::Token![>]>()?;
                } else {
                    return Err(syn::Error::new(
                        ident.span(),
                        "Generic type cannot be used on this shorthand type",
                    ));
                }
            } else {
                return Err(syn::Error::new(
                    ident.span(),
                    "Generic type cannot be used on primitive types",
                ));
            }
        }
        Ok(result)
    }
}

impl Parse for Type {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            // Array-like
            let content;
            let _bracket = syn::bracketed!(content in input);
            let ty: Type = content.parse()?;
            if content.peek(syn::Token![;]) {
                let _semicolon: syn::Token![;] = content.parse()?;
                let length = content.parse::<syn::LitInt>()?;
                Ok(Type::Array {
                    ty: Box::new(ty),
                    size: length.base10_parse()?,
                })
            } else {
                Ok(Type::Slice { ty: Box::new(ty) })
            }
        } else if let Ok(ty) = input.fork().parse::<PrimitiveType>() {
            input.parse::<PrimitiveType>()?;
            Ok(Type::Primitive(ty))
        } else {
            let path: syn::Path = input.parse()?;
            Ok(Type::Path(path.get_ident().unwrap().to_string()))
        }
    }
}

impl Parse for Field {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input.parse()?;
        let _colon: syn::Token![:] = input.parse()?;
        let ty: Type = input.parse()?;
        Ok(Self {
            ident: Some(ident.to_string()),
            ty,
        })
    }
}

impl Parse for DataStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let _struct = input.parse::<syn::Token![struct]>()?;
        let name = input.parse::<syn::Ident>()?;
        let lookahead = input.lookahead1();
        if !lookahead.peek(syn::token::Brace) {
            return Err(lookahead.error());
        }
        let content;
        let _paren: syn::token::Brace = syn::braced!(content in input);
        let fields = content.parse_terminated(Field::parse, syn::Token![,])?;
        Ok(Self {
            ident: name.to_string(),
            fields: fields.into_iter().collect(),
        })
    }
}

impl TryFrom<&str> for PlayoutModule {
    type Error = syn::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let tokens = syn::parse_str::<PlayoutModule>(value)?;
        Ok(tokens)
    }
}
