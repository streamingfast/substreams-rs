//! Store Implementation for Substreams.
//!
//! This module is the heart of all your interactions with your Substreams store needs be it setting values via
//! Store or retrieving value via StoreGet and Delta.
//!
//! The DeltaExt trait when imported in your code will bring in must-use functions on Deltas iterator for filtering
//! deltas based on its key or operationt. The [key] module is also available for extracting segments of a key.
//!
//! In a lot of use cases, you will encode data into your keys for example `user:<address>` or `position:<pool>:<id>`.
//! The `DeltaExt` trait exists to pick up deltas that matches a specific key pattern as well as extracting segment of a key.
//!
//! The `DeltaExt` trait expects keys to be of the form `<segment>[:<segment>]*` (just like the [key] module) so the `:` is
//! the segment separator. The module is meant to be used like this for filtering specific deltas and extracting specific segments:
//!
//! ```rust
//! use substreams::key;
//! use substreams::store::{Delta, DeltaExt, Deltas, DeltaBigDecimal};
//!
//! fn db_out(store: Deltas<DeltaBigDecimal>) {
//!     for delta in store.into_iter().key_first_segment_eq("user") {
//!         let address = key::segment_at(delta.get_key(), 1);
//!
//!         // Do something for this delta where the key was in format `user:<address>`
//!     }
//! }
//! ```
//!
//! Or when filtering for multiple segments:
//!
//! ```rust
//! use substreams::key;
//! use substreams::store::{Delta, DeltaExt, Deltas, DeltaBigDecimal};
//!
//! fn db_out(store: Deltas<DeltaBigDecimal>) {
//!     for delta in store.into_iter().key_first_segment_in(["user", "contract"]) {
//!         // Do something for this delta where the key was in format `(user|contract):...`
//!     }
//! }
//! ```
//!
//! You can also filter per operations and merge all this together:
//!
//! ```rust
//! use substreams::key;
//! use substreams::pb::substreams::store_delta::Operation;
//! use substreams::store::{Delta, DeltaExt, Deltas, DeltaBigDecimal};
//!
//! fn db_out(store: Deltas<DeltaBigDecimal>) {
//!     for delta in store
//!         .iter()
//!         .operation_eq(Operation::Create)
//!         .key_first_segment_in(["user", "contract"])
//!         .key_last_segment_eq("token0")
//!    {
//!         // Do something for Create delta where the key was in format `(user|contract):...:token0`
//!     }
//! }
//! ```
use std::{io::BufRead, str};

use crate::{key, operation, pb::substreams::store_delta::Operation};

use {
    crate::{
        pb::substreams::StoreDelta,
        scalar::{BigDecimal, BigInt},
        state, {pb, proto},
    },
    prost,
    std::i64,
    std::marker::PhantomData,
    std::str::FromStr,
};

/// `StoreSet` is a trait which is implemented on any type of typed StoreSet
pub trait StoreSet<V>: StoreNew + StoreDelete {
    /// Set a given key to a given value, if the key existed before, it will be replaced.
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &V);
    /// Set many keys to a given values, if the key existed before, it will be replaced.
    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &V);
}

pub trait StoreDelete {
    /// Delete values in a store given prefixed string
    fn delete_prefix(&self, ord: i64, prefix: &String) {
        state::delete_prefix(ord, prefix);
    }
}

pub trait StoreNew {
    /// Create an instance of trait implementation
    fn new() -> Self;
}

/// `StoreSetRaw` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `bytes`
///     `StoreSetRaw` implements AsRef<[u8]> to give the client the flexibility
///     to either use the API with &Vec[...] or Vec[...].
pub struct StoreSetRaw {}
impl StoreNew for StoreSetRaw {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetRaw {}

impl<V: AsRef<[u8]>> StoreSet<V> for StoreSetRaw {
    /// Set a given key to a given value, if the key existed before, it will be replaced.
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &V) {
        state::set(ord as i64, key, value);
    }

    /// Set many keys to a given values, if the key existed before, it will be replaced.
    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &V) {
        for key in keys {
            state::set(ord as i64, key, value);
        }
    }
}

/// `StoreSetString` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `string`
/// `StoreSetString` implements `AsRef<str>` to give the client the flexibility
/// to either use the API with &String or String.
pub struct StoreSetString {}
impl StoreNew for StoreSetString {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetString {}

impl<V: AsRef<str>> StoreSet<V> for StoreSetString {
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &V) {
        state::set(ord as i64, key, value.as_ref());
    }

    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &V) {
        let value = value.as_ref();

        for key in keys {
            state::set(ord as i64, key, value);
        }
    }
}

/// `StoreSetInt64` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `int64`
pub struct StoreSetInt64 {}
impl StoreNew for StoreSetInt64 {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetInt64 {}

impl StoreSet<i64> for StoreSetInt64 {
    /// Set a given key to a given value, if the key existed before, it will be replaced.
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &i64) {
        state::set(ord as i64, key, value.to_string().as_bytes());
    }

    /// Set many keys to a given values, if the key existed before, it will be replaced.
    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &i64) {
        let as_str = value.to_string();

        for key in keys {
            state::set(ord as i64, key, &as_str);
        }
    }
}

/// `StoreSetFloat64` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `float64`
pub struct StoreSetFloat64 {}
impl StoreNew for StoreSetFloat64 {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetFloat64 {}

impl StoreSet<f64> for StoreSetFloat64 {
    /// Set a given key to a given value, if the key existed before, it will be replaced.
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &f64) {
        state::set(ord as i64, key, value.to_string().as_bytes());
    }

    /// Set many keys to a given values, if the key existed before, it will be replaced.
    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &f64) {
        let as_str = value.to_string();

        for key in keys {
            state::set(ord as i64, key, &as_str);
        }
    }
}

/// `StoreSetBigDecimal` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `bigdecimal`
pub struct StoreSetBigDecimal {}
impl StoreNew for StoreSetBigDecimal {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetBigDecimal {}

impl StoreSet<BigDecimal> for StoreSetBigDecimal {
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &BigDecimal) {
        state::set(ord as i64, key, value.to_string().as_bytes())
    }

    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &BigDecimal) {
        let as_str = value.to_string();

        for key in keys {
            state::set(ord as i64, key, &as_str)
        }
    }
}

/// `StoreSetBigInt` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `bigint`
pub struct StoreSetBigInt {}
impl StoreNew for StoreSetBigInt {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetBigInt {}

impl StoreSet<BigInt> for StoreSetBigInt {
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &BigInt) {
        state::set(ord as i64, key, value.as_ref().to_string().as_bytes());
    }

    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &BigInt) {
        let as_str = value.as_ref().to_string();

        for key in keys {
            state::set(ord as i64, key, &as_str);
        }
    }
}

