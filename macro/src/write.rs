use std::collections::{BTreeMap, BTreeSet};

use playout::{Binding, PlayoutModule};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse::Parse, punctuated::Punctuated, spanned::Spanned};
pub struct DescriptorSetWriteArgs {
    pub playout_path: syn::LitStr,
    pub brace: syn::token::Brace,
    pub updates: syn::punctuated::Punctuated<DescriptorSetWriteUpdate, syn::Token![,]>,
}

pub struct DescriptorSetWriteUpdate {
    pub dst: syn::Expr,
    pub name: syn::Ident,
    pub fields: syn::punctuated::Punctuated<DescriptorSetWriteField, syn::Token![,]>,
}

impl Parse for DescriptorSetWriteUpdate {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let dst: syn::Expr = input.parse()?;
        let _colon = input.parse::<syn::Token![:]>()?;
        let name: syn::Ident = input.parse()?;
        let lookahead = input.lookahead1();
        if !lookahead.peek(syn::token::Brace) {
            return Err(lookahead.error());
        }
        let content;
        let _brace = syn::braced!(content in input);
        let fields = content.parse_terminated(DescriptorSetWriteField::parse, syn::Token![,])?;
        Ok(Self { name, fields, dst })
    }
}

impl Parse for DescriptorSetWriteArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path: syn::LitStr = input.parse()?;
        let _comma: syn::Token![,] = input.parse()?;
        let lookahead = input.lookahead1();
        if !lookahead.peek(syn::token::Brace) {
            return Err(lookahead.error());
        }
        let content;
        let brace = syn::braced!(content in input);

        Ok(Self {
            playout_path: path,
            brace,
            updates: content.parse_terminated(DescriptorSetWriteUpdate::parse, syn::Token![,])?,
        })
    }
}
pub struct DescriptorSetWriteField {
    pub name: syn::Ident,
    pub subscript: Option<syn::LitInt>,
    /// VkDescriptorImageInfo, VkDescriptorBufferInfo or VkBufferView
    pub values: Punctuated<syn::Expr, syn::Token![,]>,
}

impl Parse for DescriptorSetWriteField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name: syn::Ident = input.parse()?;
        let lookahead = input.lookahead1();
        let subscript = if lookahead.peek(syn::token::Bracket) {
            let content;
            let _bracket = syn::bracketed!(content in input);
            let subscript: syn::LitInt = content.parse()?;
            Some(subscript)
        } else {
            None
        };
        let _colon: syn::Token![:] = input.parse()?;

        let values: Punctuated<syn::Expr, syn::Token![,]> = if input.peek(syn::token::Bracket) {
            let content;
            let _bracket = syn::bracketed!(content in input);
            content.parse_terminated(syn::Expr::parse, syn::Token![,])?
        } else {
            let value: syn::Expr = input.parse()?;
            Punctuated::from_iter([value])
        };
        Ok(Self {
            name,
            values,
            subscript,
        })
    }
}

