//! A library for writing Substreams handlers.
//!
//! Substreams consists of a number of modules which provide structs and macros for
//! implementing Substreams handlers. The handlers are defined in your Manifest.
//!Learn more about Substreams at <https://substreams.streamingfast.io>
//!
//! ## Handler Examples
//!
//! Below are a few `map` handler examples. The signature of the handler function is based
//! on the inputs and output defined in the `map` module definition in the Manifest. There
//! are a few things to note:
//! * Best practice is to name your `map` module function `map_<your_action>`
//! * `map` module function must *always* return a [Result<Proto, E>]
//! * The Result must have an Error type set to [crate::errors::Error]
//!
//!```no_run
//! use substreams::prelude::{StoreGet, StoreNew};
//! use substreams::{errors::Error, store};
//! use substreams::store::{DeltaBigDecimal, StoreGetProto};
//! # mod eth { pub type Block = substreams::testing::DocExampleMessage; }
//! # mod pb { // holding all codegen'd protobuf structs
//! #   pub type Custom = substreams::testing::DocExampleMessage;
//! #   pub type Pairs = substreams::testing::DocExampleMessage;
//! # }
//!
//! /// Map handler which takes a source as input
//! #[substreams::handlers::map]
//! fn map_transfers(blk: eth::Block) -> Result<pb::Custom, Error> {
//!     unimplemented!("do something");
//! }
//!
//! /// Map handler which takes a source, and a store in get mode as inputs
//! #[substreams::handlers::map]
//! fn map_ownerships(blk: eth::Block, my_things: StoreGetProto<pb::Pairs>) -> Result<pb::Custom, Error> {
//!     unimplemented!("do something");
//! }
//!
//! /// Map handler which takes a source, another map, and a store in get mode as inputs
//! #[substreams::handlers::map]
//! fn map_mints(blk: eth::Block, mints: pb::Custom, myt_things: StoreGetProto<pb::Pairs>) -> Result<pb::Custom, Error> {
//!     unimplemented!("do something");
//! }
//!
//! /// Map handler which takes a source, another map, and a store in delta mode as inputs
//! #[substreams::handlers::map]
//! fn map_db(blk: eth::Block, mints: pb::Custom, store_deltas: store::Deltas<DeltaBigDecimal>) -> Result<pb::Custom, Error> {
//!     unimplemented!("do something");
//! }
//!
//! /// Map handler that can return no output or an error
//! #[substreams::handlers::map]
//! fn map_optional_transfers(blk: eth::Block) -> Result<Option<pb::Custom>, Error> {
//!     Ok(None)
//! }
//!
//! /// Map handler that can return no output
//! #[substreams::handlers::map]
//! fn map_maybe_transfers(blk: eth::Block) -> Option<pb::Custom> {
//!     None
//! }
//!
//! /// Map handler that can return its output only and directly
//! #[substreams::handlers::map]
//! fn map_direct_transfers(blk: eth::Block) -> pb::Custom {
//!     unimplemented!("do something");
//! }
//! ```
//!
//! Below are a few `store` handler examples. The signature of the handler function is based
//! on the inputs defined in the `store` module definition in the Manifest. There
//! are a few things to note:
//! * Best practice is to name your `map` module function `store_<your_action>`
//! * `store` module function must *return nothing*
//!
//! ```no_run
//! use substreams::store;
//! use substreams::prelude::{StoreGet, StoreNew};
//! use substreams::store::{DeltaBigDecimal, StoreGetProto, StoreAddInt64};
//! # mod pb {
//! #   use std::todo;
//! #   use substreams::pb::substreams::StoreDelta;
//! #   use substreams::store::Delta;
//! #   pub type Custom = substreams::testing::DocExampleMessage;
//! #
//! #   pub type Pairs = substreams::testing::DocExampleMessage;
//! #   pub type Tokens = substreams::testing::DocExampleMessage;
//! #   pub type Others = ();
//! # }
//!
//! #[substreams::handlers::store]
//! fn store_transfers(objects: pb::Custom, output: StoreAddInt64) {
//!     // to something
//! }
//!
//! #[substreams::handlers::store]
//! fn store_ownerships(objects: pb::Custom, store: StoreGetProto<pb::Pairs>, output: StoreAddInt64) {
//!     // to something
//! }
//!
//! #[substreams::handlers::store]
//! fn store_mints(objects: pb::Custom, store: StoreGetProto<pb::Pairs>, another_store: StoreGetProto<pb::Tokens>, store_deltas: store::Deltas<DeltaBigDecimal>, output: StoreAddInt64) {
//!     // to something
//! }
//!```
extern crate core;