/// `StoreSetProto` is a struct representing a `store` with `updatePolicy` equal to `set` and a `valueType` equal to `proto:{your_proto_type}`
#[allow(dead_code)]
pub struct StoreSetProto<V: Default + prost::Message> {
    casper: PhantomData<V>,
}

impl<V: Default + prost::Message> StoreDelete for StoreSetProto<V> {}

impl<V: Default + prost::Message> StoreNew for StoreSetProto<V> {
    fn new() -> Self {
        Self {
            // Adding a PhantomData<T> field to your type tells the compiler that
            // your type acts as though it stores a value of type T, even though
            // it doesn't really. This information is used when computing certain
            // safety properties. For a more in-depth explanation of how to use
            // PhantomData<T>
            casper: PhantomData,
        }
    }
}

impl<V: Default + prost::Message> StoreSet<V> for StoreSetProto<V> {
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &V) {
        let bytes = proto::encode(value)
            .unwrap_or_else(|_| panic!("Unable to encode store message's struct to Protobuf data"));

        state::set(ord as i64, key, &bytes)
    }

    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &V) {
        let bytes = proto::encode(value)
            .unwrap_or_else(|_| panic!("Unable to encode store message's struct to Protobuf data"));

        for key in keys {
            state::set(ord as i64, key, &bytes)
        }
    }
}

/// `StoreSetIfNotExists` is a trait which is implemented on any type of typed StoreSetIfNotExists
pub trait StoreSetIfNotExists<V>: StoreDelete + StoreNew {
    /// Set a given key to a given value, if the key existed before, it will be ignored and not set.
    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: &V);
    /// Set given keys to given values, if the key existed before, it will be ignored and not set.
    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &V);
}

/// `StoreSetIfNotExistsRaw` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `string`
///     `StoreSetIfNotExistsRaw` implements AsRef<[u8]> to give the client the flexibility
///     to either use the API with &Vec[...] or Vec[...].
pub struct StoreSetIfNotExistsRaw {}
impl StoreNew for StoreSetIfNotExistsRaw {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetIfNotExistsRaw {}

impl<V: AsRef<[u8]>> StoreSetIfNotExists<V> for StoreSetIfNotExistsRaw {
    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: &V) {
        state::set_if_not_exists(ord as i64, key, value.as_ref());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &V) {
        let value = value.as_ref();

        for key in keys {
            state::set_if_not_exists(ord as i64, key, value);
        }
    }
}

/// `StoreSetIfNotExistsString` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `string`
/// `StoreSetIfNotExistsString` implements `AsRef<str>` to give the client the flexibility
/// to either use the API with &String or String.
pub struct StoreSetIfNotExistsString {}
impl StoreNew for StoreSetIfNotExistsString {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetIfNotExistsString {}

impl<V: AsRef<str>> StoreSetIfNotExists<V> for StoreSetIfNotExistsString {
    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: &V) {
        state::set_if_not_exists(ord as i64, key, value.as_ref().as_bytes());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &V) {
        let value = value.as_ref();

        for key in keys {
            state::set_if_not_exists(ord as i64, key, value);
        }
    }
}

/// `StoreSetIfNotExistsBigDecimal` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `bigdecimal`
pub struct StoreSetIfNotExistsBigDecimal {}
impl StoreNew for StoreSetIfNotExistsBigDecimal {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetIfNotExistsBigDecimal {}

impl StoreSetIfNotExists<BigDecimal> for StoreSetIfNotExistsBigDecimal {
    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: &BigDecimal) {
        state::set_if_not_exists(ord as i64, key, value.as_ref().to_string().as_bytes());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &BigDecimal) {
        let as_str = value.to_string();

        for key in keys {
            state::set_if_not_exists(ord as i64, key, &as_str);
        }
    }
}

/// `StoreSetIfNotExistsBigInt` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `bigint`
pub struct StoreSetIfNotExistsBigInt {}
impl StoreNew for StoreSetIfNotExistsBigInt {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetIfNotExistsBigInt {}

impl StoreSetIfNotExists<BigInt> for StoreSetIfNotExistsBigInt {
    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: &BigInt) {
        state::set_if_not_exists(ord as i64, key, value.to_string().as_bytes());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &BigInt) {
        let as_str = value.to_string();

        for key in keys {
            state::set_if_not_exists(ord as i64, key, &as_str);
        }
    }
}

/// `StoreSetIfNotExistsInt64` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `int64`
pub struct StoreSetIfNotExistsInt64 {}
impl StoreNew for StoreSetIfNotExistsInt64 {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetIfNotExistsInt64 {}

impl StoreSetIfNotExists<i64> for StoreSetIfNotExistsInt64 {
    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: &i64) {
        state::set_if_not_exists(ord as i64, key, value.to_string().as_bytes());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &i64) {
        let as_str = value.to_string();

        for key in keys {
            state::set_if_not_exists(ord as i64, key, &as_str);
        }
    }
}

/// `StoreSetIfNotExistsFloat64` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `float64`
pub struct StoreSetIfNotExistsFloat64 {}
impl StoreNew for StoreSetIfNotExistsFloat64 {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetIfNotExistsFloat64 {}

impl StoreSetIfNotExists<f64> for StoreSetIfNotExistsFloat64 {
    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: &f64) {
        state::set_if_not_exists(ord as i64, key, value.to_string().as_bytes());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &f64) {
        let as_str = value.to_string();

        for key in keys {
            state::set_if_not_exists(ord as i64, key, &as_str);
        }
    }
}

/// `StoreSetIfNotExistsProto` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `proto:{your_proto_type}`
#[allow(dead_code)]
pub struct StoreSetIfNotExistsProto<T> {
    store: StoreSetIfNotExistsRaw,
    casper: PhantomData<T>,
}

impl<V: Default + prost::Message> StoreNew for StoreSetIfNotExistsProto<V> {
    fn new() -> Self {
        StoreSetIfNotExistsProto {
            store: StoreSetIfNotExistsRaw {},
            // Adding a PhantomData<T> field to your type tells the compiler that
            // your type acts as though it stores a value of type T, even though
            // it doesn't really. This information is used when computing certain
            // safety properties. For a more in-depth explanation of how to use
            // PhantomData<T>
            casper: PhantomData,
        }
    }
}

impl<V: Default + prost::Message> StoreDelete for StoreSetIfNotExistsProto<V> {}

