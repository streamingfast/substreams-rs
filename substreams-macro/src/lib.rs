use proc_macro::TokenStream;

mod assertions;
mod config;
mod errors;
mod handler;
mod store;

/// Marks a function as a Substreams map handler, generating the necessary WASM boilerplate.
///
/// # Panic Handling
///
/// Panics in handlers are trapped by the Substreams Engine and reported as deterministic
/// errors back to the user. This includes panics from `Result::Err` returns (which the
/// generated code converts to panics) and any explicit `panic!()` calls in your handler.
///
/// # Options
///
/// A handler declaring `&FooLazyView<'_>` decodes with a buffa lazy view; one declaring an
/// owned message decodes eagerly.
///
/// The macro accepts the following comma-separated options:
///
/// | Option | Description |
/// |--------|-------------|
/// | `no_testable` | Disables generation of the testable `__impl_<name>` function |
/// | `keep_empty_output` | Prevents calling `substreams::skip_empty_output()` |
///
/// # Basic Usage
///
/// ```ignore
/// #[substreams::handlers::map]
/// fn map_transfers(blk: eth::Block) -> Result<pb::Transfers, Error> {
///     // handler logic
/// }
/// ```
///
/// # Generated Code (default)
///
/// By default, the macro generates two functions:
/// 1. A testable `__impl_<name>` function with the original signature (always compiled)
/// 2. A WASM export function that calls the testable function (only on `wasm32` target)
///
/// For the example above, the macro generates:
///
/// ```ignore
/// // Testable function - always available, can be called directly in tests
/// pub fn __impl_map_transfers(blk: eth::Block) -> Result<pb::Transfers, Error> {
///     // user's handler body
/// }
///
/// // WASM export - only compiled for wasm32 target
/// #[cfg(target_arch = "wasm32")]
/// #[no_mangle]
/// pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
///     substreams::register_panic_hook();
///     let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
///         .unwrap_or_else(|_| panic!("Unable to decode..."));
///     substreams::skip_empty_output();
///     let result = __impl_map_transfers(blk);
///     if result.is_err() {
///         panic!("{:?}", result.unwrap_err())
///     }
///     substreams::output(result.expect("already checked"));
/// }
/// ```
///
/// # Testing Your Handlers
///
/// You can test your handler directly using the generated `__impl_` function:
///
/// ```ignore
/// #[test]
/// fn test_map_transfers() {
///     let blk = eth::Block::default();
///     // Call the testable function directly
///     let result = __impl_map_transfers(blk);
///     assert!(result.is_ok());
///
///     // Or use the test_map! macro for convenience
///     let result = substreams::test_map!(map_transfers(blk));
///     assert!(result.is_ok());
/// }
/// ```
///
/// # Disabling Testable Generation
///
/// Use `no_testable` to generate only the WASM export (legacy behavior):
///
/// ```ignore
/// #[substreams::handlers::map(no_testable)]
/// fn map_transfers(blk: eth::Block) -> Result<pb::Transfers, Error> {
///     // handler logic
/// }
/// ```
///
/// This generates a single function without the `__impl_` wrapper:
///
/// ```ignore
/// #[no_mangle]
/// pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
///     substreams::register_panic_hook();
///     let func = || -> Result<pb::Transfers, Error> {
///         let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
///             .unwrap_or_else(|_| panic!("Unable to decode..."));
///         // user's handler body
///     };
///     substreams::skip_empty_output();
///     let result = func();
///     if result.is_err() {
///         panic!("{:?}", result.unwrap_err())
///     }
///     substreams::output(result.expect("already checked"));
/// }
/// ```
///
/// # Combining Options
///
/// Options can be combined with commas:
///
/// ```ignore
/// #[substreams::handlers::map(no_testable, keep_empty_output)]
/// fn map_transfers(blk: eth::Block) -> Result<pb::Transfers, Error> {
///     // handler logic
/// }
/// ```
///
/// # Supported Return Types
///
/// - `Result<T, Error>` - Panics on error, outputs `T` on success
/// - `Result<Option<T>, Error>` - Panics on error, outputs `T` if `Some`, nothing if `None`
/// - `Option<T>` - Outputs `T` if `Some`, nothing if `None`
/// - `T` - Outputs `T` directly
#[proc_macro_attribute]
pub fn map(args: TokenStream, item: TokenStream) -> TokenStream {
    let options = config::HandlerOptions::parse(&args.to_string())
        .unwrap_or_else(|e| panic!("Invalid arguments for map macro: {}", e));
    handler::main(item.into(), config::ModuleType::Map, options).into()
}

