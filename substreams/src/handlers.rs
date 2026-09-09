//! Handler macros for Substreams.
//!
//! This create exports useful macros that you can use to develop
//! Substreams handlers. The goal of these macros is to significantly reduce boilerplate
//! code and ensure that your handler are more readable

/// Marks function to setup substreams map handler WASM boilerplate
///
/// ## Usage with Result<T, ...>
///
///
/// ```rust
/// # mod eth { pub type Block = substreams::testing::DocExampleMessage; }
/// # mod proto { pub type Custom = substreams::testing::DocExampleMessage; }
///
/// #[substreams::handlers::map]
/// fn map_handler(blk: eth::Block) -> Result<proto::Custom, substreams::errors::Error> {
///     unimplemented!("do something");
/// }
/// ```
///
/// Equivalent code not using `#[substreams::handlers::map]`
///
/// ```rust
/// # mod eth { pub type Block = substreams::testing::DocExampleMessage; }
/// # mod proto { pub type Custom = substreams::testing::DocExampleMessage; }
///
/// #[no_mangle]
/// pub extern "C" fn map_handler(blk_ptr: *mut u8, blk_len: usize) {
///     substreams::register_panic_hook();
///     let func = || -> Result<proto::Custom, substreams::errors::Error> {
///         let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len).unwrap();
///         {
///             unimplemented!("do something");
///         }
///     };
///     let result = func();
///     if result.is_err() {
///         panic!(result.err().unwrap())
///     }
///     substreams::output(result.unwrap());
/// }
///
/// ```
///
/// ## Usage with [Option]
///
///
/// ```rust
/// # mod eth { pub type Block = substreams::testing::DocExampleMessage; }
/// # mod proto { pub type Custom = substreams::testing::DocExampleMessage; }
///
/// #[substreams::handlers::map]
/// fn map_handler(blk: eth::Block) -> Option<proto::Custom> {
///     unimplemented!("do something");
/// }
/// ```
///
/// Equivalent code not using `#[substreams::handlers::map]`
///
/// ```rust
/// # mod eth { pub type Block = substreams::testing::DocExampleMessage; }
/// # mod proto { pub type Custom = substreams::testing::DocExampleMessage; }
///
/// #[no_mangle]
/// pub extern "C" fn map_handler(blk_ptr: *mut u8, blk_len: usize) {
///     substreams::register_panic_hook();
///     let func = || -> Option<proto::Custom> {
///         let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len).unwrap();
///         {
///             unimplemented!("do something");
///         }
///     };
///     let result = func();
///     if result.is_none() {
///         panic!("None returned from map function")
///     }
///     substreams::output(result.unwrap());
/// }
/// ```
///
/// ## Usage with T
///
///
/// ```rust
/// # mod eth { pub type Block = substreams::testing::DocExampleMessage; }
/// # mod proto { pub type Custom = substreams::testing::DocExampleMessage; }
///
/// #[substreams::handlers::map]
/// fn map_handler(blk: eth::Block) -> proto::Custom {
///     unimplemented!("do something");
/// }
/// ```
///
/// Equivalent code not using `#[substreams::handlers::map]`
///
/// ```rust
/// # mod eth { pub type Block = substreams::testing::DocExampleMessage; }
/// # mod proto { pub type Custom = substreams::testing::DocExampleMessage; }
///
/// #[no_mangle]
/// pub extern "C" fn map_handler(blk_ptr: *mut u8, blk_len: usize) {
///     substreams::register_panic_hook();
///     let func = || -> proto::Custom {
///         let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len).unwrap();
///         {
///             unimplemented!("do something");
///         }
///     };
///     let result = func();
///     substreams::output(result);
/// }
/// ```
///
/// ## Usage with a lazy view
///
/// Declaring the argument as a reference to a generated `FooLazyView<'_>` selects buffa's
/// lazy decoding: fields are decoded on access rather than up front.
///
/// ```rust
/// use substreams::pb::sf::substreams::{Clock, ClockLazyView};
///
/// #[substreams::handlers::map]
/// fn map_handler(blk: &ClockLazyView<'_>) -> Result<Clock, substreams::errors::Error> {
///     unimplemented!("do something");
/// }
/// ```
///
/// The same signature works with `no_testable`, which inlines the body into the export:
///
/// ```rust
/// use substreams::pb::sf::substreams::{Clock, ClockLazyView};
///
/// #[substreams::handlers::map(no_testable)]
/// fn map_handler(blk: &ClockLazyView<'_>) -> Result<Clock, substreams::errors::Error> {
///     unimplemented!("do something");
/// }
/// ```
pub use substreams_macro::map;

/// Marks function to setup substreams store handler WASM boilerplate
///
/// ## Usage with a lazy view
///
/// ```rust
/// use substreams::pb::sf::substreams::ClockLazyView;
/// use substreams::prelude::StoreNew;
/// use substreams::store::StoreAddInt64;
///
/// #[substreams::handlers::store]
/// fn store_handler(blk: &ClockLazyView<'_>, s: StoreAddInt64) {
///     unimplemented!("do something");
/// }
/// ```
///
/// ## Usage
///
///
/// ```rust
/// use substreams::prelude::{StoreGet, StoreNew};
/// use substreams::{log, store};
/// use substreams::store::{StoreGetProto, StoreAddInt64};
/// # mod proto {
/// #   pub type Custom = substreams::testing::DocExampleMessage;
/// #   pub type Pairs = substreams::testing::DocExampleMessage;
/// #   pub type Tokens = substreams::testing::DocExampleMessage;
/// # }
///
/// #[substreams::handlers::store]
/// fn build_nft_state(data: proto::Custom, s: StoreAddInt64, pairs: StoreGetProto<proto::Pairs>, tokens: StoreGetProto<proto::Tokens>) {
///     unimplemented!("do something");
/// }
/// ```
///
/// Equivalent code not using `#[substreams::handlers::store]`
///
/// ```rust
/// use substreams::prelude::StoreNew;
/// use substreams::{log, store};
/// use substreams::store::StoreGetProto;
/// # mod proto {
/// #   pub type Custom = substreams::testing::DocExampleMessage;
/// #   pub type Pairs = substreams::testing::DocExampleMessage;
/// #   pub type Tokens = substreams::testing::DocExampleMessage;
/// # }
///
/// #[no_mangle]
/// pub extern "C" fn build_nft_state(data_ptr: *mut u8, data_len: usize, pairs_idx: u32, tokens_idx: u32) {
///    substreams::register_panic_hook();
///    let data: proto::Custom = substreams::proto::decode_ptr(data_ptr, data_len).unwrap();
///    let pairs: StoreGetProto<proto::Pairs> = store::StoreGet::new(pairs_idx);
///    let tokens: StoreGetProto<proto::Tokens> = store::StoreGet::new(tokens_idx);
///    let s: store::StoreAddInt64 = store::StoreAddInt64::new();
///    {
///        unimplemented!("do something");
///    }
/// }
/// ```
pub use substreams_macro::store;