pub mod errors;

mod externs;
pub mod handlers;
mod hex;
pub mod log;
pub mod memory;

/// Protobuf generated Substreams models
pub mod pb;
pub mod proto;
pub mod scalar;

mod state;

pub mod key;
pub mod store;

pub mod expr_parser;
pub use expr_parser::{expr_matcher, matches_keys_in_parsed_expr, ExprMatcher};

/// Experimental high-performance expression parser.
///
/// This module provides a faster alternative to [`expr_parser`] and will replace it in a future release.
/// See the module documentation for details.
pub mod sqe;

mod operation;

/// Testing utilities for Substreams handlers.
///
/// **Experimental API**: This module is under active development. While we aim to
/// minimize breaking changes, we reserve the right to modify the API as needed.
///
/// This module is only compiled when running tests (`#[cfg(test)]`).
pub mod testing;

/// A prelude that makes all store traits available.
///
/// Add the following code to import all traits listed below at once.
///
/// ```
/// use substreams::prelude::*;
/// ```
pub mod prelude {
    pub use crate::scalar::{BigDecimal, BigInt};
    pub use crate::store::{
        Appender, Delta, DeltaArray, DeltaBigDecimal, DeltaBigInt, DeltaBool, DeltaBytes,
        DeltaFloat64, DeltaInt32, DeltaInt64, DeltaProto, DeltaString, Deltas, FoundationalStore,
        StoreAdd, StoreAddBigDecimal, StoreAddBigInt, StoreAddFloat64, StoreAddInt64, StoreAppend,
        StoreDelete, StoreGet, StoreGetBigDecimal, StoreGetBigInt, StoreGetFloat64, StoreGetInt64,
        StoreGetProto, StoreGetRaw, StoreGetString, StoreMax, StoreMaxBigDecimal, StoreMaxBigInt,
        StoreMaxFloat64, StoreMaxInt64, StoreMin, StoreMinBigDecimal, StoreMinBigInt,
        StoreMinFloat64, StoreMinInt64, StoreNew, StoreSet, StoreSetBigDecimal, StoreSetBigInt,
        StoreSetFloat64, StoreSetIfNotExists, StoreSetIfNotExistsBigDecimal,
        StoreSetIfNotExistsBigInt, StoreSetIfNotExistsFloat64, StoreSetIfNotExistsInt64,
        StoreSetIfNotExistsProto, StoreSetIfNotExistsRaw, StoreSetIfNotExistsString, StoreSetInt64,
        StoreSetProto, StoreSetRaw, StoreSetString,
    };
}

pub use crate::hex::Hex;
pub use hex_literal::hex;

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn output<M: ::buffa::Message>(msg: M) {
    #[cfg(target_arch = "wasm32")]
    {
        // Need to return the buffer and forget about it issue occurred when trying to write large data
        // wasm was "dropping" the data before we could write to it, which causes us to have garbage
        // value. By forgetting the data we can properly call external output function to write the
        // msg to heap.
        let (ptr, len, buffer) = proto::encode_to_ptr(&msg);
        std::mem::forget(buffer);
        unsafe { externs::output(ptr, len as u32) }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn skip_empty_output() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        externs::skip_empty_output()
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn output_raw(data: Vec<u8>) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        externs::output(data.as_ptr(), data.len() as u32)
    }
}

/// Re-exported so downstream crates name `MessageField`, `EnumValue` and the
/// well-known types from the same buffa version this crate was built against.
pub use buffa;
pub use buffa_types;

/// buffa support module.
pub mod lazy {
    /// Lets the macro name a single decode call for a handler argument written as
    /// `&FooLazyView<'_>`; the impl strips the reference and forwards to buffa.
    pub trait LazyDecode<'a>: Sized {
        fn decode_lazy_slice(bytes: &'a [u8]) -> Result<Self, ::buffa::DecodeError>;
    }

