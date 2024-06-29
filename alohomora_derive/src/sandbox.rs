extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, ToTokens};
use syn::{parse::Parser, DeriveInput, FnArg, Ident, ItemFn, ReturnType};
// use ::alohomora_sandbox::alloc_mem_in_sandbox;

pub fn sandbox_impl(input: ItemFn) -> TokenStream {
    let function_signature = input.sig;
    let function_name = function_signature.ident;

    let invoke_sandbox_function_name_c = format!("invoke_sandbox_{}_c", function_name);
    let invoke_sandbox_function_name_c = Ident::new(&invoke_sandbox_function_name_c, function_name.span());

    let function_name_sandbox = format!("{}_sandbox", function_name);
    let function_name_sandbox = Ident::new(&function_name_sandbox, function_name.span());

    let function_name_sandbox_free = format!("{}_sandbox_free", function_name);
    let function_name_sandbox_free = Ident::new(&function_name_sandbox_free, function_name.span());

    // Reject illegal functions.
    if function_signature.asyncness.is_some() {
        return quote_spanned!(function_name.span() => compile_error!("Sandbox function cannot be async"));
    }
    if function_signature.unsafety.is_some() {
        return quote_spanned!(function_name.span() => compile_error!("Sandbox function cannot be unsafe"));
    }
    if function_signature.abi.is_some() {
        return quote_spanned!(function_name.span() => compile_error!("Sandbox function cannot be abi"));
    }
    if function_signature.generics.lt_token.is_some() {
        return quote_spanned!(function_name.span() => compile_error!("Sandbox function cannot have generics"));
    }
    if function_signature.variadic.is_some() {
        return quote_spanned!(function_name.span() => compile_error!("Sandbox function cannot be variadic"));
    }

    // Find arguments and return types.
    let params: Vec<_> = function_signature.inputs.iter().collect();
    if params.len() != 1 {
        return quote_spanned!(function_name.span() => compile_error!("Sandbox function must have exactly one Serializable argument"));
    }
    let arg = match params[0] {
        FnArg::Receiver(_) => {
            return quote_spanned!(function_name.span() => compile_error!("Sandbox function should not take self"));
        },
        FnArg::Typed(ty) => *ty.ty.clone(),
    };
    let ret = match function_signature.output {
        ReturnType::Default => {
            return quote_spanned!(function_name.span() => compile_error!("Sandbox function must return a Serializable type"));
        },
        ReturnType::Type(_, ty) => *ty.clone(),
    };

    quote! {
        #[cfg(not(target_arch = "wasm32"))]
        extern "C" {
            pub fn #invoke_sandbox_function_name_c(arg: *mut std::ffi::c_void, size: u32) -> ::alohomora_sandbox::sandbox_out;
        }

        // This should be generated by a macro.
        #[cfg(not(target_arch = "wasm32"))]
        #[allow(non_camel_case_types)]
        pub struct #function_name {}

        #[cfg(not(target_arch = "wasm32"))]
        #[doc = "Library implementation of AlohomoraSandbox. Do not copy this docstring!"]
        impl<'__a, '__b> alohomora_sandbox::AlohomoraSandbox<'__a, '__b, #arg, #ret> for #function_name {
            fn invoke(arg: #arg) -> ::alohomora_sandbox::FinalSandboxOut<#ret> {
                alohomora_sandbox::invoke_sandbox!(#invoke_sandbox_function_name_c, arg);
            }
        }

        // This should also be generated by macro.
        #[no_mangle]
        #[cfg(target_arch = "wasm32")]
        pub extern "C" fn #function_name_sandbox(arg: *mut std::ffi::c_void, size: u32) -> *mut u8{
            alohomora_sandbox::sandbox_preamble(#function_name, arg, size)
        }

        #[no_mangle]
        #[cfg(target_arch = "wasm32")]
        pub extern "C" fn #function_name_sandbox_free(ret: *mut ::std::os::raw::c_char) {
            use std::ffi::CString;
            let _ = unsafe { CString::from_raw(ret) };
        }
    }
}

