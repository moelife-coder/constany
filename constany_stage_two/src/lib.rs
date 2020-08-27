//! **Please refer to [`constany_stage_one` document](https://docs.rs/constany_stage_one/0.1.0/constany_stage_one/).**
//! This crate is for the second stage of constany build, which will generate the final constant function based on stage one artifact.

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_attribute]
pub fn const_fn(attr: TokenStream, bare_item: TokenStream) -> TokenStream {
    let item: syn::ItemFn = syn::parse(bare_item.clone()).unwrap();
    let name = &item.sig.ident;
    let visibility = &item.vis;
    let data = std::fs::read(&format!("target/{}.res", item.sig.ident.to_string())).expect("Unable to load function content resource. Please make sure you have executed --stage-one before compiling the final product.");
    use std::convert::TryInto;
    if u64::from_be_bytes(
        data[1..9]
            .try_into()
            .expect("Broken resource file. Please execute stage one again."),
    ) != seahash::hash(bare_item.to_string().as_bytes())
    {
        panic!("Incorrect function hash. Please make sure you have executed --stage-one before compiling the final product.")
    };
    let return_type = &item.sig.output;
    let real_data = String::from_utf8(data[9..].to_vec()).unwrap();
    let const_value = if attr.to_string().contains("force_const") {
        true
    } else {
        false
    };
    let constructed = match data[0] {
        0 => {
            // A super hacky method to generate useable result.
            if const_value {
                let const_name = quote::format_ident!("CONST_VALUE_OF_FN_{}", name);
                let output_type_2 = match return_type {
                    syn::ReturnType::Default => unimplemented!(),
                    syn::ReturnType::Type(_, j) => j,
                };
                let generated = quote! {
                    const #const_name: #output_type_2 = DONOTTOUCHME;
                }
                .to_string();
                let generated = generated.replace("DONOTTOUCHME", &real_data);
                let generated: syn::Item = syn::parse_str(&generated).unwrap();
                quote! {
                    #generated
                    #visibility const fn #name() #return_type {
                        return #const_name
                    }
                }
            } else {
                let generated = quote! {
                    #visibility const fn #name() #return_type {
                        return DONOTTOUCHME
                    }
                }
                .to_string();
                let generated = generated.replace("DONOTTOUCHME", &real_data);
                let generated: syn::ItemFn = syn::parse_str(&generated).unwrap();
                quote! {
                    #generated
                }
            }
        }
        1 => {
            let byte_count = real_data.matches(',').count() + 1;
            let output_type_2 = match return_type {
                syn::ReturnType::Default => unimplemented!(),
                syn::ReturnType::Type(_, j) => j,
            };
            let (advanced_generated, generated) = if const_value {
                let const_name = quote::format_ident!("CONST_VALUE_OF_FN_{}", name);
                (
                    Some(quote! {
                        const #const_name : [u8; THISISTHEBYTECOUNT] = THISISTHEVALUE;
                    }),
                    quote! {
                        #visibility fn #name() #return_type {
                            unsafe {
                                std::mem::transmute::<[u8; THISISTHEBYTECOUNT], #output_type_2>(#const_name)
                            }
                        }
                    },
                )
            } else {
                (
                    None,
                    quote! {
                        #visibility fn #name() #return_type {
                            let constant_value = THISISTHEVALUE;
                            unsafe {
                                std::mem::transmute::<[u8; THISISTHEBYTECOUNT], #output_type_2>(constant_value)
                            }
                        }
                    },
                )
            };
            let generated = generated.to_string();
            let advanced_generated = if let Some(i) = advanced_generated {
                let constructed = i
                    .to_string()
                    .replace("THISISTHEBYTECOUNT", &byte_count.to_string())
                    .replace("THISISTHEVALUE", &real_data);
                let generated: syn::Item = syn::parse_str(&constructed).unwrap();
                quote! {
                    #generated
                }
            } else {
                quote! {}
            };
            let generated = generated
                .replace("THISISTHEBYTECOUNT", &byte_count.to_string())
                .replace("THISISTHEVALUE", &real_data);
            let generated: syn::ItemFn = syn::parse_str(&generated).unwrap();
            quote! {
                #advanced_generated
                #generated
            }
        }
        _ => unimplemented!(),
    };
    return constructed.into();
}
#[proc_macro_attribute]
pub fn main_fn(_: TokenStream, item: TokenStream) -> TokenStream {
    let item: syn::ItemFn = syn::parse(item).unwrap();
    let generated = quote! {
        #item
    };
    generated.into()
}
