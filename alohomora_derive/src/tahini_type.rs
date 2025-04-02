use attribute_derive::FromAttr;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    spanned::Spanned, Data, DataStruct, DeriveInput, Field, Fields, Ident, ItemEnum, ItemStruct,
    Variant, Visibility,
};

pub type Error = (Span, &'static str);

#[derive(FromAttr, Clone)]
#[attribute(ident = alohomora_out_type)]
struct AlohomoraTypeArgs {
    name: Option<Ident>,
    to_derive: Option<Vec<Ident>>,
    verbatim: Option<Vec<Ident>>,
}
impl AlohomoraTypeArgs {
    pub fn is_verbatim(&self, ident: &str) -> bool {
        match &self.verbatim {
            None => false,
            Some(v) => {
                for i in v {
                    if &i.to_string() == ident {
                        return true;
                    }
                }
                false
            }
        }
    }
}

pub enum DataTypeEnum {
    Struct(ItemStruct),
    Enum(ItemEnum),
}

fn parse_derive_input_inner(
    input: DeriveInput,
) -> Result<(AlohomoraTypeArgs, DataTypeEnum), Error> {
    let attrs = AlohomoraTypeArgs::from_attributes(&input.attrs).unwrap();
    match input.data {
        Data::Enum(data_enum) => Ok((
            attrs,
            DataTypeEnum::Enum(ItemEnum {
                attrs: input.attrs,
                vis: input.vis,
                enum_token: data_enum.enum_token,
                ident: input.ident,
                generics: input.generics,
                brace_token: data_enum.brace_token,
                variants: data_enum.variants,
            }),
        )),
        Data::Union(_) => Err((
            input.ident.span(),
            "derive(AlohomoraType) only works on structs",
        )),
        Data::Struct(data_struct) => Ok((
            attrs,
            DataTypeEnum::Struct(ItemStruct {
                attrs: input.attrs,
                vis: input.vis,
                struct_token: data_struct.struct_token,
                ident: input.ident,
                generics: input.generics,
                fields: data_struct.fields,
                semi_token: data_struct.semi_token,
            }),
        )),
    }
}

pub fn derive_tahini_type_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let (attrs, enumed_input) = parse_derive_input_inner(input.clone())?;
    match enumed_input {
        DataTypeEnum::Enum(data_enum) => handle_enum(attrs, data_enum),
        DataTypeEnum::Struct(data_struct) => handle_struct(attrs, data_struct),
    }
}

