use syn::{
    ext::IdentExt,
    parse::{Parse, ParseStream},
};

use crate::{Binding, DescriptorType, ImageFormat, SetLayout, ShaderStages};

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

fn parse_binding_attribute(input: ParseStream) -> syn::Result<u32> {
    let _pound: syn::Token![#] = input.parse()?;
    let content;
    let _bracket: syn::token::Bracket = syn::bracketed!(content in input);
    let ident = content.parse::<syn::Ident>()?;
    if ident == "binding" {
        let _eq = content.parse::<syn::Token![=]>()?;
        let binding_literal: syn::LitInt = content.parse::<syn::LitInt>()?;
        return binding_literal.base10_parse();
    } else {
        return Err(syn::Error::new_spanned(ident, "unknown attribute"));
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
        return inside.parse::<ShaderStages>();
    } else {
        return Err(syn::Error::new_spanned(ident, "unknown attribute"));
    }
}

impl Parse for Binding {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut binding: u32 = 0;
        loop {
            if !input.peek(syn::Token![#]) {
                break;
            }
            binding = parse_binding_attribute(input)?;
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
        })
    }
}

impl Parse for SetLayout {
    fn parse(input: ParseStream) -> syn::Result<Self> {
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
            Ok(SetLayout { bindings })
        } else {
            Err(lookahead.error())
        }
    }
}

impl TryFrom<&str> for SetLayout {
    type Error = syn::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let tokens = syn::parse_str::<SetLayout>(value)?;
        Ok(tokens)
    }
}