impl<V: Default + prost::Message> StoreSetIfNotExists<V> for StoreSetIfNotExistsProto<V> {
    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: &V) {
        let bytes = proto::encode(value)
            .unwrap_or_else(|_| panic!("Unable to encode store message's struct to Protobuf data"));

        self.store.set_if_not_exists(ord, key, &bytes)
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &V) {
        let bytes = proto::encode(value)
            .unwrap_or_else(|_| panic!("Unable to encode store message's struct to Protobuf data"));

        for key in keys {
            self.store.set_if_not_exists(ord, key, &bytes)
        }
    }
}

/// `StoreAdd` is a trait which is implemented on any type of types StoreAdd
pub trait StoreAdd<V>: StoreDelete + StoreNew {
    /// Add a given value to an already existing key
    fn add<K: AsRef<str>>(&self, ord: u64, key: K, value: V);
    /// Add multiple values to an already existing key
    fn add_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V);
}

/// `StoreAddInt64` is a struct representing a `store` module with `updatePolicy` equal to `add` and a valueType of `int64`
pub struct StoreAddInt64 {}
impl StoreNew for StoreAddInt64 {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreAddInt64 {}

impl StoreAdd<i64> for StoreAddInt64 {
    fn add<K: AsRef<str>>(&self, ord: u64, key: K, value: i64) {
        state::add_int64(ord as i64, key, value);
    }

    fn add_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: i64) {
        for key in keys {
            state::add_int64(ord as i64, key, value);
        }
    }
}

/// `StoreAddFloat64` is a struct representing a `store` module with `updatePolicy` equal to `add` and a valueType of `float64`
pub struct StoreAddFloat64 {}
impl StoreNew for StoreAddFloat64 {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreAddFloat64 {}

impl StoreAdd<f64> for StoreAddFloat64 {
    fn add<K: AsRef<str>>(&self, ord: u64, key: K, value: f64) {
        state::add_float64(ord as i64, key, value);
    }

    fn add_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: f64) {
        for key in keys {
            state::add_float64(ord as i64, key, value);
        }
    }
}

/// `StoreAddBigDecimal` is a struct representing a `store` module with `updatePolicy` equal to `add` and a valueType of `bigdecimal`
/// `StoreAddBigDecimal` implements `AsRef<BigInt>` to give the client the flexibility
/// to either use the API with &BigDecimal or BigDecimal.
pub struct StoreAddBigDecimal {}
impl StoreNew for StoreAddBigDecimal {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreAddBigDecimal {}

impl<V: AsRef<BigDecimal>> StoreAdd<V> for StoreAddBigDecimal {
    fn add<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::add_bigdecimal(ord as i64, key, value.as_ref());
    }

    fn add_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V) {
        let value = value.as_ref();

        for key in keys {
            state::add_bigdecimal(ord as i64, key, value);
        }
    }
}

/// `StoreAddBigInt` is a struct representing a `store` module with `updatePolicy` equal to `add` and a valueType of `bigint`
/// `StoreAddBigInt` implements `AsRef<BigInt>` to give the client the flexibility
/// to either use the API with &BigInt or BigInt.
pub struct StoreAddBigInt {}
impl StoreNew for StoreAddBigInt {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreAddBigInt {}

impl<V: AsRef<BigInt>> StoreAdd<V> for StoreAddBigInt {
    fn add<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::add_bigint(ord as i64, key, value.as_ref());
    }

    fn add_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V) {
        let value = value.as_ref();

        for key in keys {
            state::add_bigint(ord as i64, key, value);
        }
    }
}

/// `StoreMax` is a trait which is implemented on any type of typed StoreMax
pub trait StoreMax<V>: StoreNew + StoreDelete {
    /// max will set the provided key in the store only if the value received in
    /// parameter is bigger than the one already present in the store, with
    /// a default of the zero value when the key is absent.
    fn max<K: AsRef<str>>(&self, ord: u64, key: K, value: V);
}

