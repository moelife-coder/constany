//! ## Please refer to `constany_stage_one` document.
//! This crate is for the second stage of constany build, which will generate the final constant function based on stage one artifact.

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_attribute]
pub fn const_fn(_: TokenStream, bare_item: TokenStream) -> TokenStream {
    let item: syn::ItemFn = syn::parse(bare_item.clone()).unwrap();
    let name = &item.sig.ident;
    let visibility = &item.vis;
    if let Ok(i) = std::fs::read_to_string(&format!("target/{}.hash", item.sig.ident.to_string())) {
        if i == bare_item.to_string() {
            let data =
                std::fs::read(&format!("target/{}.result", item.sig.ident.to_string())).unwrap();
            let return_type = &item.sig.output;
            let real_data = String::from_utf8(data[1..].to_vec()).unwrap();
            let constructed = match data[0] {
                0 => {
                    // A super hacky method to generate useable result.
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
                1 => {
                    let byte_count = real_data.matches(',').count() + 1;
                    let output_type_2 = match return_type {
                        syn::ReturnType::Default => unimplemented!(),
                        syn::ReturnType::Type(_, j) => j,
                    };
                    let generated = quote! {
                        #visibility fn #name() #return_type {
                            // This is exteremly dangerous. ONLY USE IT WHEN THERE'S NOTHING ELSE!
                            let constant_value = THISISTHEVALUE;
                            unsafe {
                                std::mem::transmute::<[u8; THISISTHEBYTECOUNT], #output_type_2>(constant_value)
                            }
                        }
                    }.to_string();
                    let generated = generated
                        .replace("THISISTHEBYTECOUNT", &byte_count.to_string())
                        .replace("THISISTHEVALUE", &real_data);
                    let generated: syn::ItemFn = syn::parse_str(&generated).unwrap();
                    quote! {
                        #generated
                    }
                }
                _ => unimplemented!(),
            };
            return constructed.into();
        }
    }
    println!("{}", &format!("target/{}.hash", item.sig.ident.to_string()));
    panic!("Constany stage one file not found or it's obsolete. Please run stage one again.");
}
#[proc_macro_attribute]
pub fn main_fn(_: TokenStream, item: TokenStream) -> TokenStream {
    let item: syn::ItemFn = syn::parse(item).unwrap();
    let generated = quote! {
        #item
    };
    generated.into()
}