fn generate_hashmap(attrs: AlohomoraTypeArgs, f: Fields, for_enum: bool) -> TokenStream {
    let fields: Vec<_> = f
        .iter()
        .map(|field| {
            (
                field
                    .ident
                    .as_ref()
                    .expect("Couldn't unwrap field ident #1")
                    .clone(),
                field
                    .ident
                    .as_ref()
                    .expect("Couldn't unwrap field ident #2")
                    .to_string(),
                field.ty.clone(),
            )
        })
        .collect();

    // Filter into those that are AlohomoraTypes themselves, and those who are kept verbatim.
    let tahini_fields: Vec<_> = fields
        .iter()
        .filter(|(_, string, _)| !attrs.is_verbatim(string))
        .cloned()
        .collect();
    let verbatium_fields: Vec<_> = fields
        .iter()
        .filter(|(_, string, _)| attrs.is_verbatim(string))
        .cloned()
        .collect();

    // Split field components.
    let tahini_fields_idents: Vec<_> = tahini_fields
        .iter()
        .map(|(ident, _, _)| ident.clone())
        .collect();

    let mut tahini_fields_strings: Vec<_> = Vec::new();
    for triplet in tahini_fields.iter() {
        tahini_fields_strings.push(triplet.1.as_str());
    }
    let tahini_fields_types: Vec<_> = tahini_fields.iter().map(|(_, _, ty)| ty.clone()).collect();

    let verbatim_fields_idents: Vec<_> = verbatium_fields
        .iter()
        .map(|(ident, _, _)| ident.clone())
        .collect();

    let mut verbatim_fields_strings: Vec<_> = Vec::new();
    for triplet in verbatium_fields.iter() {
        verbatim_fields_strings.push(triplet.1.as_str());
    }
    if for_enum {
        quote! {
                ::std::collections::HashMap::from([
                #((#tahini_fields_strings, <#tahini_fields_types as TahiniType>::to_tahini_enum(#tahini_fields_idents)),)*
                #((#verbatim_fields_strings, ::alohomora::tarpc::TahiniEnum::Value(Box::new(#verbatim_fields_idents))),)*
                ])
        }
    } else {
        quote! {
                ::std::collections::HashMap::from([
                #((#tahini_fields_strings, <#tahini_fields_types as TahiniType>::to_tahini_enum(&self.#tahini_fields_idents)),)*
                #((#verbatim_fields_strings, ::alohomora::tarpc::TahiniEnum::Value(Box::new(&self.#verbatim_fields_idents))),)*
                ])
        }
    }
}
fn handle_struct(attrs: AlohomoraTypeArgs, input: ItemStruct) -> Result<TokenStream, Error> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Expand needed variables.
    let input_ident = &input.ident;

    let body = generate_hashmap(attrs, input.fields.clone(), false);

    let binding = input_ident.to_string();
    let ident_str = binding.as_str();

    let fields_ident: Vec<_> = input
        .fields
        .iter()
        .map(|x| x.ident.as_ref().expect("Couldnt parse the field name"))
        .collect();
    // Generate implementation.
    // Can extract hashmap construction for named structs and variants. Only distinction is in what
    // do we wrap it. I believe that, the function generating the hashmap tokens should not be the
    // one handling all the fields generation, such as impl_generics, input_ident and whatnot.
    Ok(quote! {
        #[automatically_derived]
        #[doc = "Library implementation of TahiniType. Do not copy this docstring!"]
        impl #impl_generics ::alohomora::tarpc::TahiniType for #input_ident #ty_generics #where_clause {
            fn to_tahini_enum(&self) -> ::alohomora::tarpc::TahiniEnum {
                let mut map: ::std::collections::HashMap<&'static str, ::alohomora::tarpc::TahiniEnum> = #body;
                ::alohomora::tarpc::TahiniEnum::Struct(#ident_str, map)
            }
            fn tahini_policy_check(
                &self,
                members_fmt: &String,
                context: &::alohomora::context::UnprotectedContext,
                reason: &::alohomora::policy::Reason,
            ) -> bool{
                let mut policy_vec = Vec::new();
                #(policy_vec.push(self.#fields_ident.tahini_policy_check(members_fmt, context, reason));)*
                policy_vec.iter().all(|x: &bool| *x)
            }
        }
    })
}

//For each variant, we will check if their fields are named, unnamed, or unit.
//We invoke the handler for each of those different type.
fn handle_enum(attrs: AlohomoraTypeArgs, input: ItemEnum) -> Result<TokenStream, Error> {
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let input_ident = &input.ident;
    // Expand needed variables.
    let parsed_variants: Vec<_> = input
        .variants
        .iter()
        .map(|var| parse_variant(attrs.clone(), var))
        .collect();
    let indices: Vec<_> = input
        .variants
        .iter()
        .enumerate()
        .map(|(index, _)| index as u32)
        .collect();
    let variant_with_args: Vec<_> = input
        .variants
        .iter()
        .map(|v| parse_variant_ident(v))
        .collect();

    let var_strings: Vec<_> = input
        .variants
        .iter()
        .map(|v| v.ident.clone().to_string())
        .collect();
    let mut variant_strs = Vec::new();
    for var in var_strings.iter() {
        variant_strs.push(var.as_str());
    }
    let ident_str = input.ident.to_string();
    let ident_str = ident_str.as_str();

    let pol_check_bodies: Vec<_> = input
        .variants
        .iter()
        .map(|v| parse_variants_pol_check(attrs.clone(), v))
        .collect();

    Ok(quote! {

        #[automatically_derived]
        #[doc = "Library implementation of TahiniType. Do not copy this docstring!"]
        impl #impl_generics ::alohomora::tarpc::TahiniType for #input_ident #ty_generics #where_clause {
            fn to_tahini_enum(&self) -> ::alohomora::tarpc::TahiniEnum {
                match self {
                    #(#input_ident::#variant_with_args => ::alohomora::tarpc::enums::TahiniEnum::Enum(
                        #ident_str,
                        #indices,
                        #variant_strs,
                        #parsed_variants
                    )),*
                }
            }
            fn tahini_policy_check(
                &self,
                members_fmt: &String,
                context: &::alohomora::context::UnprotectedContext,
                reason: &::alohomora::policy::Reason,
            ) -> bool{
                match self {
                    #(#input_ident::#variant_with_args => {#pol_check_bodies}),*
                }
            }
        }
    })
}