/// `StoreMaxInt64` is a struct representing a `store` module with `updatePolicy` equal to `max` and a valueType of `int64`
pub struct StoreMaxInt64 {}
impl StoreNew for StoreMaxInt64 {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreMaxInt64 {}

impl StoreMax<i64> for StoreMaxInt64 {
    fn max<K: AsRef<str>>(&self, ord: u64, key: K, value: i64) {
        state::set_max_int64(ord as i64, key, value);
    }
}

/// `StoreMaxBigInt` is a struct representing a `store` module with `updatePolicy` equal to `max` and a valueType of `bigint`
/// `StoreMaxBigInt` implements `AsRef<BigInt>` to give the client the flexibility
/// to either use the API with &BigInt or BigInt.
pub struct StoreMaxBigInt {}
impl StoreNew for StoreMaxBigInt {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreMaxBigInt {}

impl<V: AsRef<BigInt>> StoreMax<V> for StoreMaxBigInt {
    fn max<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set_max_bigint(ord as i64, key, value.as_ref());
    }
}

/// `StoreMaxFloat64` is a struct representing a `store` module with `updatePolicy` equal to `max` and a valueType of `float64`
pub struct StoreMaxFloat64 {}
impl StoreNew for StoreMaxFloat64 {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreMaxFloat64 {}

impl StoreMax<f64> for StoreMaxFloat64 {
    fn max<K: AsRef<str>>(&self, ord: u64, key: K, value: f64) {
        state::set_max_float64(ord as i64, key, value);
    }
}

/// `StoreMaxBigDecimal` is a struct representing a `store` module with `updatePolicy` equal to `max` and a valueType of `bigdecimal`
/// `StoreMaxBigDecimal` implements `AsRef<BigDecimal>` to give the client the flexibility
/// to either use the API with &BigDecimal or BigDecimal.
pub struct StoreMaxBigDecimal {}
impl StoreNew for StoreMaxBigDecimal {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreMaxBigDecimal {}

impl<V: AsRef<BigDecimal>> StoreMax<V> for StoreMaxBigDecimal {
    fn max<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set_max_bigdecimal(ord as i64, key, value.as_ref());
    }
}

/// `StoreMin` is a trait which is implemented on any typed StoreMin
pub trait StoreMin<V>: StoreNew + StoreDelete {
    /// Will set the provided key in the store only if the value received in
    /// parameter is smaller than the one already present in the store, with
    /// a default of the zero value when the key is absent.
    fn min<K: AsRef<str>>(&self, ord: u64, key: K, value: V);
}

/// `StoreMinInt64` is a struct representing a `store` module with `updatePolicy` equal to `min` and a valueType of `int64`
pub struct StoreMinInt64 {}
impl StoreNew for StoreMinInt64 {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreMinInt64 {}

impl StoreMin<i64> for StoreMinInt64 {
    fn min<K: AsRef<str>>(&self, ord: u64, key: K, value: i64) {
        state::set_min_int64(ord as i64, key, value);
    }
}

/// `StoreMinBigInt` is a struct representing a `store` module with `updatePolicy` equal to `min` and a valueType of `bigint`
/// `StoreMinBigInt` implements `AsRef<BigInt>` to give the client the flexibility
/// to either use the API with &BigInt or BigInt.
pub struct StoreMinBigInt {}
impl StoreNew for StoreMinBigInt {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreMinBigInt {}

impl<V: AsRef<BigInt>> StoreMin<V> for StoreMinBigInt {
    fn min<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set_min_bigint(ord as i64, key, value.as_ref());
    }
}

/// `StoreMinFloat64` is a struct representing a `store` module with `updatePolicy` equal to `min` and a valueType of `float64`
pub struct StoreMinFloat64 {}
impl StoreNew for StoreMinFloat64 {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreMinFloat64 {}

impl StoreMin<f64> for StoreMinFloat64 {
    fn min<K: AsRef<str>>(&self, ord: u64, key: K, value: f64) {
        state::set_min_float64(ord as i64, key, value);
    }
}

/// `StoreMinBigDecimal` is a struct representing a `store` module with `updatePolicy` equal to `min` and a valueType of `bigdecimal`
/// `StoreMinBigDecimal` implements `AsRef<BigDecimal>` to give the client the flexibility to either use
/// the API with &BigDecimal or BigDecimal.
pub struct StoreMinBigDecimal {}
impl StoreNew for StoreMinBigDecimal {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreMinBigDecimal {}

impl<V: AsRef<BigDecimal>> StoreMin<V> for StoreMinBigDecimal {
    fn min<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set_min_bigdecimal(ord as i64, key, value.as_ref());
    }
}

// -------------------- Appender -------------------- //
pub trait Appender<T> {
    fn new() -> Self;
    fn append<K: AsRef<str>>(&self, ord: u64, key: K, item: T);
    fn append_all<K: AsRef<str>>(&self, ord: u64, key: K, items: Vec<T>);
}

/// StoreAppend is a struct representing a `store` with
/// `updatePolicy` equal to `append`
pub struct StoreAppend<T> {
    casper: PhantomData<T>,
}

impl<T> Appender<T> for StoreAppend<T>
where
    T: Into<String>,
{
    fn new() -> Self {
        StoreAppend {
            casper: PhantomData,
        }
    }

    /// Concatenates a given value at the end of the key's current value
    fn append<K: AsRef<str>>(&self, ord: u64, key: K, item: T) {
        let item: String = item.into();
        state::append(ord as i64, &key, &format!("{};", &item).as_bytes());
    }

    fn append_all<K: AsRef<str>>(&self, ord: u64, key: K, items: Vec<T>) {
        for item in items {
            self.append(ord, &key, item);
        }
    }
}

// -------------------- StoreSetSum -------------------- //
pub trait StoreSetSum<T> {
    fn new() -> Self;
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: T);
    fn sum<K: AsRef<str>>(&self, ord: u64, key: K, value: T);
}

pub struct StoreSetSumInt64 {}

impl StoreSetSum<i64> for StoreSetSumInt64 {
    fn new() -> Self {
        StoreSetSumInt64 {}
    }

    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: i64) {
        let v = format!("set:{}", value.to_string());
        state::set_sum_int64(ord as i64, key, v);
    }

    fn sum<K: AsRef<str>>(&self, ord: u64, key: K, value: i64) {
        let v = format!("sum:{}", value.to_string());
        state::set_sum_int64(ord as i64, key, v);
    }
}

pub struct StoreSetSumFloat64 {}

impl StoreSetSum<f64> for StoreSetSumFloat64 {
    fn new() -> Self {
        StoreSetSumFloat64 {}
    }

    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: f64) {
        let v = format!("set:{}", value.to_string());
        state::set_sum_float64(ord as i64, key, v);
    }

    fn sum<K: AsRef<str>>(&self, ord: u64, key: K, value: f64) {
        let v = format!("sum:{}", value.to_string());
        state::set_sum_float64(ord as i64, key, v);
    }
}

pub struct StoreSetSumBigInt {}

impl StoreSetSum<BigInt> for StoreSetSumBigInt {
    fn new() -> Self {
        StoreSetSumBigInt {}
    }

    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: BigInt) {
        let v = format!("set:{}", value.to_string());
        state::set_sum_bigint(ord as i64, key, v);
    }

    fn sum<K: AsRef<str>>(&self, ord: u64, key: K, value: BigInt) {
        let v = format!("sum:{}", value.to_string());
        state::set_sum_bigint(ord as i64, key, v);
    }
}

pub struct StoreSetSumBigDecimal {}

impl StoreSetSum<BigDecimal> for StoreSetSumBigDecimal {
    fn new() -> Self {
        StoreSetSumBigDecimal {}
    }

    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: BigDecimal) {
        let v = format!("set:{}", value.to_string());
        state::set_sum_bigdecimal(ord as i64, key, v);
    }

    fn sum<K: AsRef<str>>(&self, ord: u64, key: K, value: BigDecimal) {
        let v = format!("sum:{}", value.to_string());
        state::set_sum_bigdecimal(ord as i64, key, v);
    }
}

// -------------------- StoreGet -------------------- //
/// StoreGet is a trait which is implemented on any type of typed StoreGet
pub trait StoreGet<T> {
    fn new(idx: u32) -> Self;
    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<T>;
    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<T>;
    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<T>;
    fn has_at<K: AsRef<str>>(&self, ord: u64, key: K) -> bool;
    fn has_last<K: AsRef<str>>(&self, key: K) -> bool;
    fn has_first<K: AsRef<str>>(&self, key: K) -> bool;
}

/// RawStoreGet is a struct representing a read only store `store`
pub struct StoreGetRaw {
    idx: u32,
}

impl StoreGet<Vec<u8>> for StoreGetRaw {
    /// Return a StoreGet object with a store index set
    fn new(idx: u32) -> StoreGetRaw {
        StoreGetRaw { idx }
    }

    /// Allows you to read a single key from the store. The type
    /// of its value can be anything, and is usually declared in
    /// the output section of the manifest. The ordinal is used here
    /// to go query a key that might have changed mid-block by
    /// the store module that built it.
    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<Vec<u8>> {
        state::get_at(self.idx, ord as i64, key)
    }

    /// Retrieves a key from the store, like `get_at`, but querying the state of
    /// the store as of the beginning of the block being processed, before any changes
    /// were applied within the current block. It does not need to rewind any changes
    /// in the middle of the block.
    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<Vec<u8>> {
        state::get_last(self.idx, key)
    }

    /// Retrieves a key from the store, like `get_at`, but querying the state of
    /// the store as of the beginning of the block being processed, before any changes
    /// were applied within the current block. However, it needs to unwind any keys that
    /// would have changed mid-block, so will be slightly less performant.
    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<Vec<u8>> {
        state::get_first(self.idx, key)
    }

