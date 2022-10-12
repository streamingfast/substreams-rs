# Change log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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