pub type Error = (Span, &'static str);

pub fn derive_swizzleable_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let mut stream = TokenStream::new();

    // 1. define unswizzled type
    // 2. implement Swizzleable trait
    // 2a. define Unswizzled as unswizzled type from (1)
    // 2b. define unswizzle function
    //      

    let struct_data = match input.data.clone() {
        syn::Data::Struct(s) => s,
        _ => return Err((input.ident.span(), "derive(Swizzleable) only works on structs")),
    };

    let fields = match struct_data.fields.clone() {
        syn::Fields::Named(f) => f,
        _ => return Err((input.ident.span(), "no named fields??")),
    };


    let struct_name = input.ident.clone();
    let struct_generics = input.generics.clone().to_token_stream();
    let struct_generic_names = input.generics.clone().params.into_iter().filter_map(|gp|{
            match gp {
                syn::GenericParam::Type(tp) => Some(format!("{}", tp.ident)),
                _ => None,
            }
    }).collect::<Vec<String>>();

    // panic!("{:?}", struct_data.fields);
    // set up the fields of the unswizzled version of this struct
    let mut unswizzled_fields = TokenStream::new();
    let mut unswizzle_function = TokenStream::new();
    let (outside_name, inside_name) = (Ident::new("outside", Span::call_site()), 
                                                     Ident::new("inside", Span::call_site()));
    let mut need_new_struct = false; // TODO: this should also be true if it contains usizes


    for field in fields.named.iter() {
        let field_name = field.ident.clone().unwrap();
        // if it's a pointer
        if let syn::Type::Ptr(ptr) = &field.ty {
            let p = match *ptr.elem.clone() {
                syn::Type::Path(p) => p,
                _ => return Err((input.ident.span(), "no named fields??")),
            };

            // if this field is a ptr, we make it a sandbox ptr in the new struct
            let ptr_to_type = &p.path.segments.first().unwrap().ident;

            let points_to_generic = struct_generic_names.contains(&format!("{}", ptr_to_type));
            println!("type {:?} has point to generic {}", ptr_to_type, points_to_generic);

            if !points_to_generic {
                // This is lowk the dumbest possible way to do this
                let new_field = syn::Field::parse_named.parse2(
                    quote! { 
                        pub #field_name: SandboxPointer<<#ptr_to_type as Swizzleable>::Unswizzled> 
                    }
                ).unwrap();
                new_field.to_tokens(&mut unswizzled_fields);
                unswizzled_fields.extend(quote!{,});            // add back comma bc we remove it when just getting the field

                // and then recursively unswizzle it into the sandbox
                let deep_convert_line = quote!{
                    (*#inside_name).#field_name = 
                        unswizzle_ptr(
                        // ^^ 3. unswizzle that pointer (to also be in the sandbox)
                            Swizzleable::unswizzle((*#outside_name).#field_name, 
                            // ^^ 2. unswizzle that whole data structure (to be in the sandbox)
                                swizzle_ptr(&(*#inside_name).#field_name, #inside_name)));
                                // ^^ 1. swizzle the inside pointer to be global
                };
                unswizzle_function.extend(deep_convert_line);
            } else {
                // TODO: this is really a bad idea to treat generics seperately,
                //          we should instead constrain them to only be swizzleable types
                // the field should just be a sandbox pointer to the generic type
                unswizzled_fields.extend(quote!{
                    pub #field_name: SandboxPointer<#ptr_to_type>,
                });

                // the copying line should just handle swizzling ptrs (not the generic)
                let deep_convert_line = quote!{
                    // TODO: THIS WHOLE LINE IS REDUNDANT, we should be copying its value instead
                    (*#inside_name).#field_name = 
                        unswizzle_ptr(
                        // ^^ 3. unswizzle that pointer (to also be in the sandbox)
                            swizzle_ptr(&(*#inside_name).#field_name, #inside_name));
                            // ^^ 1. swizzle the inside pointer to be global
                };
                unswizzle_function.extend(deep_convert_line);
            }
            
            need_new_struct = true;
        } else if let syn::Type::Path(path) = &field.ty {
            let type_ident = path.path.segments.first().unwrap().ident.clone();
            let type_str = format!("{}", &type_ident);
            if !is_primitive(path){
                // it's a struct
                let new_struct_name = format!("{type_str}Unswizzled");
                let (new_field, new_struct_ident) = field_with_path_name(field.clone(), new_struct_name.as_str(), &mut need_new_struct);
                // add new field to fields
                unswizzled_fields.extend(quote!{#new_field,});

                // and simply copy it to unswizzle
                unswizzle_function.extend(quote!{
                    Swizzleable::unswizzle(&mut (*#outside_name).#field_name as *mut #type_ident #struct_generics, &mut (*#inside_name).#field_name as *mut #new_struct_ident #struct_generics);
                });
            } else {
                // it's a primitive
                let (new_field, new_type) = match type_str.as_str() {
                    // if it's a usize, convert to u32 (for 32bit ptrs)
                    "usize" => field_with_path_name(field.clone(), "u32", &mut need_new_struct),
                    // if it's a isize, convert to i32 (for 32bit ptrs)
                    "isize" => field_with_path_name(field.clone(), "i32", &mut need_new_struct),
                    // if it's another primitive, keep it the same
                    primitive_name => (field.clone().to_token_stream(), Ident::new(primitive_name, Span::call_site())),
                };

                // add new field to fields
                unswizzled_fields.extend(quote!{#new_field,});

                // and simply copy it to unswizzle
                unswizzle_function.extend(quote!{
                    (*#inside_name).#field_name = (*#outside_name).#field_name as #new_type;
                });
            }
        } else {
            // TODO: if its a path check for usize and switch it to u64
            // then use `as u64` inconversion when unswizzling
            // and set need_new_struct

            // FIXME: if it's a struct we have to recursively unswizzle the struct

            // TODO: deal with references too

            // if not we keep as is in the new struct
            // (*field).to_tokens(&mut unswizzled_fields);
            unswizzled_fields.extend(quote!{#field,});                // add back comma bc we remove it when just getting the field

            // and simply copy it to unswizzle
            let function_copy_line = quote!{
                (*#inside_name).#field_name = (*#outside_name).#field_name;
            };
            unswizzle_function.extend(function_copy_line);
        }
    }

    
    let (unswizzled_struct_name, unswizzled_struct_generics): (Ident, TokenStream) = if need_new_struct {
        // if the struct has pointers, we have to make an unswizzled version of it
        let unswizzled_struct_name = Ident::new(&format!("{}Unswizzled", struct_name), struct_name.span());
        let unswizzled_struct_generics = struct_generics.clone();

        #[allow(non_snake_case)]
        let UNSWIZZLED_DEF = quote!{
            #[automatically_derived]
            #[derive(Debug)]
            pub struct #unswizzled_struct_name #unswizzled_struct_generics {
                #unswizzled_fields
            }
        };
        stream.extend(UNSWIZZLED_DEF);

        let empty = quote!{

        };
        (unswizzled_struct_name, unswizzled_struct_generics)
    } else {
        // if not, we can just use the normal struct version
        (struct_name.clone(), struct_generics.clone())
    };
    

    // implement the actual trait
    #[allow(non_snake_case)]
    let TRAIT_IMPL = quote!{
        #[automatically_derived]
        impl #struct_generics Swizzleable for #struct_name #struct_generics {
            type Unswizzled = #unswizzled_struct_name #unswizzled_struct_generics;
            unsafe fn unswizzle(outside: *mut Self, inside: *mut Self::Unswizzled) -> *mut Self::Unswizzled {
                #unswizzle_function
                println!("unswizzling from macro");
                inside
            }
        }
    };
    
    stream.extend(TRAIT_IMPL);
    return Ok(stream);
}


const PRIMITIVES: [&str; 14] = ["i8","i16","i32","i64","i128", "u8","u16","u32","u64","u128", "f32","f64", "usize", "isize"];
fn is_primitive(path: &syn::TypePath) -> bool {
    let type_str = format!("{}", &path.path.segments.first().unwrap().ident);
    PRIMITIVES.contains(&type_str.as_str())
}

fn field_with_path_name<'a>(mut old_field: syn::Field, new_name: &'a str, need_new_struct: &mut bool) -> (TokenStream, Ident) {
    if let syn::Type::Path(mut old_path) = old_field.ty {
        *need_new_struct = true;
        let old_ident = old_path.path.segments.first().unwrap().ident.clone();
        let mut old_segments = old_path.path.segments.iter_mut();
        old_segments.next().unwrap().ident = Ident::new(new_name, old_ident.span());
        old_field.ty = syn::Type::Path(old_path);
        (old_field.to_token_stream(), Ident::new(new_name, Span::call_site()))
    } else {
        panic!("field doesnt have a path");
    }
}