/// Marks a function as a Substreams store handler, generating the necessary WASM boilerplate.
///
/// # Panic Handling
///
/// Panics in handlers are trapped by the Substreams Engine and reported as deterministic
/// errors back to the user. This includes any explicit `panic!()` calls in your handler.
///
/// # Options
///
/// A handler declaring `&FooLazyView<'_>` decodes with a buffa lazy view; one declaring an
/// owned message decodes eagerly.
///
/// The macro accepts the following comma-separated options:
///
/// | Option | Description |
/// |--------|-------------|
/// | `keep_empty_output` | Prevents calling `substreams::skip_empty_output()` |
///
/// **Note**: The `testable` option is not currently supported for store handlers.
///
/// # Usage
///
/// ```ignore
/// use substreams::store::StoreAddInt64;
///
/// #[substreams::handlers::store]
/// fn store_transfers(transfers: pb::Transfers, store: StoreAddInt64) {
///     // store logic
/// }
/// ```
///
/// Store handlers must not return a value. The writable store parameter is automatically
/// instantiated by the macro.
#[proc_macro_attribute]
pub fn store(args: TokenStream, item: TokenStream) -> TokenStream {
    let options = config::HandlerOptions::parse(&args.to_string())
        .unwrap_or_else(|e| panic!("Invalid arguments for store macro: {}", e));
    handler::main(item.into(), config::ModuleType::Store, options).into()
}

// todo: remove this once satisfied with implementation of StoreDelete
#[proc_macro_derive(StoreWriter)]
pub fn derive(input: TokenStream) -> TokenStream {
    store::main(input)
}

