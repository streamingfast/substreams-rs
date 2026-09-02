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
//! # mod eth { pub type Block = (); }
//! # mod pb { // holding all codegen'd protobuf structs
//! #   pub type Custom = ();
//! #   #[derive(Clone, PartialEq, ::prost::Message)]
//! #   pub struct Pairs {}
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
//! #   pub type Custom = ();
//! #
//! #   #[derive(Clone, PartialEq, ::prost::Message)]
//! #   pub struct Pairs {}
//! #   #[derive(Clone, PartialEq, ::prost::Message)]
//! #   pub struct Tokens {}
//! #   #[derive(Clone, PartialEq, ::prost::Message)]
//! #   pub struct Others {}
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
// pub use crate::store::FoundationalStore;
pub use hex_literal::hex;

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn output<M: prost::Message>(msg: M) {
    #[cfg(target_arch = "wasm32")]
    {
        // Need to return the buffer and forget about it issue occurred when trying to write large data
        // wasm was "dropping" the data before we could write to it, which causes us to have garbage
        // value. By forgetting the data we can properly call external output function to write the
        // msg to heap.
        let (ptr, len, buffer) = proto::encode_to_ptr(&msg).unwrap_or_else(|_| {
            panic!(
                "Unable to encode '{}' message's struct to Protobuf data",
                stringify!(M)
            )
        });
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

/// Quick-protobuf support module.
///
/// This module provides helpers for encoding and decoding quick-protobuf messages.
/// Only available when the `quick-protobuf` feature is enabled.
#[cfg(feature = "quick-protobuf")]
pub mod quick {
    use quick_protobuf::{BytesReader, MessageRead, MessageWrite, Result as QpResult};

    /// Output a quick-protobuf message.
    ///
    /// This function serializes the message using quick-protobuf and outputs it via the WASM host.
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
    pub fn output<M: MessageWrite>(msg: &M) {
        #[cfg(target_arch = "wasm32")]
        {
            use quick_protobuf::Writer;
            let size = msg.get_size();
            let mut buffer = Vec::with_capacity(size);
            {
                let mut writer = Writer::new(&mut buffer);
                msg.write_message(&mut writer).unwrap_or_else(|_| {
                    panic!(
                        "Unable to encode '{}' message's struct to Protobuf data",
                        std::any::type_name::<M>()
                    )
                });
            }
            let ptr = buffer.as_ptr();
            let len = buffer.len();
            std::mem::forget(buffer);
            unsafe { crate::externs::output(ptr, len as u32) }
        }
    }

    /// Decode a quick-protobuf message from a byte slice.
    pub fn decode<'a, T: MessageRead<'a>>(bytes: &'a [u8]) -> QpResult<T> {
        let mut reader = BytesReader::from_bytes(bytes);
        T::from_reader(&mut reader, bytes)
    }

    /// Decode a quick-protobuf message from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ptr` is valid for `size` bytes and that
    /// the memory remains valid for the lifetime of the returned message.
    pub unsafe fn decode_ptr<T: for<'a> MessageRead<'a>>(
        ptr: *mut u8,
        size: usize,
    ) -> QpResult<T> {
        let bytes = std::slice::from_raw_parts(ptr, size);
        let mut reader = BytesReader::from_bytes(bytes);
        T::from_reader(&mut reader, bytes)
    }
}

/// buffa support module.
///
/// Mirrors the `quick` module above, for buffa's owned `Message` API.
///
/// # Why the OWNED api, and not the lazy view
///
/// buffa's fastest decode is `FooLazyView::decode_lazy`, which records nested
/// message fields as undecoded byte ranges. It cannot be used here. The macro
/// hands the handler a value it owns, and a view borrows from the input buffer
/// -- `FooView<'a>` / `FooLazyView<'a>` carry a lifetime tied to those bytes,
/// so they cannot cross the `fn map_transfers(block: Block)` boundary the
/// handler signature defines.
///
/// Exposing the lazy path to module authors is a larger change: the handler
/// signature would have to take `BlockLazyView<'_>`, which changes every
/// module's source and every generated `pb` tree. That is a real option, and
/// the benchmarks say it is where the win is, but it is not a drop-in.
///
/// What this module gives is the drop-in half: the same `Block` the handler
/// already receives, decoded by buffa instead of prost.
#[cfg(feature = "buffa")]
pub mod buffa {
    /// Output a buffa message.
    #[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
    pub fn output<M: ::buffa::Message>(msg: &M) {
        #[cfg(target_arch = "wasm32")]
        {
            let buffer = ::buffa::Message::encode_to_vec(msg);
            let ptr = buffer.as_ptr();
            let len = buffer.len();
            std::mem::forget(buffer);
            unsafe { crate::externs::output(ptr, len as u32) }
        }
    }

    /// Decode a buffa message from a byte slice.
    pub fn decode<T: ::buffa::Message>(bytes: &[u8]) -> Result<T, ::buffa::DecodeError> {
        <T as ::buffa::Message>::decode_from_slice(bytes)
    }

    /// Decode a buffa message from a raw pointer.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `ptr` is valid for `size` bytes.
    pub unsafe fn decode_ptr<T: ::buffa::Message>(
        ptr: *mut u8,
        size: usize,
    ) -> Result<T, ::buffa::DecodeError> {
        let bytes = std::slice::from_raw_parts(ptr, size);
        <T as ::buffa::Message>::decode_from_slice(bytes)
    }
}

/// buffa lazy-view decoding for handler inputs.
///
/// buffa's lazy view is its fastest decode: `decode_lazy` does one
/// non-recursive scan over the message's own fields and records each nested or
/// repeated message field as an undecoded byte range, decoded on access. A
/// module that reads a few fields out of a large block never pays for the rest.
///
/// The view borrows from the input buffer, so a handler using it takes
/// `&FooLazyView<'_>` rather than an owned `Foo`. The generated export binds
/// the input bytes in its own scope and passes a reference, so the borrow is
/// live for the whole handler body.
///
/// This trait exists so the macro can name one call for an argument written as
/// `&FooLazyView<'_>`: the impl below strips the reference and forwards to
/// buffa's `LazyMessageView::decode_lazy`.
#[cfg(feature = "buffa")]
pub mod buffa_lazy {
    /// Decode `Self` (a `&FooLazyView<'a>`) from a byte slice.
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
