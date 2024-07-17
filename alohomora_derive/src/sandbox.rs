extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{FnArg, Ident, ItemFn, ReturnType};
// use ::alohomora_sandbox::alloc_mem_in_sandbox;

// TODO: (aportlan) macro still requires return type to be serializeable even tho we don't
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
            pub fn #invoke_sandbox_function_name_c(arg: *mut std::ffi::c_void, slot: usize) -> ::alohomora_sandbox::sandbox_out;
        }

        // This should be generated by a macro.
        #[cfg(not(target_arch = "wasm32"))]
        #[allow(non_camel_case_types)]
        pub struct #function_name {}

        #[cfg(not(target_arch = "wasm32"))]
        #[doc = "Library implementation of AlohomoraSandbox. Do not copy this docstring!"]
        impl<'__a, '__b> alohomora_sandbox::AlohomoraSandbox<'__a, '__b, #arg, #ret> for #function_name {
            fn invoke(arg: <#arg as ::alohomora_sandbox::Sandboxable>::InSandboxUnswizzled, sandbox_index: usize) -> *mut <#ret as ::alohomora_sandbox::Sandboxable>::InSandboxUnswizzled {
                alohomora_sandbox::invoke_sandbox!(#invoke_sandbox_function_name_c, arg, #arg, #ret, sandbox_index);
            }
        }

        // This should also be generated by macro.
        #[no_mangle]
        #[cfg(target_arch = "wasm32")]
        pub extern "C" fn #function_name_sandbox(arg: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
            alohomora_sandbox::sandbox_preamble(#function_name, arg)
        }

        #[no_mangle]
        #[cfg(target_arch = "wasm32")]
        pub extern "C" fn #function_name_sandbox_free(ret: *mut ::std::os::raw::c_char) {
            use std::ffi::CString;
            let _ = unsafe { CString::from_raw(ret) };
        }
    }
}

pub type Error = (proc_macro2::Span, &'static str);

pub fn derive_swizzleable_impl(input: syn::DeriveInput) -> Result<TokenStream, Error> {
    let mut stream = TokenStream::new();

    Ok(stream)
    // derive the 
}