    /// Checks if a key exists in the store. The ordinal is used here
    /// to check if a key that might have changed mid-block by
    /// the store module that built it exists.
    fn has_at<K: AsRef<str>>(&self, ord: u64, key: K) -> bool {
        state::has_at(self.idx, ord as i64, key)
    }

    /// Checks if a key exists in the store, like `has_at`, but querying the state of
    /// the store as of the beginning of the block being processed, before any changes
    /// were applied within the current block. It does not need to rewind any changes
    /// in the middle of the block.
    fn has_last<K: AsRef<str>>(&self, key: K) -> bool {
        state::has_last(self.idx, key)
    }

    /// Checks if a key exists in the store, like `has_at`, but querying the state of
    /// the store as of the beginning of the block being processed, before any changes
    /// were applied within the current block. However, it needs to unwind any keys that
    /// would have changed mid-block, so will be slightly less performant.
    fn has_first<K: AsRef<str>>(&self, key: K) -> bool {
        state::has_first(self.idx, key)
    }
}

/// StoreGetString is as struct representing a read only store `store`
pub struct StoreGetString {
    idx: u32,
}

impl StoreGet<String> for StoreGetString {
    fn new(idx: u32) -> Self {
        StoreGetString { idx }
    }

    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<String> {
        let key_ref = key.as_ref();

        state::get_at(self.idx, ord as i64, key_ref).map(|bytes| {
            String::from_utf8(bytes).unwrap_or_else(|_| {
                panic!("Invalid UTF-8 sequence in store value for key: {}", key_ref)
            })
        })
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<String> {
        let key_ref = key.as_ref();

        state::get_last(self.idx, key_ref).map(|bytes| {
            String::from_utf8(bytes).unwrap_or_else(|_| {
                panic!("Invalid UTF-8 sequence in store value for key: {}", key_ref)
            })
        })
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<String> {
        let key_ref = key.as_ref();

        state::get_first(self.idx, key_ref).map(|bytes| {
            String::from_utf8(bytes).unwrap_or_else(|_| {
                panic!("Invalid UTF-8 sequence in store value for key: {}", key_ref)
            })
        })
    }

    fn has_at<K: AsRef<str>>(&self, ord: u64, key: K) -> bool {
        state::has_at(self.idx, ord as i64, key)
    }

    fn has_last<K: AsRef<str>>(&self, key: K) -> bool {
        state::has_last(self.idx, key)
    }

    fn has_first<K: AsRef<str>>(&self, key: K) -> bool {
        state::has_first(self.idx, key)
    }
}

pub struct StoreGetInt64(StoreGetRaw);
impl StoreGet<i64> for StoreGetInt64 {
    fn new(idx: u32) -> Self {
        Self {
            0: StoreGetRaw { idx },
        }
    }

    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<i64> {
        state::get_at(self.0.idx, ord as i64, key)
            .as_ref()
            .map(decode_bytes_to_i64)
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<i64> {
        state::get_last(self.0.idx, key)
            .as_ref()
            .map(decode_bytes_to_i64)
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<i64> {
        state::get_first(self.0.idx, key)
            .as_ref()
            .map(decode_bytes_to_i64)
    }

    fn has_at<K: AsRef<str>>(&self, ord: u64, key: K) -> bool {
        state::has_at(self.0.idx, ord as i64, key)
    }

    fn has_last<K: AsRef<str>>(&self, key: K) -> bool {
        state::has_last(self.0.idx, key)
    }

    fn has_first<K: AsRef<str>>(&self, key: K) -> bool {
        state::has_first(self.0.idx, key)
    }
}

pub struct StoreGetFloat64(StoreGetRaw);
impl StoreGet<f64> for StoreGetFloat64 {
    fn new(idx: u32) -> Self {
        Self {
            0: StoreGetRaw { idx },
        }
    }

    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<f64> {
        state::get_at(self.0.idx, ord as i64, key)
            .as_ref()
            .map(decode_bytes_to_f64)
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<f64> {
        state::get_last(self.0.idx, key)
            .as_ref()
            .map(decode_bytes_to_f64)
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<f64> {
        state::get_first(self.0.idx, key)
            .as_ref()
            .map(decode_bytes_to_f64)
    }

    fn has_at<K: AsRef<str>>(&self, ord: u64, key: K) -> bool {
        state::has_at(self.0.idx, ord as i64, key)
    }

    fn has_last<K: AsRef<str>>(&self, key: K) -> bool {
        state::has_last(self.0.idx, key)
    }

    fn has_first<K: AsRef<str>>(&self, key: K) -> bool {
        state::has_first(self.0.idx, key)
    }
}

pub struct StoreGetBigDecimal(StoreGetRaw);
impl StoreGet<BigDecimal> for StoreGetBigDecimal {
    fn new(idx: u32) -> Self {
        Self {
            0: StoreGetRaw { idx },
        }
    }

    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<BigDecimal> {
        state::get_at(self.0.idx, ord as i64, key).map(|bytes| BigDecimal::from_store_bytes(&bytes))
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<BigDecimal> {
        state::get_last(self.0.idx, key).map(|bytes| BigDecimal::from_store_bytes(&bytes))
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<BigDecimal> {
        state::get_first(self.0.idx, key).map(|bytes| BigDecimal::from_store_bytes(&bytes))
    }

    fn has_at<K: AsRef<str>>(&self, ord: u64, key: K) -> bool {
        state::has_at(self.0.idx, ord as i64, key)
    }

    fn has_last<K: AsRef<str>>(&self, key: K) -> bool {
        state::has_last(self.0.idx, key)
    }

    fn has_first<K: AsRef<str>>(&self, key: K) -> bool {
        state::has_first(self.0.idx, key)
    }
}

pub struct StoreGetBigInt(StoreGetRaw);
impl StoreGet<BigInt> for StoreGetBigInt {
    fn new(idx: u32) -> Self {
        Self {
            0: StoreGetRaw { idx },
        }
    }

    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<BigInt> {
        state::get_at(self.0.idx, ord as i64, key).map(|bytes| BigInt::from_store_bytes(&bytes))
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<BigInt> {
        state::get_last(self.0.idx, key).map(|bytes| BigInt::from_store_bytes(&bytes))
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<BigInt> {
        state::get_first(self.0.idx, key).map(|bytes| BigInt::from_store_bytes(&bytes))
    }

    fn has_at<K: AsRef<str>>(&self, ord: u64, key: K) -> bool {
        state::has_at(self.0.idx, ord as i64, key)
    }

    fn has_last<K: AsRef<str>>(&self, key: K) -> bool {
        state::has_last(self.0.idx, key)
    }

    fn has_first<K: AsRef<str>>(&self, key: K) -> bool {
        state::has_first(self.0.idx, key)
    }
}

#[allow(dead_code)]
pub struct StoreGetArray<T> {
    store: StoreGetRaw,
    casper: PhantomData<T>,
}

impl<T: Into<String> + From<String>> StoreGet<Vec<T>> for StoreGetArray<T> {
    fn new(idx: u32) -> Self {
        Self {
            store: StoreGetRaw { idx },
            casper: PhantomData,
        }
    }

    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<Vec<T>> {
        self.store.get_at(ord, key).and_then(split_array)
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<Vec<T>> {
        self.store.get_last(key).and_then(split_array)
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<Vec<T>> {
        self.store.get_first(key).and_then(split_array)
    }

    fn has_at<K: AsRef<str>>(&self, ord: u64, key: K) -> bool {
        self.store.has_at(ord, key)
    }

    fn has_last<K: AsRef<str>>(&self, key: K) -> bool {
        self.store.has_last(key)
    }

    fn has_first<K: AsRef<str>>(&self, key: K) -> bool {
        self.store.has_first(key)
    }
}

fn split_array<T: Into<String> + From<String>>(bytes: Vec<u8>) -> Option<Vec<T>> {
    let parts = std::io::Cursor::new(bytes).split(b';');
    let chunks: Vec<_> = parts
        .map(|x| x.expect("Cursor is infallible"))
        .filter(|x| x.len() > 0)
        .map(|part| {
            String::from_utf8(part)
                .unwrap_or_else(|_| panic!("Invalid UTF-8 sequence in store value"))
                .into()
        })
        .collect();

    match chunks.len() {
        0 => None,
        _ => Some(chunks),
    }
}

#[allow(dead_code)]
pub struct StoreGetProto<T> {
    store: StoreGetRaw,
    casper: PhantomData<T>,
}

impl<T: Default + prost::Message> StoreGetProto<T> {
    pub fn must_get_last<K: AsRef<str>>(&self, key: K) -> T {
        self.get_last(&key)
            .unwrap_or_else(|| panic!("cannot get_last value: key {} not found", key.as_ref()))
    }
}

impl<T> StoreGet<T> for StoreGetProto<T>
where
    T: Default + prost::Message,
{
    /// Return a StoreGet object with a store index set
    fn new(idx: u32) -> StoreGetProto<T> {
        StoreGetProto {
            store: StoreGetRaw { idx },
            casper: PhantomData,
        }
    }

    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<T> {
        self.store
            .get_at(ord, key)
            .and_then(|bytes| proto::decode::<T>(&bytes).ok())
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<T> {
        self.store
            .get_last(key)
            .and_then(|bytes| proto::decode::<T>(&bytes).ok())
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<T> {
        self.store
            .get_first(key)
            .and_then(|bytes| proto::decode::<T>(&bytes).ok())
    }

    fn has_at<K: AsRef<str>>(&self, ord: u64, key: K) -> bool {
        self.store.has_at(ord, key)
    }

    fn has_last<K: AsRef<str>>(&self, key: K) -> bool {
        self.store.has_last(key)
    }

    fn has_first<K: AsRef<str>>(&self, key: K) -> bool {
        self.store.has_first(key)
    }
}

pub trait Delta: PartialEq {
    fn get_key(&self) -> &String;
    fn get_operation(&self) -> pb::substreams::store_delta::Operation;
}

pub trait DeltaExt: Iterator {
    /// Equivalent to `filter(|x| segment(x.get_key(), index) == value)`.
    fn key_segment_at_eq<S: AsRef<str>>(self, index: usize, value: S) -> key::SegmentAtEq<Self, S>
    where
        Self::Item: Delta,
        Self: Sized,
    {
        key::SegmentAtEq::new(value, Some(index), self)
    }

    /// Equivalent to `filter(|x| first_segment(x.get_key(), index) == value)`.
    fn key_first_segment_eq<S: AsRef<str>>(self, value: S) -> key::SegmentAtEq<Self, S>
    where
        Self::Item: Delta,
        Self: Sized,
    {
        key::SegmentAtEq::new(value, Some(0), self)
    }

    /// Equivalent to `filter(|x| last_segment(x.get_key(), index) == value)`.
    fn key_last_segment_eq<S: AsRef<str>>(self, value: S) -> key::SegmentAtEq<Self, S>
    where
        Self::Item: Delta,
        Self: Sized,
    {
        key::SegmentAtEq::new(value, None, self)
    }

    /// Equivalent to `filter(|x| values.contains(first_segment(x.get_key(), index)))`.
    fn key_first_segment_in<S: AsRef<str>, V: AsRef<[S]>>(
        self,
        values: V,
    ) -> key::SegmentAtIn<Self, S, V>
    where
        Self::Item: Delta,
        Self: Sized,
    {
        key::SegmentAtIn::new(values, Some(0), self)
    }

    /// Equivalent to `filter(|x| values.contains(last_segment(x.get_key(), index)))`.
    fn key_last_segment_in<S: AsRef<str>, V: AsRef<[S]>>(
        self,
        values: V,
    ) -> key::SegmentAtIn<Self, S, V>
    where
        Self::Item: Delta,
        Self: Sized,
    {
        key::SegmentAtIn::new(values, None, self)
    }

    /// Equivalent to `filter(|x| x.get_operation() == operation)`.
    fn operation_eq(self, operation: Operation) -> operation::OperationIs<Self>
    where
        Self::Item: Delta,
        Self: Sized,
    {
        operation::OperationIs::new(operation, false, self)
    }

    /// Equivalent to `filter(|x| x.get_operation() != operation)`.
    fn operation_not_eq(self, operation: Operation) -> operation::OperationIs<Self>
    where
        Self::Item: Delta,
        Self: Sized,
    {
        operation::OperationIs::new(operation, true, self)
    }
}

impl<I: Iterator> DeltaExt for I {}

#[derive(Debug, Clone, PartialEq)]
pub struct Deltas<T: Delta> {
    pub deltas: Vec<T>,
}

impl<T: Delta + From<StoreDelta>> Deltas<T> {
    pub fn new(store_deltas: Vec<StoreDelta>) -> Self {
        Deltas {
            deltas: store_deltas.into_iter().map(Into::into).collect(),
        }
    }

    /// Shortcut for `self.deltas.iter()`.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.deltas.iter()
    }

    /// Shortcut for `self.deltas.into_iter()`.
    pub fn into_iter(self) -> impl Iterator<Item = T> {
        self.deltas.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaBigDecimal {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: BigDecimal,
    pub new_value: BigDecimal,
}

impl From<StoreDelta> for DeltaBigDecimal {
    fn from(d: StoreDelta) -> Self {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal,
            key: d.key,
            old_value: BigDecimal::from_store_bytes(&d.old_value),
            new_value: BigDecimal::from_store_bytes(&d.new_value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaBigInt {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: BigInt,
    pub new_value: BigInt,
}

impl From<StoreDelta> for DeltaBigInt {
    fn from(d: StoreDelta) -> Self {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal,
            key: d.key,
            old_value: BigInt::from_store_bytes(&d.old_value),
            new_value: BigInt::from_store_bytes(&d.new_value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaInt32 {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: i32,
    pub new_value: i32,
}

impl From<StoreDelta> for DeltaInt32 {
    fn from(d: StoreDelta) -> Self {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal,
            key: d.key,
            old_value: decode_bytes_to_i32(&d.old_value),
            new_value: decode_bytes_to_i32(&d.new_value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaInt64 {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: i64,
    pub new_value: i64,
}

impl From<StoreDelta> for DeltaInt64 {
    fn from(d: StoreDelta) -> Self {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal,
            key: d.key,
            old_value: decode_bytes_to_i64(&d.old_value),
            new_value: decode_bytes_to_i64(&d.new_value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaFloat64 {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: f64,
    pub new_value: f64,
}

impl From<StoreDelta> for DeltaFloat64 {
    fn from(d: StoreDelta) -> Self {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal,
            key: d.key,
            old_value: decode_bytes_to_f64(&d.old_value),
            new_value: decode_bytes_to_f64(&d.new_value),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaBool {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: bool,
    pub new_value: bool,
}

impl From<StoreDelta> for DeltaBool {
    fn from(d: StoreDelta) -> Self {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal,
            key: d.key,
            old_value: !d.old_value.contains(&0),
            new_value: !d.new_value.contains(&0),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaBytes {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: Vec<u8>,
    pub new_value: Vec<u8>,
}

impl From<StoreDelta> for DeltaBytes {
    fn from(d: StoreDelta) -> Self {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal,
            key: d.key,
            old_value: d.old_value,
            new_value: d.new_value,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaString {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: String,
    pub new_value: String,
}

impl From<StoreDelta> for DeltaString {
    fn from(d: StoreDelta) -> Self {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal,
            key: d.key,
            old_value: String::from_utf8(d.old_value).unwrap_or_else(|_| {
                panic!("Invalid UTF-8 sequence in Store DeltaString old value")
            }),
            new_value: String::from_utf8(d.new_value).unwrap_or_else(|_| {
                panic!("Invalid UTF-8 sequence in Store DeltaString new value")
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaProto<T> {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: T,
    pub new_value: T,
}

impl<T: Default + prost::Message + PartialEq> From<StoreDelta> for DeltaProto<T> {
    fn from(d: StoreDelta) -> Self {
        let nv: T = prost::Message::decode(d.new_value.as_ref())
            .unwrap_or_else(|_| panic!("Unable to decode Store DeltaProto for new value"));
        let ov: T = prost::Message::decode(d.old_value.as_ref())
            .unwrap_or_else(|_| panic!("Unable to decode Store DeltaProto for old value"));

        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal,
            key: d.key,
            old_value: ov,
            new_value: nv,
        }
    }
}

impl<T: Default + prost::Message + PartialEq> Delta for DeltaProto<T> {
    fn get_key(&self) -> &String {
        &self.key
    }
    fn get_operation(&self) -> pb::substreams::store_delta::Operation {
        return self.operation;
    }
}

impl<T: Default + prost::Message + PartialEq> Delta for &DeltaProto<T> {
    fn get_key(&self) -> &String {
        &self.key
    }
    fn get_operation(&self) -> pb::substreams::store_delta::Operation {
        return self.operation;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeltaArray<T> {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: Vec<T>,
    pub new_value: Vec<T>,
}

impl<T: Into<String> + From<String> + PartialEq> From<StoreDelta> for DeltaArray<T> {
    fn from(d: StoreDelta) -> Self {
        let old = split_array::<T>(d.old_value).unwrap_or_default();
        let new = split_array::<T>(d.new_value).unwrap_or_default();

        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal,
            key: d.key,
            old_value: old,
            new_value: new,
        }
    }
}

impl<T: Into<String> + From<String> + PartialEq> Delta for DeltaArray<T> {
    fn get_key(&self) -> &String {
        &self.key
    }
    fn get_operation(&self) -> pb::substreams::store_delta::Operation {
        return self.operation;
    }
}

impl<T: Into<String> + From<String> + PartialEq> Delta for &DeltaArray<T> {
    fn get_key(&self) -> &String {
        &self.key
    }
    fn get_operation(&self) -> pb::substreams::store_delta::Operation {
        return self.operation;
    }
}

macro_rules! impl_delta_ref {
    ($name:ty) => {
        impl Delta for $name {
            fn get_key(&self) -> &String {
                &self.key
            }
            fn get_operation(&self) -> pb::substreams::store_delta::Operation {
                self.operation
            }
        }
    };
}

macro_rules! impl_delta {
    ($name:ty) => {
        impl Delta for $name {
            fn get_key(&self) -> &String {
                &self.key
            }
            fn get_operation(&self) -> pb::substreams::store_delta::Operation {
                self.operation
            }
        }
        impl $name {
            pub fn get_key(&self) -> &String {
                &self.key
            }
            pub fn get_operation(&self) -> pb::substreams::store_delta::Operation {
                self.operation
            }
        }
    };
}

impl_delta!(DeltaBigDecimal);
impl_delta!(DeltaBigInt);
impl_delta!(DeltaInt32);
impl_delta!(DeltaInt64);
impl_delta!(DeltaFloat64);
impl_delta!(DeltaBool);
impl_delta!(DeltaBytes);
impl_delta!(DeltaString);

impl_delta_ref!(&DeltaBigDecimal);
impl_delta_ref!(&DeltaBigInt);
impl_delta_ref!(&DeltaInt32);
impl_delta_ref!(&DeltaInt64);
impl_delta_ref!(&DeltaFloat64);
impl_delta_ref!(&DeltaBool);
impl_delta_ref!(&DeltaBytes);
impl_delta_ref!(&DeltaString);

fn convert_i32_to_operation(operation: i32) -> pb::substreams::store_delta::Operation {
    Operation::from_i32(operation).unwrap_or_else(|| panic!("unhandled operation: {}", operation))
}

// We accept &Vec<u8> instead of &[u8] because use internally and makes it easier to chain
fn decode_bytes_to_i32(bytes: &Vec<u8>) -> i32 {
    if bytes.is_empty() {
        return 0;
    }

    // FIXME: If we are ready to accept the fact that `bytes` is always valid UTF-8, we could even use
    //        the unsafe `from_utf8_unchecked` version, we would need first to measure the impact and
    //        better understand implication of an invalid UTF-8 &str with `from_str` call.
    let int_as_str =
        std::str::from_utf8(bytes).expect("received bytes expected to be valid UTF-8 string");

    i32::from_str(int_as_str).unwrap_or_else(|_| {
        panic!(
            "value {} is not a valid representation of an i32",
            int_as_str
        )
    })
}

// We accept &Vec<u8> instead of &[u8] because use internally and makes it easier to chain
fn decode_bytes_to_i64(bytes: &Vec<u8>) -> i64 {
    if bytes.is_empty() {
        return 0;
    }

    // FIXME: If we are ready to accept the fact that `bytes` is always valid UTF-8, we could even use
    //        the unsafe `from_utf8_unchecked` version, we would need first to measure the impact and
    //        better understand implication of an invalid UTF-8 &str with `from_str` call.
    let int_as_str =
        std::str::from_utf8(bytes).expect("received bytes expected to be valid UTF-8 string");

    i64::from_str(int_as_str).unwrap_or_else(|_| {
        panic!(
            "value {} is not a valid representation of an i64",
            int_as_str
        )
    })
}

// We accept &Vec<u8> instead of &[u8] because use internally and makes it easier to chain
fn decode_bytes_to_f64(bytes: &Vec<u8>) -> f64 {
    if bytes.is_empty() {
        return 0.0;
    }

    // FIXME: If we are ready to accept the fact that `bytes` is always valid UTF-8, we could even use
    //        the unsafe `from_utf8_unchecked` version, we would need first to measure the impact and
    //        better understand implication of an invalid UTF-8 &str with `from_str` call.
    let float64_as_str =
        std::str::from_utf8(bytes).expect("received bytes expected to be valid UTF-8 string");

    f64::from_str(float64_as_str).unwrap_or_else(|_| {
        panic!(
            "value {} is not a valid representation of an f64",
            float64_as_str
        )
    })
}

#[cfg(test)]
mod tests {
    use crate::{
        pb::substreams::{store_delta::Operation, StoreDelta},
        store::{
            decode_bytes_to_f64, decode_bytes_to_i32, decode_bytes_to_i64, split_array, DeltaArray,
            Deltas,
        },
    };

    #[test]
    fn valid_int64_decode_bytes_to_i32() {
        let bytes: Vec<u8> = "1".as_bytes().to_vec();
        assert_eq!(1, decode_bytes_to_i32(&bytes))
    }

    #[test]
    fn valid_int64_max_value_decode_bytes_to_i32() {
        let bytes: Vec<u8> = i32::MAX.to_string().as_bytes().to_vec();
        assert_eq!(i32::MAX, decode_bytes_to_i32(&bytes))
    }

    #[test]
    #[should_panic]
    fn invalid_bytes_decode_bytes_to_i32() {
        let bytes: Vec<u8> = "invalid".as_bytes().to_vec();
        decode_bytes_to_i32(&bytes);
    }

    #[test]
    fn no_bytes_decode_bytes_to_i32() {
        let bytes: Vec<u8> = vec![];
        decode_bytes_to_i32(&bytes);
    }

    #[test]
    fn valid_int64_decode_bytes_to_i64() {
        let bytes: Vec<u8> = "1".as_bytes().to_vec();
        assert_eq!(1, decode_bytes_to_i64(&bytes))
    }

    #[test]
    fn valid_int64_max_value_decode_bytes_to_i64() {
        let bytes: Vec<u8> = i64::MAX.to_string().as_bytes().to_vec();
        assert_eq!(i64::MAX, decode_bytes_to_i64(&bytes))
    }

    #[test]
    #[should_panic]
    fn invalid_bytes_decode_bytes_to_i64() {
        let bytes: Vec<u8> = "invalid".as_bytes().to_vec();
        decode_bytes_to_i64(&bytes);
    }

    #[test]
    fn no_bytes_decode_bytes_to_i64() {
        let bytes: Vec<u8> = vec![];
        decode_bytes_to_i64(&bytes);
    }

    #[test]
    fn valid_f64_decode_bytes_to_f64() {
        let bytes: Vec<u8> = "1.00".as_bytes().to_vec();
        assert_eq!(1.00, decode_bytes_to_f64(&bytes))
    }

    #[test]
    fn valid_f64_max_value_decode_bytes_to_f64() {
        let bytes: Vec<u8> = f64::MAX.to_string().as_bytes().to_vec();
        assert_eq!(f64::MAX, decode_bytes_to_f64(&bytes))
    }

    #[test]
    #[should_panic]
    fn invalid_bytes_decode_bytes_to_f64() {
        let bytes: Vec<u8> = "invalid".as_bytes().to_vec();
        decode_bytes_to_f64(&bytes);
    }

    #[test]
    fn no_bytes_decode_bytes_to_f64() {
        let bytes: Vec<u8> = vec![];
        decode_bytes_to_f64(&bytes);
    }

    #[test]
    fn delta_array_strring() {
        let deltas = Deltas::<DeltaArray<String>>::new(vec![StoreDelta {
            operation: 1,
            ordinal: 0,
            key: "".to_string(),
            old_value: ";".as_bytes().to_vec(),
            new_value: "1.1;2.2;3.3;".as_bytes().to_vec(),
        }]);

        assert_eq!(
            Deltas::<DeltaArray<String>> {
                deltas: vec![DeltaArray::<String> {
                    operation: Operation::Create,
                    ordinal: 0,
                    key: "".to_string(),
                    old_value: vec![],
                    new_value: vec!["1.1".to_string(), "2.2".to_string(), "3.3".to_string(),]
                }]
            },
            deltas
        );
    }

    #[test]
    fn split_arrays_no_elements() {
        let value = "";
        let bytes = value.as_bytes();

        let expected_value = None;
        let actual_value = split_array::<String>(bytes.to_vec());

        assert_eq!(expected_value, actual_value)
    }

    #[test]
    fn split_arrays_one_string_element() {
        let value = "1;";
        let bytes = value.as_bytes();

        let expected_value = Some(vec!["1".to_string()]);
        let actual_value = split_array::<String>(bytes.to_vec());

        assert_eq!(expected_value, actual_value)
    }

    #[test]
    fn split_arrays_multiple_string_elements() {
        let value = "1;2;3;";
        let bytes = value.as_bytes();

        let expected_value = Some(vec!["1".to_string(), "2".to_string(), "3".to_string()]);
        let actual_value = split_array::<String>(bytes.to_vec());

        assert_eq!(expected_value, actual_value)
    }
}