impl DescriptorSetWriteUpdate {
    fn into_vk(
        mut self,
        module: &PlayoutModule,
        ctx: &mut DescriptorSetWriteCtx,
    ) -> Result<Vec<TokenStream>, (Span, String)> {
        let Some(set) = module
            .descriptor_sets
            .iter()
            .find(|set| set.name == self.name.to_string())
        else {
            let avilable_sets = module
                .descriptor_sets
                .iter()
                .map(|set| set.name.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let message = format!(
                "Descriptor set not found in playout file. Available descriptor sets: {}",
                avilable_sets
            );
            return Err((self.name.span(), message));
        };
        let map_name_to_binding_id = set
            .bindings
            .iter()
            .map(|binding| {
                let name = binding.ident.to_string();
                (name, binding)
            })
            .collect::<BTreeMap<_, _>>();
        let mut fields: Vec<(&mut DescriptorSetWriteField, &Binding, u32)> =
            Vec::with_capacity(self.fields.len());

        let mut bindings_seen: BTreeSet<(u32, u32)> = BTreeSet::new(); // Set of (binding, array)
        for field in self.fields.iter_mut() {
            let Some(binding) = map_name_to_binding_id.get(&field.name.to_string()).cloned() else {
                return Err((field.name.span(), "Binding not found".to_string()));
            };
            let subscript = if let Some(subscript_lit) = &field.subscript {
                let Ok(subscript) = subscript_lit.base10_parse::<u32>() else {
                    return Err((
                        field.name.span(),
                        "Expects integer literal for subscript".to_string(),
                    ));
                };
                if subscript >= binding.descriptor_count {
                    let message = format!(
                        "Subscript exceeds descriptor array length {}",
                        binding.descriptor_count
                    );
                    return Err((field.name.span(), message));
                }
                if binding.descriptor_count == 1 {
                    let message = "Not an array descriptor".to_string();
                    return Err((subscript_lit.span(), message));
                }
                subscript
            } else {
                if field.values.len() != binding.descriptor_count as usize {
                    let message = format!(
                        "Expected array literal of length {}",
                        binding.descriptor_count
                    );
                    return Err((field.values.span(), message));
                }
                0
            };

            if bindings_seen.contains(&(binding.binding, subscript)) {
                let message = if field.subscript.is_none() {
                    format!("Binding {} is already written", field.name,)
                } else {
                    format!("Binding {}[{}] is already written", field.name, subscript)
                };
                return Err((field.name.span(), message));
            }
            if field.subscript.is_some() {
                bindings_seen.insert((binding.binding, subscript));
                bindings_seen.insert((binding.binding, 0));
            } else {
                for i in 0..binding.descriptor_count {
                    bindings_seen.insert((binding.binding, i));
                }
            }
            fields.push((field, binding, subscript));
        }
        fields.sort_by_key(|(_, binding, subscript)| (binding.binding, *subscript));

        use itertools::Itertools;
        let token_streams = fields
            .iter_mut()
            .map(|(field, binding, subscript)| {
                let field: &mut DescriptorSetWriteField = field;
                let values = std::mem::take(&mut field.values);
                (
                    field,                    // field
                    *binding,                 // start binding
                    *subscript,               // start array element
                    binding.descriptor_count, // count in current binding
                    values,
                )
            })
            .coalesce(|mut prev, curr| {
                if !prev.1.descriptor_type.same_type_as(&curr.1.descriptor_type) {
                    //
                    return Err((prev, curr));
                }
                if prev.1.stages != curr.1.stages {
                    // Consecutive bindings must have identical VkShaderStageFlags
                    return Err((prev, curr));
                }
                if prev.1.binding == curr.1.binding {
                    // within the same binding. Require consecutive subscripts.
                    if prev.2 + prev.3 == curr.2 {
                        prev.4.extend(curr.4);
                        Ok((prev.0, prev.1, prev.2, prev.3 + curr.3, prev.4))
                    // merge.
                    } else {
                        Err((prev, curr))
                    }
                } else {
                    // different binding.
                    // Two bindings must be consecutive. (Assuming each binding has at least 1 descriptor)
                    if prev.1.binding + 1 != curr.1.binding {
                        return Err((prev, curr));
                    }
                    // prev binding must be fully written.
                    if (prev.2 + prev.3) < prev.1.descriptor_count {
                        return Err((prev, curr));
                    }
                    // curr binding must be the first element.
                    if curr.2 != 0 {
                        return Err((prev, curr));
                    }
                    prev.4.extend(curr.4);
                    Ok((prev.0, prev.1, prev.2, curr.3, prev.4))
                    // merge.
                }
            })
            .map(|(_field, binding, subscript, _, descs)| {
                let dst = self.dst.to_token_stream();
                let count = descs.len() as u32;
                let descriptor_type = crate::vk::descriptor_type_to_vk(&binding.descriptor_type);
                let ptr_quote = match &binding.descriptor_type {
                    playout::DescriptorType::Sampler
                    | playout::DescriptorType::StorageImage { .. }
                    | playout::DescriptorType::SampledImage => {
                        let index = ctx.img_info.len();
                        ctx.img_info.extend(descs);
                        quote! {
                            p_image_info: unsafe{img_info.as_ptr().add(#index)}
                        }
                    }
                    playout::DescriptorType::UniformBuffer { ty: _ }
                    | playout::DescriptorType::StorageBuffer { ty: _ } => {
                        let index = ctx.buffer_info.len();
                        ctx.buffer_info.extend(descs);
                        quote! {
                            p_buffer_info: unsafe{buffer_info.as_ptr().add(#index)}
                        }
                    }
                    playout::DescriptorType::AccelerationStructure => todo!(),
                };
                quote::quote! {
                    vk::WriteDescriptorSet {
                        dst_set: #dst,
                        dst_array_element: #subscript,
                        descriptor_count: #count,
                        descriptor_type: #descriptor_type,
                        #ptr_quote,
                        ..Default::default()
                    }
                }
            })
            .collect();
        Ok(token_streams)
    }
}

impl DescriptorSetWriteArgs {
    pub fn into_vk(self) -> TokenStream {
        if self.updates.iter().map(|a| a.fields.len()).sum::<usize>() == 0 {
            return quote_spanned! { self.brace.span =>
                compile_error!("Expects at least one update")
            };
        }

        let path = self
            .playout_path
            .span()
            .unwrap()
            .source_file()
            .path()
            .join(self.playout_path.value());
        let file = match std::fs::read_to_string(path) {
            Ok(file) => file,
            Err(err) => {
                let message = err.to_string();
                return quote! {
                    compile_error!(#message)
                };
            }
        };
        let module = match PlayoutModule::try_from(file.as_str()) {
            Ok(module) => module,
            Err(err) => {
                let message = err.to_string();
                return quote! {
                    compile_error!(#message)
                };
            }
        };

        let mut token_streams = Vec::new();
        let mut ctx = DescriptorSetWriteCtx::default();
        for update in self.updates.into_iter() {
            let token_stream = match update.into_vk(&module, &mut ctx) {
                Ok(token_stream) => token_stream,
                Err((span, message)) => {
                    return quote_spanned! {span=>
                        compile_error!(#message)
                    };
                }
            };
            token_streams.extend(token_stream.into_iter());
        }

        let num_img_info = ctx.img_info.len();
        let num_buffer_info = ctx.buffer_info.len();
        let num_buffer_view = ctx.buffer_view.len();
        let img_info = ctx.img_info;
        let buffer_info = ctx.buffer_info;
        let buffer_view = ctx.buffer_view;

        quote! {{
            let img_info: [vk::DescriptorImageInfo; #num_img_info] = [#(#img_info),*];
            let buffer_info: [vk::DescriptorBufferInfo; #num_buffer_info] = [#(#buffer_info),*];
            let buffer_view: [vk::BufferView; #num_buffer_view] = [#(#buffer_view),*];
            [
                #(#token_streams),*
            ]
        }}
    }
}

#[derive(Default)]
struct DescriptorSetWriteCtx {
    img_info: Vec<syn::Expr>,
    buffer_info: Vec<syn::Expr>,
    buffer_view: Vec<syn::Expr>,
}