fn parse_variant_ident(var: &Variant) -> TokenStream {
    let name = var.ident.clone();
    match var.fields {
        Fields::Named(..) => {
            let field_names: Vec<_> = var
                .fields
                .iter()
                .map(|x| x.ident.clone().expect("Couldn't unwrap ident #3"))
                .collect();
            quote! { #name{#(#field_names),*}}
        }
        Fields::Unnamed(..) => quote! {#name(x)},
        Fields::Unit => quote! {#name},
    }
}

fn parse_variant(attrs: AlohomoraTypeArgs, var: &Variant) -> TokenStream {
    match var.fields {
        Fields::Named(..) => handle_named_variant(attrs, var),
        Fields::Unnamed(..) => handle_unnamed_variant(var).unwrap(),
        Fields::Unit => handle_unit_variant(),
    }
}

fn parse_variants_pol_check(attrs: AlohomoraTypeArgs, var: &Variant) -> TokenStream {
    match var.fields {
        Fields::Named(..) => handle_named_variant_pol_check(attrs, var),
        Fields::Unnamed(..) => {
            handle_unnamed_variant_pol_check(var).expect("Trying to parse enum variant")
        }
        Fields::Unit => quote! {
                true

        },
    }
}

fn handle_named_variant_pol_check(attrs: AlohomoraTypeArgs, var: &Variant) -> TokenStream {
    let fields = var.fields.clone();
    let fields_ident: Vec<_> = fields
        .iter()
        .map(|field| {
            field
                .ident.as_ref()
                .expect("Couldn't unwrap named variant field name")
        })
        .collect();
    quote! {
            {
                let mut policy_vec = Vec::new();
                #(policy_vec.push(#fields_ident.tahini_policy_check(members_fmt, context, reason));)*
                policy_vec.iter().all(|x: &bool| *x)
            }
    }
}

fn handle_unnamed_variant_pol_check(var: &Variant) -> Result<TokenStream, Error> {
    match var.fields.len() {
        1 => Ok(quote! {
                    x.tahini_policy_check(members_fmt, context, reason)
        }),
        _ => {
            Err((
                var.ident.span(),
                "derive(TahiniType) does not work for tuple variants",
            ))
            // let fields_ident = var.fields.iter().map(|x| x);
            // todo!()
        } // quote! {
          // }
    }
}

fn handle_unit_variant() -> TokenStream {
    quote! {
        ::alohomora::tarpc::enums::TahiniVariantsEnum::Unit
    }
}

fn handle_unnamed_variant(var: &Variant) -> Result<TokenStream, Error> {
    match var.fields.len() {
        1 => Ok(
            quote! { ::alohomora::tarpc::enums::TahiniVariantsEnum::NewType(Box::new(x.to_tahini_enum()))},
        ),
        _ => {
            Err((
                var.ident.span(),
                "derive(TahiniType) does not work for tuple variants",
            ))
            // let fields_ident = var.fields.iter().map(|x| x);
            // todo!()
        } // quote! {
          // }
    }
}

fn handle_named_variant(attrs: AlohomoraTypeArgs, var: &Variant) -> TokenStream {
    let fields = var.fields.clone();
    let body = generate_hashmap(attrs, fields, true);
    quote! {
        {
            ::alohomora::tarpc::enums::TahiniVariantsEnum::Struct(#body)
        }
    }
}
