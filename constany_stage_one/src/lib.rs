//! **Constany is a rust macro to allow constant result for any function.**
//!
//! Constant functions in rust is a group of function with its result will be evaluated during compile time. It can significantly reduce generated binary size and improve performance. However, due to technical and logical limitations, some expression cannot be evaluated as constant function. For example:
//! ```compile_fail
//! fn main() {
//!    println!("{}", add_one_to_six());
//! }
//! const fn add_one_to_six() -> String {
//!    let mut a = 1;
//!    for b in 1..7 { // error[E0744]: `for` is not allowed in a `const fn`
//!        a += b;
//!    }
//!    return a.to_string();
//! }
//! ```
//!
//! Constany use a workaround for this: by using Constany, the function and the main function will be compiled twice. The value of the function will be evaluated at the first time, and the value will be wrapped into a constant function at the second time.
//!
//! `Cargo.toml:`
//! ```toml
//! [features]
//! stage_one = ["constany_stage_one"]
//! stage_two = ["constany_stage_two"]
//! [dependencies]
//! constany_stage_one = {version = "0.1", optional = true}
//! constany_stage_two = {version = "0.1", optional = true}
//! constany_blank = {version = "1"}
//! ```
//!
//! `main.rs:`
//! ```ignore
//! #[cfg(any(
//!     not(any(feature = "stage_one", feature = "stage_two")),
//!     all(feature = "stage_two", feature = "stage_one")
//! ))]
//! use constany_blank as constany; // This line is for grammar checkers that enable all feature / disable all feature. If you do not have a checker, you can delete those lines safely.
//! #[cfg(all(feature = "stage_one", not(feature = "stage_two")))]
//! use constany_stage_one as constany;
//! #[cfg(all(feature = "stage_two", not(feature = "stage_one")))]
//! use constany_stage_two as constany;
//! #[constany::main_fn("function_evaled_at_compile_time")]
//! fn main() {
//!     println!("Hello, world!");
//!     function_evaled_at_compile_time();
//! }
//! #[constany::const_fn]
//! fn function_evaled_at_compile_time() -> i32 {
//!     let mut a = 1;
//!     let b = 5;
//!     for _ in 0..b {
//!         a += 1; // For loop is not allowed in `const fn`
//!     }
//!     a
//! }
//! ```
//! When you need to build the function, do this:
//! ```bash
//! $ cargo run --featues stage_one
//! $ cargo build --features stage_two // If you want to run the command instead, use `cargo run`
//! ```
//! And your function will be interpreted as constant function.
//!
//! ## Multiple constant function
//! Having multiple constant functions are also applicable, you just need to make sure every function you want to make constant are labeled with `const_fn` and the function name is inside `main_fn`:
//! ```ignore
//! #[cfg(any(
//!     not(any(feature = "stage_one", feature = "stage_two")),
//!     all(feature = "stage_two", feature = "stage_one")
//! ))]
//! use constany_blank as constany;
//! #[cfg(all(feature = "stage_one", not(feature = "stage_two")))]
//! use constany_stage_one as constany;
//! #[cfg(all(feature = "stage_two", not(feature = "stage_one")))]
//! use constany_stage_two as constany;
//! #[constany::main_fn("function_evaled_at_compile_time", "function_evaled_at_compile_time")]
//! fn main() {
//!     println!("Hello, world!");
//!     function_evaled_at_compile_time();
//!     function_evaled_at_compile_time_2();
//! }
//! #[constany::const_fn]
//! fn function_evaled_at_compile_time() -> i32 {
//!     let mut a = 1;
//!     let b = 5;
//!     for _ in 0..b {
//!         a += 1;
//!     }
//!     a
//! }
//! #[constany::const_fn]
//! fn function_evaled_at_compile_time_2() -> i32 {
//!     let mut a = 1;
//!     let b = 100;
//!     for _ in 0..b {
//!         a += 1;
//!     }
//!     a
//! }
//! ```
//!
//! ## Function with non-primitive result
//! Returning a non-primitive result (probably `struct` or `enum`) is troublesome and prone to error. The most elegant way is to use `lazy_static` for stage one and default to avoid compiler warning, and use constant value function for stage two:
//! ```compile_fail
//! #[cfg(feature = "stage_two")]
//! const ABC: String = constant_function().to_string();
//! #[cfg(not(feature = "stage_two"))]
//! lazy_static::lazy_static! {
//!     const ref ABC: String = constant_function().to_string();
//! }
//! ```
//! However, this will not work for most of the non-primitive type because their constructor is usually not `static`.
//!
//! There are two workaround for this: the `debug + pub` solution and `memop` solution.
//!
//! The `debug + pub` solution first use `debug` trait to print the structure, and use the `pub` trait to rebuild it.
//! This solution can recreate the structure without `unsafe` code. However, this require the structure to derive `Debug`.
//! Current implementation also require the structure to not have `paths`, such as `std::string::String` (if there are `::` in the identifier, it's likely that this solution will not work out).
//!
//! The `memop` solution transmute the memory directly.
//! This solution can rebuild any structure, but please note that this method is `unsafe` and very dangerous.
//! The generated function will be `fn` instead of `const_fn` because memory allocation is not allowed in `const`, although the memory itself is hard-coded inside the function.

extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn;

fn is_primitive_type(input: &syn::Type) -> bool {
    match input {
        syn::Type::Path(i) => {
            if i.path.leading_colon.is_none() && i.path.segments.len() == 1 {
                match i.path.segments[0].ident.to_string().as_str() {
                    "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32"
                    | "u64" | "u128" | "usize" | "f32" | "f64" | "bool" | "char" | "str" => true,
                    _ => false,
                }
            } else {
                false
            }
        }
        syn::Type::Array(i) => is_primitive_type(&i.elem),
        syn::Type::Group(i) => is_primitive_type(&i.elem),
        syn::Type::Slice(i) => is_primitive_type(&i.elem),
        syn::Type::Tuple(i) => {
            for i in &i.elems {
                if !is_primitive_type(&i) {
                    return false;
                }
            }
            return true;
        }
        _ => false,
    }
}

/// Generate a constant function
#[proc_macro_attribute]
pub fn const_fn(attr: TokenStream, bare_item: TokenStream) -> TokenStream {
    let item: syn::ItemFn = syn::parse(bare_item.clone()).unwrap();
    let name = &item.sig.ident;
    let visibility = &item.vis;
    let output_type = &item.sig.output;
    let wrapper_fn_name = quote::format_ident!("_{}_wrapper_fn", name.to_string());
    let (generation_method, fbyte) = if match output_type {
        syn::ReturnType::Default => {
            return syn::Error::new_spanned(
                output_type,
                "Fn with `()` output should not become constant",
            )
            .to_compile_error()
            .into()
        }
        syn::ReturnType::Type(_, i) => is_primitive_type(&i),
    } {
        (
            quote! {
                format!("{:?}", #name())
            },
            0u8,
        )
    } else {
        let a = quote::format_ident!("{}", name.to_string());
        if attr.to_string().contains("memop") {
            let output_type = match output_type {
                syn::ReturnType::Default => unimplemented!(),
                syn::ReturnType::Type(_, j) => j,
            };
            (
                quote! {
                    format!("{:?}", unsafe {
                        std::mem::transmute::<#output_type, [u8; std::mem::size_of::<#output_type>()]>(#a())
                    })
                },
                1,
            )
        } else {
            // This should be changed.
            (
                quote! {
                    format!("{:?}", #name())
                },
                0,
            )
        }
    };
    let hash_file_name = format!("target/{}.hash", name.to_string());
    let code_identifier = bare_item.to_string();
    let generated = quote! {
        #item
        /// Wrapper fn for previous function
        #visibility fn #wrapper_fn_name() -> (String, u8) {
            // Write identifier to target
            std::fs::write(#hash_file_name, #code_identifier).unwrap();
            (#generation_method, #fbyte)
        }
    };
    generated.into()
}

/// Attribute appending on `fn main()`
///
/// When generating a constant function, you need to include it in the attribute: eg. `#[main_fn(a_constant_function, another_constant_function)]`
#[proc_macro_attribute]
pub fn main_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    drop(item);
    let mut attr_vec: Vec<String> = Vec::new();
    for i in attr {
        let i: proc_macro::TokenStream = i.into();
        attr_vec.push(i.to_string())
    }
    let mut fn_vec = Vec::new();
    for i in attr_vec {
        if i != "," {
            let fn_name = &i[1..i.len() - 1];
            let wrapper_fn_name = quote::format_ident!("_{}_wrapper_fn", fn_name.to_string());
            fn_vec.push((wrapper_fn_name, format!("target/{}.result", fn_name)));
        }
    }
    let mut generated = quote! {};
    for i in fn_vec {
        let (i, j) = i;
        generated = quote! {
            #generated
            let (j, i) = #i();
            let mut constructed = vec![i];
            constructed.extend_from_slice(&j.into_bytes());
            std::fs::write(#j, constructed).unwrap();
        }
    }
    //let item: syn::ItemFn = syn::parse(item).unwrap();
    let generated = quote! {
        fn main() {
            #generated
        }
    };
    generated.into()
}
