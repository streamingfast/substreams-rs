# Change log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.5](https://github.com/streamingfast/substreams-rs/release/tag/v0.5.5)

* Macros: Use std::mem::ManuallyDrop to manage memory instead of std::mem::forget for String input parameters.
* Macros: Add StoreSetIfNotExistsString to the list of supported stores.

## [0.5.4](https://github.com/streamingfast/substreams-rs/release/tag/v0.5.4)

* Bugfix: Fixed a bug where memory was not properly freed when using String input parameters in macros.
* Added `has_at`, `has_first` and `has_last` methods to StoreGet

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