/// Test helper macro that transforms a handler call to use the `__impl_` testable function.
///
/// By default, `#[substreams::handlers::map]` generates a testable `__impl_<name>` function
/// alongside the WASM export. This macro provides a convenient way to call the testable
/// function in tests.
///
/// # Example
///
/// ```ignore
/// use substreams::test_map;
///
/// #[substreams::handlers::map]
/// fn map_transfers(blk: eth::Block) -> Result<Events, Error> {
///     // handler logic
/// }
///
/// #[test]
/// fn test_map_transfers() {
///     let blk = eth::Block::default();
///     let result = test_map!(map_transfers(blk));
///     assert!(result.is_ok());
/// }
/// ```
///
/// The macro transforms `test_map!(map_transfers(blk))` into `__impl_map_transfers(blk)`.
#[proc_macro]
pub fn test_map(input: TokenStream) -> TokenStream {
    use proc_macro2::TokenStream as TokenStream2;
    use quote::quote;
    use syn::{parse_macro_input, Expr, ExprCall, ExprPath};

    let call = parse_macro_input!(input as ExprCall);

    // Extract the function path from the call
    let func_expr = &*call.func;
    let args = &call.args;

    match func_expr {
        Expr::Path(ExprPath { path, .. }) => {
            // Get the last segment (function name) and prepend __impl_
            let mut new_path = path.clone();
            if let Some(last_segment) = new_path.segments.last_mut() {
                let new_name = quote::format_ident!("__impl_{}", last_segment.ident);
                last_segment.ident = new_name;
            }

            let result: TokenStream2 = quote! {
                #new_path(#args)
            };
            result.into()
        }
        _ => {
            panic!("test_map! expects a function call expression, e.g., test_map!(handler(args))")
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        assertions::assert_ast_eq,
        config::{HandlerOptions, ModuleType},
        handler::main,
    };
    use quote::quote;

    fn opts(keep_empty_output: bool, no_testable: bool) -> HandlerOptions {
        HandlerOptions {
            keep_empty_output,
            no_testable,
        }
    }

    // Tests for default behavior (testable enabled)
    #[test]
    fn test_map_default_plain() {
        let item = quote! {
            fn map_transfers(blk: eth::Block) -> pb::Custom {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                        .unwrap_or_else(|_| panic!("Unable to decode Protobuf data ({} bytes) to '{}' message's struct", blk_len, stringify!(eth::Block)));
                    let result = __impl_map_transfers(blk);
                    substreams::output(result);
                }

                pub fn __impl_map_transfers(blk: eth::Block) -> pb::Custom {
                    unimplemented!("do something");
                }
            },
        );
    }

    #[test]
    fn test_map_default_mut() {
        let item = quote! {
            fn map_transfers(mut blk: eth::Block) -> pb::Custom {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let mut blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                        .unwrap_or_else(|_| panic!("Unable to decode Protobuf data ({} bytes) to '{}' message's struct", blk_len, stringify!(eth::Block)));
                    let result = __impl_map_transfers(blk);
                    substreams::output(result);
                }

                pub fn __impl_map_transfers(mut blk: eth::Block) -> pb::Custom {
                    unimplemented!("do something");
                }
            },
        );
    }

    #[test]
    fn test_map_default_result() {
        let item = quote! {
            fn map_transfers(blk: eth::Block) -> Result<pb::Custom, Error> {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                        .unwrap_or_else(|_| panic!("Unable to decode Protobuf data ({} bytes) to '{}' message's struct", blk_len, stringify!(eth::Block)));
                    let result = __impl_map_transfers(blk);
                    if result.is_err() {
                        panic!("{:?}", result.unwrap_err())
                    }
                    substreams::output(result.expect("already checked that result is not an error"));
                }

                pub fn __impl_map_transfers(blk: eth::Block) -> Result<pb::Custom, Error> {
                    unimplemented!("do something");
                }
            },
        );
    }

    #[test]
    fn test_map_default_with_string_param() {
        let item = quote! {
            fn map_transfers(params: String, blk: eth::Block) -> Result<pb::Custom, Error> {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(params_ptr: *mut u8, params_len: usize, blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let params: String = std::mem::ManuallyDrop::new(unsafe { String::from_raw_parts(params_ptr, params_len, params_len) }).to_string();
                    let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                        .unwrap_or_else(|_| panic!("Unable to decode Protobuf data ({} bytes) to '{}' message's struct", blk_len, stringify!(eth::Block)));
                    let result = __impl_map_transfers(params, blk);
                    if result.is_err() {
                        panic!("{:?}", result.unwrap_err())
                    }
                    substreams::output(result.expect("already checked that result is not an error"));
                }

                pub fn __impl_map_transfers(params: String, blk: eth::Block) -> Result<pb::Custom, Error> {
                    unimplemented!("do something");
                }
            },
        );
    }

    #[test]
    fn test_map_default_with_readable_store() {
        let item = quote! {
            fn map_transfers(blk: eth::Block, store: StoreGetProto<pb::Pairs>) -> Result<pb::Custom, Error> {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize, store_idx: u32) {
                    substreams::register_panic_hook();
                    let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                        .unwrap_or_else(|_| panic!("Unable to decode Protobuf data ({} bytes) to '{}' message's struct", blk_len, stringify!(eth::Block)));
                    let store: StoreGetProto<pb::Pairs> = StoreGetProto::new(store_idx);
                    let result = __impl_map_transfers(blk, store);
                    if result.is_err() {
                        panic!("{:?}", result.unwrap_err())
                    }
                    substreams::output(result.expect("already checked that result is not an error"));
                }

                pub fn __impl_map_transfers(blk: eth::Block, store: StoreGetProto<pb::Pairs>) -> Result<pb::Custom, Error> {
                    unimplemented!("do something");
                }
            },
        );
    }

    #[test]
    fn test_map_default_with_writable_store() {
        let item = quote! {
            fn map_transfers(blk: eth::Block, output: StoreAddInt64) -> Result<pb::Custom, Error> {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len)
                        .unwrap_or_else(|_| panic!("Unable to decode Protobuf data ({} bytes) to '{}' message's struct", blk_len, stringify!(eth::Block)));
                    let output: StoreAddInt64 = StoreAddInt64::new();
                    let result = __impl_map_transfers(blk, output);
                    if result.is_err() {
                        panic!("{:?}", result.unwrap_err())
                    }
                    substreams::output(result.expect("already checked that result is not an error"));
                }

                pub fn __impl_map_transfers(blk: eth::Block, output: StoreAddInt64) -> Result<pb::Custom, Error> {
                    unimplemented!("do something");
                }
            },
        );
    }

    // Tests for no_testable option (legacy behavior)
    #[test]
    fn test_map_no_testable_plain() {
        let item = quote! {
            fn map_transfers(blk: eth::Block) -> pb::Custom {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, true)).into(),
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
    fn test_map_no_testable_option() {
        let item = quote! {
            fn map_transfers(blk: eth::Block) -> Option<pb::Custom> {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, true)).into(),
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
    fn test_map_no_testable_result_option() {
        let item = quote! {
            fn map_transfers(blk: eth::Block) -> Result<Option<pb::Custom>> {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item.clone(), ModuleType::Map, opts(true, true)).into(),
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
            main(item, ModuleType::Map, opts(false, true)).into(),
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
    fn test_store_handler() {
        let item = quote! {
            fn store_values(blk: eth::Block, store: StoreAddInt64) {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item.clone(), ModuleType::Store, opts(true, false)).into(),
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
            main(item, ModuleType::Store, opts(false, false)).into(),
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

    // Tests for the buffa option. The handler takes a lazy view, so the export binds the
    // input bytes in its own scope and passes a reference; the view borrows from them.
    #[test]
    fn test_map_lazy_plain() {
        let item = quote! {
            fn map_transfers(blk: &eth::BlockLazyView<'_>) -> pb::Custom {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let bytes_blk: &[u8] = unsafe {
                        std::slice::from_raw_parts(blk_ptr, blk_len)
                    };
                    let owned_blk = <eth::BlockLazyView<'_> as substreams::lazy::LazyDecode>::decode_lazy_slice(bytes_blk)
                        .unwrap_or_else(|_| panic!(
                            "Unable to decode buffa lazy view ({} bytes) for '{}'",
                            blk_len, stringify!(&eth::BlockLazyView<'_>)
                        ));
                    let blk = &owned_blk;
                    let result = __impl_map_transfers(blk);
                    substreams::output(result);
                }

                pub fn __impl_map_transfers(blk: &eth::BlockLazyView<'_>) -> pb::Custom {
                    unimplemented!("do something");
                }
            },
        );
    }

    #[test]
    fn test_map_lazy_result() {
        let item = quote! {
            fn map_transfers(blk: &eth::BlockLazyView<'_>) -> Result<pb::Custom, Error> {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let bytes_blk: &[u8] = unsafe {
                        std::slice::from_raw_parts(blk_ptr, blk_len)
                    };
                    let owned_blk = <eth::BlockLazyView<'_> as substreams::lazy::LazyDecode>::decode_lazy_slice(bytes_blk)
                        .unwrap_or_else(|_| panic!(
                            "Unable to decode buffa lazy view ({} bytes) for '{}'",
                            blk_len, stringify!(&eth::BlockLazyView<'_>)
                        ));
                    let blk = &owned_blk;
                    let result = __impl_map_transfers(blk);
                    if result.is_err() {
                        panic!("{:?}", result.unwrap_err())
                    }
                    substreams::output(result.expect("already checked that result is not an error"));
                }

                pub fn __impl_map_transfers(blk: &eth::BlockLazyView<'_>) -> Result<pb::Custom, Error> {
                    unimplemented!("do something");
                }
            },
        );
    }

    #[test]
    fn test_map_lazy_option() {
        let item = quote! {
            fn map_transfers(blk: &eth::BlockLazyView<'_>) -> Option<pb::Custom> {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let bytes_blk: &[u8] = unsafe {
                        std::slice::from_raw_parts(blk_ptr, blk_len)
                    };
                    let owned_blk = <eth::BlockLazyView<'_> as substreams::lazy::LazyDecode>::decode_lazy_slice(bytes_blk)
                        .unwrap_or_else(|_| panic!(
                            "Unable to decode buffa lazy view ({} bytes) for '{}'",
                            blk_len, stringify!(&eth::BlockLazyView<'_>)
                        ));
                    let blk = &owned_blk;
                    let result = __impl_map_transfers(blk);
                    if let Some(value) = result {
                        substreams::output(value);
                    }
                }

                pub fn __impl_map_transfers(blk: &eth::BlockLazyView<'_>) -> Option<pb::Custom> {
                    unimplemented!("do something");
                }
            },
        );
    }

    #[test]
    fn test_map_lazy_no_testable() {
        let item = quote! {
            fn map_transfers(blk: &eth::BlockLazyView<'_>) -> pb::Custom {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, true)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let func = || -> pb::Custom {
                    let bytes_blk: &[u8] = unsafe {
                        std::slice::from_raw_parts(blk_ptr, blk_len)
                    };
                    let owned_blk = <eth::BlockLazyView<'_> as substreams::lazy::LazyDecode>::decode_lazy_slice(bytes_blk)
                        .unwrap_or_else(|_| panic!(
                            "Unable to decode buffa lazy view ({} bytes) for '{}'",
                            blk_len, stringify!(&eth::BlockLazyView<'_>)
                        ));
                    let blk = &owned_blk;
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
    fn test_map_lazy_with_string_param() {
        let item = quote! {
            fn map_transfers(blk: &eth::BlockLazyView<'_>, name: String) -> pb::Custom {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize, name_ptr: *mut u8, name_len: usize) {
                    substreams::register_panic_hook();
                    let bytes_blk: &[u8] = unsafe {
                        std::slice::from_raw_parts(blk_ptr, blk_len)
                    };
                    let owned_blk = <eth::BlockLazyView<'_> as substreams::lazy::LazyDecode>::decode_lazy_slice(bytes_blk)
                        .unwrap_or_else(|_| panic!(
                            "Unable to decode buffa lazy view ({} bytes) for '{}'",
                            blk_len, stringify!(&eth::BlockLazyView<'_>)
                        ));
                    let blk = &owned_blk;
                    let name: String = std::mem::ManuallyDrop::new(unsafe {
                        String::from_raw_parts(name_ptr, name_len, name_len)
                    }).to_string();
                    let result = __impl_map_transfers(blk, name);
                    substreams::output(result);
                }

                pub fn __impl_map_transfers(blk: &eth::BlockLazyView<'_>, name: String) -> pb::Custom {
                    unimplemented!("do something");
                }
            },
        );
    }

    /// `StoreDeltas` is a substreams type carried over the host boundary, not module schema,
    /// so the deltas argument is decoded separately from the handler's own input.
    #[test]
    fn test_map_lazy_with_deltas() {
        let item = quote! {
            fn map_transfers(blk: &eth::BlockLazyView<'_>, deltas: Deltas<DeltaInt64>) -> pb::Custom {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize, deltas_ptr: *mut u8, deltas_len: usize) {
                    substreams::register_panic_hook();
                    let bytes_blk: &[u8] = unsafe {
                        std::slice::from_raw_parts(blk_ptr, blk_len)
                    };
                    let owned_blk = <eth::BlockLazyView<'_> as substreams::lazy::LazyDecode>::decode_lazy_slice(bytes_blk)
                        .unwrap_or_else(|_| panic!(
                            "Unable to decode buffa lazy view ({} bytes) for '{}'",
                            blk_len, stringify!(&eth::BlockLazyView<'_>)
                        ));
                    let blk = &owned_blk;
                    let raw_deltas = substreams::proto::decode_ptr::<substreams::pb::substreams::StoreDeltas>(deltas_ptr, deltas_len)
                        .unwrap_or_else(|_| panic!(
                            "Unable to decode Protobuf data ({} bytes) to 'substreams::pb::substreams::StoreDeltas' message's struct",
                            deltas_len
                        )).store_deltas;
                    let deltas: Deltas<DeltaInt64> = substreams::store::Deltas::new(raw_deltas);
                    let result = __impl_map_transfers(blk, deltas);
                    substreams::output(result);
                }

                pub fn __impl_map_transfers(blk: &eth::BlockLazyView<'_>, deltas: Deltas<DeltaInt64>) -> pb::Custom {
                    unimplemented!("do something");
                }
            },
        );
    }

    #[test]
    fn test_map_lazy_with_readable_store() {
        let item = quote! {
            fn map_transfers(blk: &eth::BlockLazyView<'_>, store: StoreGetInt64) -> pb::Custom {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Map, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize, store_idx: u32) {
                    substreams::register_panic_hook();
                    let bytes_blk: &[u8] = unsafe {
                        std::slice::from_raw_parts(blk_ptr, blk_len)
                    };
                    let owned_blk = <eth::BlockLazyView<'_> as substreams::lazy::LazyDecode>::decode_lazy_slice(bytes_blk)
                        .unwrap_or_else(|_| panic!(
                            "Unable to decode buffa lazy view ({} bytes) for '{}'",
                            blk_len, stringify!(&eth::BlockLazyView<'_>)
                        ));
                    let blk = &owned_blk;
                    let store: StoreGetInt64 = StoreGetInt64::new(store_idx);
                    let result = __impl_map_transfers(blk, store);
                    substreams::output(result);
                }

                pub fn __impl_map_transfers(blk: &eth::BlockLazyView<'_>, store: StoreGetInt64) -> pb::Custom {
                    unimplemented!("do something");
                }
            },
        );
    }

    #[test]
    fn test_store_lazy_handler() {
        let item = quote! {
            fn store_values(blk: &eth::BlockLazyView<'_>, store: StoreAddInt64) {
                unimplemented!("do something");
            }
        };

        assert_ast_eq(
            main(item, ModuleType::Store, opts(true, false)).into(),
            quote! {
                #[no_mangle]
                pub extern "C" fn store_values(blk_ptr: *mut u8, blk_len: usize) {
                    substreams::register_panic_hook();
                    let bytes_blk: &[u8] = unsafe {
                        std::slice::from_raw_parts(blk_ptr, blk_len)
                    };
                    let owned_blk = <eth::BlockLazyView<'_> as substreams::lazy::LazyDecode>::decode_lazy_slice(bytes_blk)
                        .unwrap_or_else(|_| panic!(
                            "Unable to decode buffa lazy view ({} bytes) for '{}'",
                            blk_len, stringify!(&eth::BlockLazyView<'_>)
                        ));
                    let blk = &owned_blk;
                    let store: StoreAddInt64 = StoreAddInt64::new();
                    let result = {
                        unimplemented!("do something");
                    };
                    result
                }
            },
        );
    }
}