    impl<'a, V> LazyDecode<'a> for V
    where
        V: ::buffa::view::LazyMessageView<'a>,
    {
        #[inline]
        fn decode_lazy_slice(bytes: &'a [u8]) -> Result<Self, ::buffa::DecodeError> {
            <V as ::buffa::view::LazyMessageView<'a>>::decode_lazy(bytes)
        }
    }
}

#[cfg(test)]
mod buffa_tests {
    use crate::lazy::LazyDecode;

    #[derive(Clone, Default, PartialEq)]
    struct Block;

    impl buffa::DefaultInstance for Block {
        fn default_instance() -> &'static Self {
            static DEFAULT: Block = Block;
            &DEFAULT
        }
    }

    impl buffa::Message for Block {
        fn compute_size(&self, _cache: &mut buffa::SizeCache) -> u32 {
            0
        }

        fn write_to(&self, _cache: &mut buffa::SizeCache, _buf: &mut impl buffa::EncodeSink) {}

        fn merge_field(
            &mut self,
            tag: buffa::encoding::Tag,
            buf: &mut impl bytes::Buf,
            _ctx: buffa::DecodeContext<'_>,
        ) -> Result<(), buffa::DecodeError> {
            buffa::encoding::skip_field(tag, buf)
        }

        fn clear(&mut self) {
            *self = Self;
        }
    }

    /// Stand-in for a generated `BlockLazyView<'a>`, so the blanket `LazyDecode` impl is
    /// exercised against the real `LazyMessageView` bound rather than only asserted as tokens
    /// in the macro's expected output.
    struct BlockLazyView<'a> {
        buf: &'a [u8],
    }

    impl<'a> buffa::view::LazyMessageView<'a> for BlockLazyView<'a> {
        type Owned = Block;

        fn decode_lazy(buf: &'a [u8]) -> Result<Self, buffa::DecodeError> {
            if buf.first() == Some(&0xff) {
                return Err(buffa::DecodeError::InvalidFieldNumber);
            }

            Ok(Self { buf })
        }

        fn merge_lazy(
            &mut self,
            _buf: &'a [u8],
            _ctx: buffa::DecodeContext<'_>,
        ) -> Result<(), buffa::DecodeError> {
            unimplemented!("not exercised by these tests")
        }

        fn to_owned_message(&self) -> Result<Self::Owned, buffa::DecodeError> {
            Ok(Block)
        }
    }

    #[test]
    fn it_decodes_a_lazy_view_borrowing_the_input() {
        let bytes = vec![0x01, 0x02, 0x03];

        let view = BlockLazyView::decode_lazy_slice(&bytes).expect("valid input");

        assert_eq!(view.buf, &bytes[..], "the view borrows the input buffer");
    }

    #[test]
    fn it_propagates_decode_errors() {
        let bytes = vec![0xff];

        assert!(BlockLazyView::decode_lazy_slice(&bytes).is_err());
    }
}

/// Registers a Substreams custom panic hook. The panic hook is invoked when then handler panics

pub fn register_panic_hook() {
    #[cfg(target_arch = "wasm32")]
    {
        use std::sync::Once;
        static SET_HOOK: Once = Once::new();
        SET_HOOK.call_once(|| {
            std::panic::set_hook(Box::new(hook));
        });
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused))]
fn hook(info: &std::panic::PanicHookInfo<'_>) {
    #[cfg(target_arch = "wasm32")]
    {
        let error_msg = info
            .payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| info.payload().downcast_ref::<&'static str>().copied())
            .unwrap_or("");
        let location = info.location();

        unsafe {
            let _ = match location {
                Some(loc) => {
                    let file = loc.file();
                    let line = loc.line();
                    let column = loc.column();

                    externs::register_panic(
                        error_msg.as_ptr(),
                        error_msg.len() as u32,
                        file.as_ptr(),
                        file.len() as u32,
                        line,
                        column,
                    )
                }
                None => externs::register_panic(
                    error_msg.as_ptr(),
                    error_msg.len() as u32,
                    std::ptr::null(),
                    0,
                    0,
                    0,
                ),
            };
        }
    }
}
