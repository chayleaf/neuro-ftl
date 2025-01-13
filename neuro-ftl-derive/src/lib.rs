use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, DeriveInput, Ident, ItemStruct, LitStr};

fn vtable3(_attr: TokenStream, input: TokenStream, windows: bool) -> (TokenStream, TokenStream) {
    let mut inp: ItemStruct = syn::parse2(input).unwrap();
    let mut ret2 = TokenStream::new();
    let syn::Fields::Named(fields) = &mut inp.fields else {
        panic!("not named");
    };
    for field in &mut fields.named {
        let syn::Type::Path(ref mut path) = &mut field.ty else {
            panic!("not path type");
        };
        for seg in &mut path.path.segments {
            let syn::PathArguments::AngleBracketed(ref mut x) = &mut seg.arguments else {
                continue;
                // panic!("not angle");
            };
            let arg = x.args.first_mut().unwrap();
            let syn::GenericArgument::Type(ty) = arg else {
                panic!("not generic ty");
            };
            /*let syn::Type::Ptr(ptr) = ty else {
                panic!("not ptr")
            };*/
            let syn::Type::BareFn(fun) = ty else {
                panic!("not bare fn")
            };
            assert!(fun.abi.is_none());
            let mut args = TokenStream::new();
            let mut args2 = TokenStream::new();
            args.extend(quote! { &self });
            for (i, arg) in fun.inputs.iter().enumerate() {
                let name = Ident::new(&format!("arg{i}"), Span::call_site());
                let ty = arg.ty.clone();
                args.extend(quote! {
                    , #name: #ty
                });
                if windows && i == 1 {
                    args2.extend(quote! { std::ptr::null_mut(), });
                }
                args2.extend(quote! {
                    #name,
                });
            }
            let out = fun.output.clone();
            let name = field.ident.clone().unwrap();
            ret2.extend(quote! {
                pub unsafe fn #name(#args) #out {
                    (self.#name.unwrap())(#args2)
                }
            });
            fun.unsafety = Some(syn::token::Unsafe {
                span: Span::call_site(),
            });
            if windows {
                let mut inputs = Punctuated::new();
                let mut ty = None;
                for (i, inp) in fun.inputs.clone().into_iter().enumerate() {
                    if i == 1 {
                        inputs.push(syn::BareFnArg {
                            attrs: Default::default(),
                            name: None,
                            ty: ty.unwrap(),
                        });
                    }
                    ty = Some(inp.ty.clone());
                    inputs.push(inp);
                }
                fun.inputs = inputs;
                fun.abi = Some(syn::Abi {
                    extern_token: syn::token::Extern {
                        span: Span::call_site(),
                    },
                    name: Some(LitStr::new("fastcall", Span::call_site())),
                });
            } else {
                fun.abi = Some(syn::Abi {
                    extern_token: syn::token::Extern {
                        span: Span::call_site(),
                    },
                    name: Some(LitStr::new("C", Span::call_site())),
                });
            }
        }
    }
    let name = inp.ident.clone();
    let ret2 = quote! {
        impl #name {
            #ret2
        }
    };
    (inp.to_token_stream(), ret2)
}

fn vtable2(attr: TokenStream, input: TokenStream) -> TokenStream {
    let (linux1, linux2) = vtable3(attr.clone(), input.clone(), false);
    let (windows1, windows2) = vtable3(attr.clone(), input.clone(), true);
    // panic!("{}", windows1.to_string());
    quote! {
        #[cfg(target_os = "windows")]
        #[repr(C)]
        #[derive(Debug)]
        #windows1
        #[cfg(target_os = "windows")]
        #windows2
        #[cfg(target_os = "linux")]
        #[repr(C)]
        #[derive(Debug)]
        #linux1
        #[cfg(target_os = "linux")]
        #linux2
    }
}

#[proc_macro_attribute]
pub fn vtable(
    attr: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    vtable2(attr.into(), input.into()).into()
}

fn derive_test_offset2(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse2(input).unwrap();
    let syn::Data::Struct(x) = input.data else {
        panic!()
    };
    let mut tests = TokenStream::new();
    let ident = &input.ident;
    for field in x.fields {
        let field1 = field.ident.unwrap();
        for attr in field.attrs {
            if !attr.path().is_ident("test_offset") {
                continue;
            }
            let val = match attr.meta {
                syn::Meta::NameValue(x) => x.value,
                _ => panic!(),
            };
            tests.extend(quote! {
                assert_eq!(std::mem::offset_of!(super::#ident, #field1), #val);
            });
        }
    }
    let test_ident = Ident::new(
        &("__test_mod_".to_owned() + &input.ident.to_string()),
        Span::call_site(),
    );
    quote! {
        #[cfg(test)]
        #[allow(non_snake_case)]
        mod #test_ident {
            #[test]
            fn test() {
                #tests
            }
        }
    }
}

#[proc_macro_derive(TestOffsets, attributes(test_offset))]
pub fn derive_test_offset(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_test_offset2(input.into()).into()
}

fn derive_json_schema_no_ref2(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse2(input).unwrap();
    let name = &input.ident;

    let private_ident = Ident::new(
        &("__private_mod_".to_owned() + &name.to_string()),
        Span::call_site(),
    );
    quote! {
        #[allow(non_snake_case)]
        mod #private_ident {
            use super::*;
            #[derive(schemars::JsonSchema)]
            #input
        }

        impl schemars::JsonSchema for #name {
            fn schema_id() -> std::borrow::Cow<'static, str> {
                #private_ident::#name::schema_id()
            }
            fn schema_name() -> std::borrow::Cow<'static, str> {
                #private_ident::#name::schema_name()
            }
            fn json_schema(gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
                #private_ident::#name::json_schema(gen)
            }
            fn always_inline_schema() -> bool {
                true
            }
        }
    }
}

