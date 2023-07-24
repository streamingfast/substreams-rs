# Change log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 0.5.9

### Highlights

#### `#[substreams::handlers::map]` now handles `Result<Option<T>, Error>` and `Option<T>`

It's now possible to avoid sending back output from your mapper entirely by using `Option<T>` or `Result<Option<T>>`. This should be used whenever you are not returning something every block. This can make some use cases easier to "view" and comes with a small improved speed as the Protobuf encoding of an "empty" object will be avoided completely and a WASM intrinsic call will be avoided.

#### Error Handling

The `substreams::errors::Error` is now a plain alias to `anyhow::Error` which means it much easier to create generic errors, contextualize existing one and we gain the ability to be converted from any error that implements `std:error:Error` interface which is the majority of errors out there. This enables proper usage of the [`?` Rust operator](https://doc.rust-lang.org/reference/expressions/operator-expr.html#the-question-mark-operator).

```rust
#[substreams::handlers::map]
fn map_transfers(params: String, block: Block) -> Result<Transfers, substreams::errors::Error> {
    let address = Hex::decode(params)?;

    // ...
}
```

Here, a decoding error returned by `Hex::decode` will be converted to `substreams::errors::Error` and an early return will happen at that point. This will make error handling and reporting much easier.

The Rust [anyhow](https://docs.rs/anyhow/latest/anyhow/) library can now be used seamlessly to quickly write ad-hoc error as well as adding context to errors. First add `anyhow` as a dependency:

```bash
cargo add anyhow
```

Then use this code to contextualize another error:

```rust
use anyhow::Context;

#[substreams::handlers::map]
fn map_transfers(params: String, block: Block) -> Result<Transfers, substreams::errors::Error> {
    let address = Hex::decode(&params).with_context(|| format!("invalid address '{}'", &params))?;

    // ...
}
```

This should be a seamless upgrade for the vast majority of users. This change comes at the price that `Error::Unexpected("msg".to_string())` is not available anymore. Add `anyhow` as a dependency to your project:

```bash
cargo add anyhow
```

And then convert `substreams::errors:Error::Unexpected` usage with:

```rust
use anyhow::anyhow;

#[substreams::handlers::map]
fn map_transfers(block: Block) -> Result<Transfers, substreams::errors::Error> {
    if block.number == 0 {
        return Err(anyhow!("invalid block #{}", block.number))
    }

    // ...
}
```

### Changed

* Added support `Result<Option<>>` and `Option<>` in `substreams::handlers::map` macro.

* **Breaking** `substreams::errors:Error` is now an alias to `anyhow:Error`. This has been done for improving dealing with errors within Substreams Rust handler. If you were using `substreams::errors:Error::Unexpected`, now use `Err(anyhow!("invalid block #{}", block.number))` (add `anyhow = "1"` as a dependency of your project).

## [0.5.8](https://github.com/streamingfast/substreams-rs/release/tag/v0.5.8)

### Highlights

This is a re-packaging of https://github.com/streamingfast/substreams-rs/release/tag/v0.5.7 with a small removal that was actually wrong. Please see [0.5.7](https://github.com/streamingfast/substreams-rs/release/tag/v0.5.7) release notes for highlights of previous release.

### Fixed

* Removed `impl<I: Iterator>` from `Deltas`, this was implemented using `pop` which returns deltas in reverse order.

## [0.5.7](https://github.com/streamingfast/substreams-rs/release/tag/v0.5.7)

### Highlights

* New helpers to work with `store` and `delta` keys.
* Improved a bit performance of `delta` implementation.
* `BigInt` and `BigDecimal` quality of life improvements.

In this release we add various helpers to more easily decode store keys and extract meaningful information from them as well as dealing with store deltas.

In a lot of use cases, you will encode data into your keys for example `user:<address>` or `position:<pool>:<id>`. The new helpers make it easier than before to work with those. The Substreams default `key` key format is now to use the `:` segment separator to separate logical part of a key.

Import at the top of your module the `use substreams::store::DeltaExt;` trait and gain access to `key_segment_at_eq`, `key_first_segment_eq`, `key_last_segment_eq`, `key_first_segment_in` and `key_last_segment_in` on iterator of type `Delta`.

The new `key` module can then be used to extract useful part of the key:

```rust
use substreams::key;
use substreams::store::{Delta, DeltaExt, Deltas, DeltaBigDecimal};
fn db_out(store: Deltas<DeltaBigDecimal>) {
    for delta in store.key_first_segment_eq("user") {
        let address = key::segment_at(delta.get_key(), 1);
        // Do something for this delta where the key was in format `user:<address>`
    }
}
```

Or when filtering for multiple segments:

```rust
use substreams::key;
use substreams::store::{Delta, DeltaExt, Deltas, DeltaBigDecimal};
fn db_out(store: Deltas<DeltaBigDecimal>) {
    for delta in store.key_first_segment_in(["user", "contract"]) {
        // Do something for this delta where the key was in format `(user|contract):...`
    }
}
```

The `DeltaExt` trait also brings in `operation_eq` and `operation_not_eq` to filter `Deltas` based on the actual operation.

```rust
use substreams::key;
use substreams::pb::substreams::store_delta::Operation;
use substreams::store::{Delta, DeltaExt, Deltas, DeltaBigDecimal};
fn db_out(store: Deltas<DeltaBigDecimal>) {
    for delta in store
        .iter()
        .operation_eq(Operation::Create)
        .key_first_segment_in(["user", "contract"])
        .key_last_segment_eq("token0")
   {
        // Do something for Create delta(s) where the key was in format `(user|contract):...:token0`
    }
}
```

### Added

* Core: Added `key` module which contains extractor `segment_at`, `first_segment`, `last_segment`, `try_segment_at`, `try_first_segment` and `try_last_segment` to extract parts of a key.
* Stores: Added `store::DeltaExt` trait which contains predicates `key_segment_at_eq`, `key_first_segment_eq`, `key_last_segment_eq`, `key_first_segment_in`, `key_last_segment_in`, `operation_eq` and `operation_not_eq` for filtering of delta's keys.
* Stores: Added `get_key`, `get_operation` to the `Delta` trait, implemented for all Delta implementations.
* Macros: Add support for `Option<T>` and `T` as supported map output types, in addition to `Result<T, ...>`.
* Scalars: `BigInt` and `BigDecimal` types now implement the std `Default` trait (defaults to `0`) to be able to use `unwrap_or_default()`.
* Scalar: Added `absolute` method on `BigInt` and `BigDecimal` types.
* Scalar: Added `to_i32()` on `BigInt`

### Improved

* Stores: Reduced amount of clone performed in the `store` module which should improve speed a bit.

### Changed

* Stores: **Breaking** `Delta` trait method `new()` has been removed, removed by a trait bound `From<StoreDelta>` on `Deltas`, shouldn't affect anyone.
* Stores: **Breaking** `Deltas` now require the trait bound `From<StoreDelta>` implemented for all Delta implementations, shouldn't affect anyone.

## [0.5.6](https://github.com/streamingfast/substreams-rs/release/tag/v0.5.6)

* Macros: Add `StoreSetIfNotExists*` to the list of supported stores.

## [0.5.5](https://github.com/streamingfast/substreams-rs/release/tag/v0.5.5)

* Macros: Use `std::mem::ManuallyDrop` to manage memory instead of `std::mem::forget` for String input parameters.
* Macros: Add `StoreSetIfNotExistsString` to the list of supported stores.

## [0.5.4](https://github.com/streamingfast/substreams-rs/release/tag/v0.5.4)

* Bugfix: Fixed a bug where memory was not properly freed when using String input parameters in macros.
* Added `has_at`, `has_first` and `has_last` methods to `StoreGet`.

## [0.5.3](https://github.com/streamingfast/substreams-rs/release/tag/v0.5.3)

### Changed
* Support common-sense cross-type arithmetic operations for `BigInt` and `BigDecimal`. (e.g. `BigInt` + `f64` -> `BigDecimal`)

## [0.5.2](https://github.com/streamingfast/substreams-rs/release/tag/v0.5.1)

* Fixed tests
* Macros: Add support for String input parameters

## [0.5.1](https://github.com/streamingfast/substreams-rs/release/tag/v0.5.1)

* Add `from<usize>` for `BigDecimal`
* Added `new` method for `BigInt` and `BigDecimal`.
* Removed forced precision of 100 when returning a `BigDecimal` from a store.
* Fixed a bug where empty byte arrays were not properly handled

## [0.5.0](https://github.com/streamingfast/substreams-rs/release/tag/v0.5.0)

### Added
* Added `BigInt::from_unsigned_bytes_be` to create the `BigInt` from unsigned big endian bytes.

### Changed
* *Breaking* Changed signature of `BigInt::from_store_bytes(bytes: Vec<u8>)` to `BigInt::from_store_bytes(bytes: &[u8])`.
* *Breaking* Changed signature of `BigDecimal::from_store_bytes(bytes: Vec<u8>)` to `BigDecimal::from_store_bytes(bytes: &[u8])`.
* Improved implementation of `BigDecimal::divide_by_decimals` to rely on `BigDecimal` instead of a padded string.
* Reduced allocation performed when using `Store::set_if_not_exists_many`, `Store::set_many` and `Store::add_many` functions.
* Removed a bunch of unnecessary clones and removed some useless conversion which should increase overall speed of various `Store` and `Scalar` operations.

## [0.4.0](https://github.com/streamingfast/substreams-rs/release/tag/v0.4.0)

* Renaming `StoreSetIfNotExistsI64`, `StoreI64`, `DeltaI32`, `DeltaI64` to `StoreSetIfNotExistsInt64`, `StoreInt64`, `DeltaInt32` and `DeltaInt64`.
* Adding `StoreSetString`, `StoreGetString` and `StoreGetArray` typed stores.

## [0.3.2](https://github.com/streamingfast/substreams-rs/releases/tag/v0.3.2)

* Adding `DeltaI32`, `DeltaBool` and `DeltaBytes`.

## [0.3.1](https://github.com/streamingfast/substreams-rs/releases/tag/v0.3.1)

* Made Windows target(s) able to run tests when depending on `substreams` crate.

## [0.3.0](https://github.com/streamingfast/substreams-rs/releases/tag/v0.3.0)

* Abstraction of `StoreDelete` to implement `delete_prefix` and `StoreNew`.

* Removing config flag `wasm32`.

## [0.2.1](https://github.com/streamingfast/substreams-rs/releases/tag/v0.2.1)

* Added conditional compilation to make sure code that is linked to wasm modules can only be compiled when a wasm target is specified. Non-wasm targets will skip compiling the linked code allowing the crate to be compiled with any target.

## [0.2.0](https://github.com/streamingfast/substreams-rs/releases/tag/v0.2.0)

### Breaking changes

* Renamed all `{Types}StoreGet` (e.g.: `BigDecimalStoreGet`, `BigIntStoreGet`, etc.) and `{Types}StoreSet` (e.g.: `BigDecimalStoreSet`, `BigIntStoreSet`, etc.) to `StoreGet{Types}` and `StoreSet{Types}`
* Renamed all `{Types}Delta` (e.g.: `DeltaBigDecimal`, `DeltaBigInt`) to `Delta{Types}`
* Added `StoreGetI64` and `StoreSetI64`

## [0.1.0](https://github.com/streamingfast/substreams-rs/releases/tag/v0.1.0)

### Breaking changes

* `StoreGet`, `StoreSet` and `StoreSetIfNotExists` have all been changed from a `struct` to a `trait`
  * Multiple implementations for `StoreGet`, `StoreSet` and `StoreSetIfNotExists` have been added. Notably:
    * `BigDecimalStoreGet`, `BigDecimalStoreSet`, `BigIntStoreGet`, `BigIntStoreSet`. These stores are typed, meaning the user does not need to think about the encoding and the decoding as it's done for you. The user only needs to create a `BigDecimal` and store it. Storing and reading it will work out of the box for the users. No need to decode it.
    * `ProtoStoreGet<ProtobufType>`, `ProtoStoreSet<ProtobufType>` and `ProtoStoreSetIfNotExists<ProtobufType>`. All these implementations of proto have to be typed.
      example:
      ```bash
      #[derive(Clone, PartialEq, ::prost::Message)]
      pub struct ProtobufType {
          [...] // your attributes defined in your proto
      }

      #[substreams::handlers::map]
      pub fn map_my_substreams(store: ProtoStoreGet<ProtobufType>) -> Result<[...]> {
          [...]
      }
      ```
  * The previous `StoreGet`, `StoreSet` and `StoreSetIfNotExists` can still be used, but they need to be used as `RawStoreGet`, `RawStoreSet` and `RawStoreSetIfNotExists` which all use bytes as input/outputs instead of a typed value. Using any of the `Raw` stores has the same behaviour as before this release meaning if you have a `RawStore` of `BigInt`, `BigDecimal` or `ProtobufType` you would need to decode/encode them.
  * When fetching data from a typed `Store`, the user will not need to decode the returned value. Meaning a method call to `get_last()`, `get_at()` will all already return the decoded `ProtobufType` specified by the user.
* Custom `BigInt` and `BigDecimal` have been added to be able to add synthetic sugar and make the code more readable in a substreams.
  * Instead of doing manipulations like `BigDecimal::from_str(your_str_representation_of_big_decimal.as_str()).unwrap()` the user can do `let bd: BigDecimal = your_str_representation_of_big_decimal.indo()`. Much clearer and less convoluted


## [0.0.21](https://github.com/streamingfast/substreams-rs/releases/tag/v0.0.21)

* Ported rust modules from github.com/streamingfast/substreams to this repository
