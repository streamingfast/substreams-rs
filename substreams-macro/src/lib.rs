use proc_macro::TokenStream;

mod assertions;
mod config;
mod errors;
mod handler;
mod store;

#[proc_macro_attribute]
pub fn map(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut keep_empty_output = false;
    match args.to_string().as_str() {
        "" => {},
        "keep_empty_output" => {keep_empty_output = true},
        _ => panic!("Invalid argument '{}' for map macro", args)
    }
    handler::main(item.into(), config::ModuleType::Map, keep_empty_output).into()
}

#[proc_macro_attribute]
pub fn store(args: TokenStream, item: TokenStream) -> TokenStream {
    let mut keep_empty_output = false;
    match args.to_string().as_str() {
        "" => {},
        "keep_empty_output" => {keep_empty_output = true},
        _ => panic!("Invalid argument '{}' for store macro", args)
    }
    handler::main(item.into(), config::ModuleType::Store, keep_empty_output).into()
}

// todo: remove this once satisfied with implementation of StoreDelete
#[proc_macro_derive(StoreWriter)]
pub fn derive(input: TokenStream) -> TokenStream {
    store::main(input)
}

#[cfg(test)]
mod test {
    use crate::{assertions::assert_ast_eq, config::ModuleType, handler::main};
    use quote::quote;

    #[test]
    fn test_map_plain() {
        let item = quote! {
            fn map_transfers(blk: eth::Block) -> pb::Custom {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, true).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let func = || -> pb::Custom {
                        let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                            .unwrap_or_else(|_| panic!("Unable to decode Protobuf data ({} bytes) to '{}' message's struct", blk_len, stringify!(eth::Block)));
                        let result = {
                            unimplemented!("do something");
                        };
                        result
                    };
                    let result = func();
                    substreams::output(result);
                }
            },
        );
    }

    #[test]
    fn test_map_mut() {
        let item = quote! {
            fn map_transfers(mut blk: eth::Block) -> pb::Custom {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, true).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let func = || -> pb::Custom {
                        let mut blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                            .unwrap_or_else(|_| panic!("Unable to decode Protobuf data ({} bytes) to '{}' message's struct", blk_len, stringify!(eth::Block)));
                        let result = {
                            unimplemented!("do something");
                        };
                        result
                    };
                    let result = func();
                    substreams::output(result);
                }
            },
        );
    }

    #[test]
    fn test_map_option() {
        let item = quote! {
            fn map_transfers(blk: eth::Block) -> Option<pb::Custom> {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, true).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let func = || -> Option<pb::Custom> {
                        let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                            .unwrap_or_else(|_| panic!("Unable to decode Protobuf data ({} bytes) to '{}' message's struct", blk_len, stringify!(eth::Block)));
                        let result = { unimplemented!("do something"); };
                        result
                    };

                    let result = func();
                    if let Some(value) = result {
                        substreams::output(value);
                    }
                }
            },
        );
    }

    #[test]
    fn test_map_result() {
        let item = quote! {
            fn map_transfers(blk: eth::Block) -> Result<pb::Custom> {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, true).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let func = || -> Result<pb::Custom> {
                        let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                            .unwrap_or_else(|_| panic!("Unable to decode Protobuf data ({} bytes) to '{}' message's struct", blk_len, stringify!(eth::Block)));
                        let result = { unimplemented!("do something"); };
                        result
                    };

                    let result = func();
                    if result.is_err() {
                        panic!("{:?}", result.unwrap_err())
                    }
                    substreams::output(result.expect("already checked that result is not an error"));
                }
            },
        );
    }

    #[test]
    fn test_map_result_option() {
        let item = quote! {
            fn map_transfers(blk: eth::Block) -> Result<Option<pb::Custom>> {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item.clone(), ModuleType::Map, true).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let func = || -> Result<Option<pb::Custom> > {
                        let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                            .unwrap_or_else(|_| panic!("Unable to decode Protobuf data ({} bytes) to '{}' message's struct", blk_len, stringify!(eth::Block)));
                        let result = { unimplemented!("do something"); };
                        result
                    };

                    let result = func();
                    if result.is_err() {
                        panic!("{:?}", result.unwrap_err())
                    }
                    if let Some(inner) = result.expect("already checked that result is not an error") {
                        substreams::output(inner);
                    }
                }
            },
        );

        assert_ast_eq(
            main(item, ModuleType::Map, false).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let func = || -> Result<Option<pb::Custom> > {
                        let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                            .unwrap_or_else(|_| panic!("Unable to decode Protobuf data ({} bytes) to '{}' message's struct", blk_len, stringify!(eth::Block)));
                        let result = { unimplemented!("do something"); };
                        result
                    };

                    substreams :: skip_empty_output () ; 
                    let result = func();
                    if result.is_err() {
                        panic!("{:?}", result.unwrap_err())
                    }
                    if let Some(inner) = result.expect("already checked that result is not an error") {
                        substreams::output(inner);
                    }
                }
            },
        );

    }

    #[test]
    fn test_store_result_option() {
        let item = quote! {
            fn store_values(blk: eth::Block, store: StoreAddInt64) {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item.clone(), ModuleType::Store, true).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn store_values(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                        .unwrap_or_else(|_|
                            panic!(
                                "Unable to decode Protobuf data ({} bytes) to '{}' message's struct",
                                blk_len, stringify!(eth::Block)
                            )
                        );
                    let store: StoreAddInt64 = StoreAddInt64::new();
                    let result = {
                        unimplemented!("do something");
                    };
                    result
                }
            },
        );

        assert_ast_eq(
            main(item, ModuleType::Store, false).into(),
            quote! {
                #[no_mangle]
                    pub extern "C" fn store_values(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                        .unwrap_or_else(|_|
                            panic!(
                                "Unable to decode Protobuf data ({} bytes) to '{}' message's struct",
                                blk_len, stringify!(eth::Block)
                            )
                        );
                    let store: StoreAddInt64 = StoreAddInt64::new();
                    substreams :: skip_empty_output () ; 
                    let result = {
                        unimplemented!("do something");
                    };
                    result
                }
            },
        );

    }
}