#[proc_macro_derive(JsonSchemaNoRef)]
pub fn derive_json_schema_no_ref(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_json_schema_no_ref2(input.into()).into()
}

fn derive_delta2(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse2(input).unwrap();
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = input;

    let syn::Generics {
        lt_token: _,
        gt_token: _,
        params,
        where_clause: wher,
    } = generics;
    // let mut impl_gen = TokenStream::new();
    let mut impl_gen2 = TokenStream::new();
    let mut ty_gen = TokenStream::new();
    let mut ty_gen2 = TokenStream::new();
    impl_gen2.extend(quote! { 'delta, });
    ty_gen2.extend(quote! { 'delta, });
    for param in params {
        match param {
            syn::GenericParam::Type(mut p) => {
                p.bounds
                    .push(syn::TypeParamBound::Lifetime(syn::Lifetime::new(
                        "'delta",
                        Span::call_site(),
                    )));
                let name = &p.ident;
                // impl_gen.extend(quote! { #p, });
                impl_gen2.extend(quote! { #p, });
                ty_gen.extend(quote! { #name, });
                ty_gen2.extend(quote! { #name, });
            }
            syn::GenericParam::Const(p) => {
                let name = &p.ident;
                // impl_gen.extend(quote! { #p, });
                impl_gen2.extend(quote! { #p, });
                ty_gen.extend(quote! { #name, });
                ty_gen2.extend(quote! { #name, });
            }
            syn::GenericParam::Lifetime(p) => {
                ty_gen.extend(quote! { #p, });
                ty_gen2.extend(quote! { #p, });
                // impl_gen.extend(quote! { #p, });
                impl_gen2.extend(quote! { #p, });
            }
        }
    }
    let ty_gen = if ty_gen.is_empty() {
        ty_gen
    } else {
        quote! { <#ty_gen> }
    };

    let syn::Data::Struct(data) = data else {
        panic!("must be struct")
    };

    let syn::Fields::Named(fields1) = data.fields else {
        panic!("fields must be named")
    };
    let attr = attrs
        .iter()
        .find(|x| x.path().to_token_stream().to_string().as_str() == "serde")
        .cloned();

    let mut fields = TokenStream::new();
    let mut body1 = TokenStream::new();
    let mut body2 = TokenStream::new();
    let mut has_id_ty = TokenStream::new();
    let mut has_id_ident = TokenStream::new();
    for field in fields1.named {
        let syn::Field {
            attrs,
            vis,
            mutability: _,
            ident,
            colon_token: _,
            ty,
        } = field;
        let mut attr = attrs
            .iter()
            .find(|x| x.path().to_token_stream().to_string().as_str() == "delta")
            .cloned();
        let attr1 = attrs
            .iter()
            .find(|x| x.path().to_token_stream().to_string().as_str() == "delta1")
            .cloned();
        let path = match attr.as_mut().map(|x| &mut x.meta) {
            None => None,
            Some(syn::Meta::Path(p)) => Some(p),
            Some(syn::Meta::List(x)) => Some(&mut x.path),
            Some(syn::Meta::NameValue(x)) => Some(&mut x.path),
        };
        if let Some(path) = path {
            path.segments.first_mut().unwrap().ident = Ident::new("serde", Span::call_site());
        }
        if attr1.is_some() {
            fields.extend(quote! {
                #attr
                #vis #ident: &'delta #ty,
            });
            body1.extend(quote! {
                let #ident = &self.#ident;
            });
            body2.extend(quote! {
                #ident,
            });
            has_id_ty.extend(quote! {
                &'delta #ty,
            });
            has_id_ident.extend(quote! {
                &self.#ident,
            });
        } else {
            fields.extend(quote! {
                #attr
                #[serde(skip_serializing_if = "Option::is_none")]
                #vis #ident: Option<<#ty as Delta<'delta>>::Delta>,
            });
            body1.extend(quote! {
                let #ident = self.#ident.delta(&prev.#ident);
                changed = changed || #ident.is_some();
            });
            body2.extend(quote! {
                #ident,
            });
        }
    }

    let name_delta = syn::Ident::new(&(ident.to_string() + "Delta"), Span::call_site());
    let mut ret = quote! {
        #[derive(Clone, Debug, serde::Serialize)]
        #attr
        #vis struct #name_delta <#impl_gen2> {
            #fields
        }
        impl <#impl_gen2> Delta<'delta> for #ident #ty_gen #wher {
            type Delta = #name_delta <#ty_gen2>;
            fn delta(&'delta self, prev: &'delta Self) -> Option<Self::Delta> {
                let mut changed = false;
                #body1
                if !changed {
                    return None;
                }
                Some(Self::Delta {
                    #body2
                })
            }
        }
    };
    if !has_id_ty.is_empty() {
        ret.extend(quote! {
            impl<#impl_gen2> HasId<'delta> for #ident #ty_gen #wher {
                type Id = (#has_id_ty);
                fn id(&'delta self) -> Self::Id {
                    (#has_id_ident)
                }
            }
        });
    }
    ret
}

#[proc_macro_derive(Delta, attributes(serde, delta, delta1))]
pub fn derive_delta(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    derive_delta2(input.into()).into()
}
