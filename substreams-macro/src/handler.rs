use crate::config::{FinalConfiguration, ModuleType};
use crate::errors;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;

pub fn main(_args: TokenStream, item: TokenStream, module_type: ModuleType) -> TokenStream {
    let original = item.clone();

    let final_config = FinalConfiguration { module_type };
    let input = syn::parse_macro_input!(item as syn::ItemFn);

    let output_result = parse_func_output(&final_config, input.sig.output.clone());
    match output_result {
        Ok(_) => {}
        Err(e) => return token_stream_with_error(original, e),
    }
    let mut has_seen_writable_store = false;
    let mut args: Vec<proc_macro2::TokenStream> = Vec::with_capacity(input.sig.inputs.len() * 2);
    let mut proto_decodings: Vec<proc_macro2::TokenStream> =
        Vec::with_capacity(input.sig.inputs.len());
    let mut read_only_stores: Vec<proc_macro2::TokenStream> =
        Vec::with_capacity(input.sig.inputs.len());
    let mut writable_store: proc_macro2::TokenStream = quote! {};

    for i in (&input.sig.inputs).into_iter() {
        match i {
            syn::FnArg::Receiver(_) => {
                return token_stream_with_error(
                    original,
                    syn::Error::new(
                        i.span(),
                        format!("handler function does not support 'self' receiver"),
                    ),
                );
            }
            syn::FnArg::Typed(pat_type) => {
                match &*pat_type.pat {
                    syn::Pat::Ident(v) => {
                        let var_name = v.ident.clone();

                        let argument_type = &*pat_type.ty;
                        let input_res = parse_input_type(argument_type);
                        if input_res.is_err() {
                            return token_stream_with_error(
                                original,
                                syn::Error::new(
                                    pat_type.span(),
                                    format!("failed to parse input {:?}", input_res.err()),
                                ),
                            );
                        }
                        let input_obj = input_res.unwrap();

                        if input_obj.is_writable_store {
                            if has_seen_writable_store {
                                return token_stream_with_error(
                                original,
                                syn::Error::new(pat_type.span(), format!("handler cannot have more then one writable store as an input"))
                            );
                            }
                            has_seen_writable_store = true;
                            let store_type = format_ident!("{}", input_obj.store_type);
                            writable_store =
                                quote! { let #var_name: #argument_type = #store_type::new(); };
                            continue;
                        }

                        if input_obj.is_readable_store {
                            let var_idx = format_ident!("{}_idx", var_name);
                            let store_type = format_ident!("{}", input_obj.store_type);
                            args.push(quote! { #var_idx: u32 });
                            read_only_stores.push(quote! { let #var_name: #argument_type = #store_type::new(#var_idx); });
                            continue;
                        }

                        if final_config.module_type == ModuleType::Store
                            && var_name.to_string().ends_with("_idx")
                        {
                            args.push(quote! { #pat_type });
                            continue;
                        }
                        let var_ptr = format_ident!("{}_ptr", var_name);
                        let var_len = format_ident!("{}_len", var_name);
                        args.push(quote! { #var_ptr: *mut u8 });
                        args.push(quote! { #var_len: usize });

                        if input_obj.is_deltas {
                            let raw = format_ident!("raw_{}", var_name);
                            proto_decodings.push(quote! {
                                let #raw = substreams::proto::decode_ptr::<substreams::pb::substreams::StoreDeltas>(#var_ptr, #var_len).unwrap().deltas;
                                let #var_name: #argument_type = substreams::store::Deltas::new(#raw);
                            })
                        } else if input_obj.is_string {
                            proto_decodings.push(quote! { let #var_name: String = unsafe { String::from_raw_parts(#var_ptr, #var_len, #var_len) }; })
                        } else {
                            proto_decodings.push(quote! { let #var_name: #argument_type = substreams::proto::decode_ptr(#var_ptr, #var_len).unwrap(); })
                        }
                    }
                    _ => {
                        return token_stream_with_error(
                            original,
                            syn::Error::new(pat_type.span(), format!("unknown argument type")),
                        );
                    }
                }
            }
        }
    }

    match final_config.module_type {
        ModuleType::Store => build_store_handler(
            input,
            args,
            proto_decodings,
            read_only_stores,
            writable_store,
        ),
        ModuleType::Map => build_map_handler(
            input,
            args,
            proto_decodings,
            read_only_stores,
            writable_store,
        ),
    }
}

const WRITABLE_STORE: [&'static str; 22] = [
    "StoreSetRaw",
    "StoreSetString",
    "StoreSetBigInt",
    "StoreSetBigDecimal",
    "StoreSetProto",
    "StoreSetInt64",
    "StoreSetFloat64",
    "StoreSetIfNotExistsRaw",
    "StoreSetIfNotExistsProto",
    "StoreAddInt64",
    "StoreAddFloat64",
    "StoreAddBigDecimal",
    "StoreAddBigInt",
    "StoreMaxInt64",
    "StoreMaxBigInt",
    "StoreMaxFloat64",
    "StoreMaxBigDecimal",
    "StoreMinInt64",
    "StoreMinBigInt",
    "StoreMinFloat64",
    "StoreMinBigDecimal",
    "StoreAppend",
];

const READABLE_STORE: [&'static str; 8] = [
    "StoreGetInt64",
    "StoreGetFloat64",
    "StoreGetBigDecimal",
    "StoreGetBigInt",
    "StoreGetProto",
    "StoreGetRaw",
    "StoreGetString",
    "StoreGetArray",
];

#[derive(Debug)]
struct Input {
    is_writable_store: bool,
    is_readable_store: bool,
    is_deltas: bool,
    is_string: bool,
    resolved_ty: String,
    store_type: String,
}

fn parse_input_type(ty: &syn::Type) -> Result<Input, errors::SubstreamMacroError> {
    match ty {
        syn::Type::Path(p) => {
            let mut input = Input {
                is_writable_store: false,
                is_readable_store: false,
                is_deltas: false,
                is_string: false,
                resolved_ty: "".to_owned(),
                store_type: "".to_string(),
            };
            let mut last_type = "".to_owned();
            for segment in p.path.segments.iter() {
                last_type = segment.ident.to_string();
            }
            input.resolved_ty = last_type.clone();
            if last_type == "String".to_owned() {
                input.is_string = true;
            }
            for t in WRITABLE_STORE {
                if last_type == t.to_owned() {
                    input.is_writable_store = true;
                    input.store_type = last_type.clone();
                }
            }
            for t in READABLE_STORE {
                if last_type == t.to_owned() {
                    input.is_readable_store = true;
                    input.store_type = last_type.clone();
                }
            }
            if last_type == "Deltas".to_owned() {
                // todo: should check that it's fully qualified to be our `store::Deltas`
                input.is_deltas = true;
            }
            Ok(input)
        }
        _ => Err(errors::SubstreamMacroError::UnknownInputType(
            "unable to parse input type".to_owned(),
        )),
    }
}

fn parse_func_output(
    final_config: &FinalConfiguration,
    output: syn::ReturnType,
) -> Result<(), syn::Error> {
    return match final_config.module_type {
        ModuleType::Map => {
            if output == syn::ReturnType::Default {
                return Err(syn::Error::new(Span::call_site(), "Module of type Map should have a return of type Result<YOUR_TYPE, SubstreamError>"));
            }

            let expected = vec!["-".to_owned(), ">".to_owned(), "Result".to_owned()];
            let mut index = 0;
            let mut valid = true;
            for i in output.into_token_stream().into_iter() {
                if index == expected.len() {
                    return if valid {
                        Ok(())
                    } else {
                        Err(syn::Error::new(
                            Span::call_site(),
                            "Module of type Map should return a Result<>",
                        ))
                    };
                }
                if i.to_string() != expected[index] {
                    valid = false
                }
                index += 1;
            }
            Ok(())
        }
        ModuleType::Store => {
            if output != syn::ReturnType::Default {
                return Err(syn::Error::new(
                    Span::call_site(),
                    "Module of type Store should not have a return statement",
                ));
            }
            Ok(())
        }
    };
}

fn build_map_handler(
    input: syn::ItemFn,
    collected_args: Vec<proc_macro2::TokenStream>,
    decodings: Vec<proc_macro2::TokenStream>,
    read_only_stores: Vec<proc_macro2::TokenStream>,
    writable_store: proc_macro2::TokenStream,
) -> TokenStream {
    let body = &input.block;
    let header = quote! {
        #[no_mangle]
    };
    let func_name = input.sig.ident.clone();
    let lambda_return = input.sig.output.clone();
    let lambda = quote! {
        let func = || #lambda_return {
            #(#decodings)*
            #(#read_only_stores)*
            #writable_store
            #body
        };
    };
    let result = quote! {
        #header
        pub extern "C" fn #func_name(#(#collected_args),*){
            substreams::register_panic_hook();
            #lambda
            let result = func();
            if result.is_err() {
                panic!("{:?}", result.err().unwrap())
            }
            substreams::output(result.unwrap());
        }
    };
    result.into()
}

fn build_store_handler(
    input: syn::ItemFn,
    collected_args: Vec<proc_macro2::TokenStream>,
    decodings: Vec<proc_macro2::TokenStream>,
    read_only_stores: Vec<proc_macro2::TokenStream>,
    writable_store: proc_macro2::TokenStream,
) -> TokenStream {
    let body = &input.block;
    let header = quote! {
        #[no_mangle]
    };
    let func_name = input.sig.ident.clone();
    let result = quote! {
        #header
        pub extern "C" fn #func_name(#(#collected_args),*){
            substreams::register_panic_hook();
            #(#decodings)*
            #(#read_only_stores)*
            #writable_store
            #body
        }
    };
    result.into()
}

fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}
