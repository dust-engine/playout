use playout::{Binding, DescriptorType, PlayoutModule, SetLayout, ShaderStages};
use proc_macro2::{Span, TokenStream};
use quote::quote;

pub fn push_constant_layout_to_vk(module: &PlayoutModule) -> TokenStream {
    let mut ranges: Vec<(ShaderStages, u32, u32)> = Vec::new(); // stage, start, size
    let mut current_stage_flags = ShaderStages::empty();
    for field in module.push_constants.fields.iter() {
        if field.stages != current_stage_flags {
            let start = ranges.last().map(|a| a.1 + a.2).unwrap_or(0);
            ranges.push((field.stages, start, 0));
            current_stage_flags = field.stages;
        } else {
            let last = ranges.last_mut().unwrap();
            let size = field.field.ty.layout(module).pad_to_align().size();
            last.2 += size as u32; // TODO: should be sizeof(T)
        }
    }

    let ranges = ranges.iter().map(|(stage, start, size)| {
        let stage = stage_flag_to_vk(stage);
        quote! {
            vk::PushConstantRange {
                stage_flags: #stage,
                offset: #start,
                size: #size,
            }
        }
    });

    quote! {[
        #(#ranges),*
    ]}
}

pub fn set_layout_to_vk(layout: &SetLayout) -> TokenStream {
    let bindings = layout.bindings.iter().map(binding_to_vk);
    quote! {[
        #(#bindings),*
    ]}
}

// VkDescriptorSetLayoutBinding
fn binding_to_vk(binding: &Binding) -> TokenStream {
    let binding_num = binding.binding;
    let count_num = binding.descriptor_count;
    let descriptor_type = descriptor_type_to_vk(&binding.descriptor_type);
    let shader_stage_flags = stage_flag_to_vk(&binding.stages);
    quote! {
        vk::DescriptorSetLayoutBinding {
            binding: #binding_num,
            descriptor_type: #descriptor_type,
            descriptor_count: #count_num,
            stage_flags: #shader_stage_flags,
            p_immutable_samplers: ::std::ptr::null(),
        }
    }
}

fn stage_flag_to_vk(stage_flag: &ShaderStages) -> TokenStream {
    let names = stage_flag
        .iter_names()
        .map(|a| syn::Ident::new(a.0, Span::call_site()));
    quote! {
        #(vk::ShaderStageFlags::#names)|*
    }
}

pub(crate) fn descriptor_type_to_vk(descriptor_type: &DescriptorType) -> TokenStream {
    match descriptor_type {
        DescriptorType::Sampler => quote! {
            vk::DescriptorType::SAMPLER
        },
        DescriptorType::StorageImage { .. } => quote! {
            vk::DescriptorType::STORAGE_IMAGE
        },
        DescriptorType::SampledImage => quote! {
            vk::DescriptorType::SAMPLED_IMAGE
        },
        DescriptorType::UniformBuffer { .. } => quote! {
            vk::DescriptorType::UNIFORM_BUFFER
        },
        DescriptorType::StorageBuffer { .. } => quote! {
            vk::DescriptorType::STORAGE_BUFFER
        },
        DescriptorType::AccelerationStructure => quote! {
            vk::DescriptorType::ACCELERATION_STRUCTURE_KHR
        },
    }
}
