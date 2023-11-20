#![feature(proc_macro_span)]
#![feature(alloc_layout_extra)]

#[cfg(feature = "vulkan")]
mod vk;
#[cfg(feature = "vulkan")]
mod write;

use playout::PlayoutModule;
use quote::quote;

#[proc_macro]
pub fn layout(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut input = input.into_iter();
    let Some(token) = input.next() else {
        return quote! {
            compile_error!("Expects path to .playout file")
        }
        .into();
    };

    match input.next() {
        Some(proc_macro::TokenTree::Punct(punct)) if punct.as_char() == ',' => (),
        _ => {
            return quote! {
                compile_error!("Expects comma")
            }
            .into()
        }
    }

    let set_id = match input.next() {
        Some(proc_macro::TokenTree::Literal(lit)) => {
            let lit = lit.to_string();
            if let Ok(set_id) = lit.to_string().parse::<u32>() {
                Some(set_id)
            } else {
                return quote! {
                    compile_error!("Expects integer literal for set id")
                }
                .into();
            }
        }
        Some(proc_macro::TokenTree::Ident(ident)) if ident.to_string() == "\"push\"" => None,
        _ => {
            return quote! {
                compile_error!("Expects set id or push")
            }
            .into()
        }
    };

    if input.next().is_some() {
        return quote! {
            compile_error!("Expects exactly one string literal as input")
        }
        .into();
    }
    let path = token.to_string();
    let path = path.strip_prefix('"').unwrap().strip_suffix('"').unwrap();
    let path = token.span().source_file().path().parent().unwrap().join(path);
    let file = match std::fs::read_to_string(path) {
        Ok(file) => file,
        Err(err) => {
            let message = err.to_string();
            return quote! {
                compile_error!(#message)
            }
            .into();
        }
    };
    let module = match PlayoutModule::try_from(file.as_str()) {
        Ok(module) => module,
        Err(err) => {
            let message = err.to_string();
            return quote! {
                compile_error!(#message)
            }
            .into();
        }
    };

    if let Some(set_id) = set_id {
        let Some(set) = module.descriptor_sets.iter().find(|set| set.set == set_id) else {
            let missing_id = format!("Set id {set_id} does not exist within this playout file");
            return quote! {
                compile_error!(#missing_id)
            }
            .into();
        };
        vk::set_layout_to_vk(&module, set).into()
    } else {
        vk::push_constant_layout_to_vk(&module).into()
    }
}

#[proc_macro]
pub fn write(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(tokens as write::DescriptorSetWriteArgs);
    input.into_vk().into()
}
