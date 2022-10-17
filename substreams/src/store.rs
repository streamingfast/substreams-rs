//! Store Implementation for Substreams.
//!
//! This crate implements the different Stores which can be used in your Substreams
//! handlers.
//!

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

/// `StoreSetRaw` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `string`
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
///     `StoreSetString` implements AsRef<str> to give the client the flexibility
///     to either use the API with &String or String.
pub struct StoreSetString {}
impl StoreNew for StoreSetString {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetString {}

impl<V: AsRef<str>> StoreSet<V> for StoreSetString {
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &V) {
        state::set(ord as i64, key, value.as_ref().as_bytes());
    }

    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &V) {
        for key in keys {
            state::set(ord as i64, key, value.as_ref().as_bytes());
        }
    }
}

/// `StoreSetI64` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `int64`
pub struct StoreSetI64 {}
impl StoreNew for StoreSetI64 {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetI64 {}

impl StoreSet<i64> for StoreSetI64 {
    /// Set a given key to a given value, if the key existed before, it will be replaced.
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &i64) {
        state::set(ord as i64, key, value.to_string().as_bytes());
    }

    /// Set many keys to a given values, if the key existed before, it will be replaced.
    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &i64) {
        for key in keys {
            state::set(ord as i64, key, value.to_string().as_bytes());
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
        for key in keys {
            state::set(ord as i64, key, value.to_string().as_bytes());
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
        for key in keys {
            state::set(ord as i64, key, value.to_string().as_bytes())
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
        for key in keys {
            state::set(ord as i64, key, value.as_ref().to_string().as_bytes());
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
        match proto::encode(value) {
            Ok(bytes) => state::set(ord as i64, key, &bytes),
            Err(_) => panic!("failed to encode message"),
        }
    }

    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &V) {
        for key in keys {
            match proto::encode(value) {
                Ok(bytes) => state::set(ord as i64, key, &bytes),
                Err(_) => panic!("failed to encode message"),
            }
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
        for key in keys {
            state::set_if_not_exists(ord as i64, key, value.as_ref());
        }
    }
}

/// `StoreSetIfNotExistsString` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `string`
///     `StoreSetIfNotExistsString` implements AsRef<str> to give the client the flexibility
///     to either use the API with &String or String.
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
        for key in keys {
            state::set_if_not_exists(ord as i64, key, value.as_ref().as_bytes());
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
        for key in keys {
            state::set_if_not_exists(ord as i64, key, value.as_ref().to_string().as_bytes());
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
        for key in keys {
            state::set_if_not_exists(ord as i64, key, value.to_string().as_bytes());
        }
    }
}

/// `StoreSetIfNotExistsI64` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `int64`
pub struct StoreSetIfNotExistsI64 {}
impl StoreNew for StoreSetIfNotExistsI64 {
    fn new() -> Self {
        Self {}
    }
}

impl StoreDelete for StoreSetIfNotExistsI64 {}

impl StoreSetIfNotExists<i64> for StoreSetIfNotExistsI64 {
    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: &i64) {
        state::set_if_not_exists(ord as i64, key, value.to_string().as_bytes());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &i64) {
        for key in keys {
            state::set_if_not_exists(ord as i64, key, value.to_string().as_bytes());
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
        for key in keys {
            state::set_if_not_exists(ord as i64, key, value.to_string().as_bytes());
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
        match proto::encode(value) {
            Ok(bytes) => self.store.set_if_not_exists(ord, key, &bytes),
            Err(_) => panic!("failed to encode message"),
        }
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &V) {
        for key in keys {
            match proto::encode(value) {
                Ok(bytes) => self.store.set_if_not_exists(ord, key, &bytes),
                Err(_) => panic!("failed to encode message"),
            }
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
///     `StoreAddBigDecimal` implements AsRef<BigInt> to give the client the flexibility
///     to either use the API with &BigDecimal or BigDecimal.
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
        for key in keys {
            state::add_bigdecimal(ord as i64, key, value.as_ref());
        }
    }
}

/// `StoreAddBigInt` is a struct representing a `store` module with `updatePolicy` equal to `add` and a valueType of `bigint`
///     `StoreAddBigInt` implements AsRef<BigInt> to give the client the flexibility
///     to either use the API with &BigInt or BigInt.
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
        for key in keys {
            state::add_bigint(ord as i64, key, value.as_ref());
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
///     `StoreMaxBigInt` implements AsRef<BigInt> to give the client the flexibility
///     to either use the API with &BigInt or BigInt.
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
///     `StoreMaxBigDecimal` implements AsRef<BigDecimal> to give the client the flexibility
///     to either use the API with &BigDecimal or BigDecimal.
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
///     `StoreMinBigInt` implements AsRef<BigInt> to give the client the flexibility
///     to either use the API with &BigInt or BigInt.
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
///     `StoreMinBigDecimal` implements AsRef<BigDecimal> to give the client the flexibility
///     to either use the API with &BigDecimal or BigDecimal.
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
        state::append(ord as i64, &key, &format!("{};", &item).as_bytes().to_vec());
    }

    fn append_all<K: AsRef<str>>(&self, ord: u64, key: K, items: Vec<T>) {
        for item in items {
            self.append(ord, &key, item);
        }
    }
}

// -------------------- StoreGet -------------------- //
/// StoreGet is a trait which is implemented on any type of typed StoreGet
pub trait StoreGet<T> {
    fn new(idx: u32) -> Self;
    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<T>;
    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<T>;
    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<T>;
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
        return state::get_at(self.idx, ord as i64, key);
    }

    /// Retrieves a key from the store, like `get_at`, but querying the state of
    /// the store as of the beginning of the block being processed, before any changes
    /// were applied within the current block. Tt does not need to rewind any changes
    /// in the middle of the block.
    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<Vec<u8>> {
        return state::get_last(self.idx, key);
    }

    /// Retrieves a key from the store, like `get_at`, but querying the state of
    /// the store as of the beginning of the block being processed, before any changes
    /// were applied within the current block. However, it needs to unwind any keys that
    /// would have changed mid-block, so will be slightly less performant.
    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<Vec<u8>> {
        return state::get_first(self.idx, key);
    }
}

pub struct StoreGetI64(StoreGetRaw);
impl StoreGet<i64> for StoreGetI64 {
    fn new(idx: u32) -> Self {
        Self {
            0: StoreGetRaw { idx },
        }
    }

    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<i64> {
        let value = state::get_at(self.0.idx, ord as i64, key);
        return match value {
            None => None,
            Some(bytes) => decode_bytes_to_i64(bytes),
        };
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<i64> {
        let value = state::get_last(self.0.idx, key);
        return match value {
            None => None,
            Some(bytes) => decode_bytes_to_i64(bytes),
        };
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<i64> {
        let value = state::get_first(self.0.idx, key);
        return match value {
            None => None,
            Some(bytes) => decode_bytes_to_i64(bytes),
        };
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
        let value = state::get_at(self.0.idx, ord as i64, key);
        return match value {
            None => None,
            Some(bytes) => decode_bytes_to_f64(bytes),
        };
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<f64> {
        let value = state::get_last(self.0.idx, key);
        return match value {
            None => None,
            Some(bytes) => decode_bytes_to_f64(bytes),
        };
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<f64> {
        let value = state::get_first(self.0.idx, key);
        return match value {
            None => None,
            Some(bytes) => decode_bytes_to_f64(bytes),
        };
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
        let bytes_option: Option<Vec<u8>> = state::get_at(self.0.idx, ord as i64, key);
        match bytes_option {
            None => None,
            Some(bytes) => Some(BigDecimal::from_store_bytes(bytes)),
        }
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<BigDecimal> {
        let bytes_option: Option<Vec<u8>> = state::get_last(self.0.idx, key);
        match bytes_option {
            None => None,
            Some(bytes) => Some(BigDecimal::from_store_bytes(bytes)),
        }
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<BigDecimal> {
        let bytes_option: Option<Vec<u8>> = state::get_first(self.0.idx, key);
        match bytes_option {
            None => None,
            Some(bytes) => Some(BigDecimal::from_store_bytes(bytes)),
        }
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
        let store_bytes: Option<Vec<u8>> = state::get_at(self.0.idx, ord as i64, key);
        match store_bytes {
            None => None,
            Some(bytes) => Some(BigInt::from_store_bytes(bytes)),
        }
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<BigInt> {
        let store_bytes: Option<Vec<u8>> = state::get_last(self.0.idx, key);
        match store_bytes {
            None => None,
            Some(bytes) => Some(BigInt::from_store_bytes(bytes)),
        }
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<BigInt> {
        let store_bytes: Option<Vec<u8>> = state::get_first(self.0.idx, key);
        match store_bytes {
            None => None,
            Some(bytes) => Some(BigInt::from_store_bytes(bytes)),
        }
    }
}

#[allow(dead_code)]
pub struct StoreGetProto<T> {
    store: StoreGetRaw,
    casper: PhantomData<T>,
}

impl<T: Default + prost::Message> StoreGetProto<T> {
    pub fn must_get_last<K: AsRef<str>>(&self, key: K) -> T {
        match self.get_last(key.as_ref().clone()) {
            None => {
                panic!("pool does not exist skipping pool {:?}", &key.as_ref());
            }
            Some(value) => value,
        }
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
        match self.store.get_at(ord, key) {
            None => None,
            Some(bytes) => {
                let value: Result<T, prost::DecodeError> = proto::decode(&bytes);
                match value {
                    Ok(_) => Some(value.unwrap()),
                    Err(_) => None,
                }
            }
        }
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<T> {
        match self.store.get_last(key) {
            None => None,
            Some(bytes) => {
                let value: Result<T, prost::DecodeError> = proto::decode(&bytes);
                match value {
                    Ok(_) => Some(value.unwrap()),
                    Err(_) => None,
                }
            }
        }
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<T> {
        match self.store.get_first(key) {
            None => None,
            Some(bytes) => {
                let value: Result<T, prost::DecodeError> = proto::decode(&bytes);
                match value {
                    Ok(_) => Some(value.unwrap()),
                    Err(_) => None,
                }
            }
        }
    }
}

pub trait Delta {
    fn new(d: &StoreDelta) -> Self;
}

pub struct Deltas<T> {
    pub deltas: Vec<T>,
}

impl<T: Delta> Deltas<T> {
    pub fn new(store_deltas: Vec<StoreDelta>) -> Self {
        let mut deltas = Deltas { deltas: vec![] };

        for d in store_deltas.iter() {
            deltas.deltas.push(T::new(d))
        }

        deltas
    }
}

pub trait DeltaDecoder<T> {
    fn decode(d: &StoreDelta) -> T;
}

#[derive(Debug)]
pub struct DeltaBigDecimal {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: BigDecimal,
    pub new_value: BigDecimal,
}

impl Delta for DeltaBigDecimal {
    fn new(d: &StoreDelta) -> Self {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal.clone(),
            key: d.key.clone(),
            old_value: BigDecimal::from_store_bytes(d.old_value.clone()),
            new_value: BigDecimal::from_store_bytes(d.new_value.clone()),
        }
    }
}

#[derive(Debug)]
pub struct DeltaBigInt {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: BigInt,
    pub new_value: BigInt,
}

impl Delta for DeltaBigInt {
    fn new(d: &StoreDelta) -> Self {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal.clone(),
            key: d.key.clone(),
            old_value: BigInt::from_store_bytes(d.old_value.clone()),
            new_value: BigInt::from_store_bytes(d.new_value.clone()),
        }
    }
}

#[derive(Debug)]
pub struct DeltaI64 {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: i64,
    pub new_value: i64,
}

impl Delta for DeltaI64 {
    fn new(d: &StoreDelta) -> DeltaI64 {
        let mut ov = i64::default();
        if d.old_value.len() != 0 {
            ov = match decode_bytes_to_i64(d.old_value.clone()) {
                None => 0,
                Some(value) => value,
            };
        }
        let mut nv = i64::default();
        if d.new_value.len() != 0 {
            nv = match decode_bytes_to_i64(d.new_value.clone()) {
                None => 0,
                Some(value) => value,
            };
        }

        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal.clone(),
            key: d.key.clone(),
            old_value: ov,
            new_value: nv,
        }
    }
}

#[derive(Debug)]
pub struct DeltaFloat64 {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: f64,
    pub new_value: f64,
}

impl Delta for DeltaFloat64 {
    fn new(d: &StoreDelta) -> DeltaFloat64 {
        let mut ov = f64::default();
        if d.old_value.len() != 0 {
            ov = match decode_bytes_to_f64(d.old_value.clone()) {
                None => 0 as f64,
                Some(value) => value,
            };
        }
        let mut nv = f64::default();
        if d.new_value.len() != 0 {
            nv = match decode_bytes_to_f64(d.new_value.clone()) {
                None => 0 as f64,
                Some(value) => value,
            };
        }

        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal.clone(),
            key: d.key.clone(),
            old_value: ov,
            new_value: nv,
        }
    }
}

#[derive(Debug)]
pub struct DeltaString {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: String,
    pub new_value: String,
}

impl Delta for DeltaString {
    fn new(d: &StoreDelta) -> DeltaString {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal.clone(),
            key: d.key.clone(),
            old_value: String::from_utf8(d.old_value.clone()).unwrap(),
            new_value: String::from_utf8(d.new_value.clone()).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct DeltaProto<T> {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: T,
    pub new_value: T,
}

impl<T> Delta for DeltaProto<T>
where
    T: Default + prost::Message,
{
    fn new(d: &StoreDelta) -> Self {
        let nv: T = prost::Message::decode(&d.new_value[..]).unwrap();
        let ov: T = prost::Message::decode(&d.old_value[..]).unwrap();
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal.clone(),
            key: d.key.clone(),
            old_value: ov,
            new_value: nv,
        }
    }
}

#[derive(Debug)]
pub struct DeltaArray<T> {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: Vec<T>,
    pub new_value: Vec<T>,
}

impl<T> Delta for DeltaArray<T>
where
    T: Into<String> + From<String>,
{
    fn new(d: &StoreDelta) -> Self {
        let old_chunks = String::from_utf8(d.old_value.clone()).unwrap();
        let mut old_values: Vec<T> = old_chunks
            .split(";")
            .map(|v| v.to_string())
            .map(|v| v.into())
            .collect();

        let new_string = String::from_utf8(d.new_value.clone()).unwrap();
        let mut new_values: Vec<T> = new_string
            .split(";")
            .map(|v| v.to_string())
            .map(|v| v.into())
            .collect();

        // remove last element which is a blank one, since there is always a ;
        old_values.pop();
        new_values.pop();

        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal.clone(),
            key: d.key.clone(),
            old_value: old_values,
            new_value: new_values,
        }
    }
}

fn convert_i32_to_operation(operation: i32) -> pb::substreams::store_delta::Operation {
    return match operation {
        x if x == pb::substreams::store_delta::Operation::Unset as i32 => {
            pb::substreams::store_delta::Operation::Unset
        }
        x if x == pb::substreams::store_delta::Operation::Create as i32 => {
            pb::substreams::store_delta::Operation::Create
        }
        x if x == pb::substreams::store_delta::Operation::Update as i32 => {
            pb::substreams::store_delta::Operation::Update
        }
        x if x == pb::substreams::store_delta::Operation::Delete as i32 => {
            pb::substreams::store_delta::Operation::Delete
        }
        _ => panic!("unhandled operation: {}", operation),
    };
}

fn decode_bytes_to_i64(bytes: Vec<u8>) -> Option<i64> {
    let int_as_string = String::from_utf8_lossy(&bytes.as_slice()).to_string();
    return match i64::from_str(int_as_string.as_str()) {
        Ok(value) => Some(value),
        Err(_) => panic!(
            "value {} is not a valid representation of an i64",
            int_as_string
        ),
    };
}

fn decode_bytes_to_f64(bytes: Vec<u8>) -> Option<f64> {
    let float64_as_string = String::from_utf8_lossy(&bytes.as_slice()).to_string();
    return match f64::from_str(float64_as_string.as_str()) {
        Ok(value) => Some(value),
        Err(_) => panic!(
            "value {} is not a valid representation of an f64",
            float64_as_string
        ),
    };
}

#[cfg(test)]
mod test {
    use crate::store::{decode_bytes_to_f64, decode_bytes_to_i64};

    #[test]
    fn valid_int64_decode_bytes_to_i64() {
        let bytes: Vec<u8> = Vec::from("1".to_string());
        assert_eq!(1, decode_bytes_to_i64(bytes).unwrap())
    }

    #[test]
    fn valid_int64_max_value_decode_bytes_to_i64() {
        let bytes: Vec<u8> = Vec::from(i64::MAX.to_string());
        assert_eq!(i64::MAX, decode_bytes_to_i64(bytes).unwrap())
    }

    #[test]
    #[should_panic]
    fn invalid_bytes_decode_bytes_to_i64() {
        let bytes: Vec<u8> = Vec::from("invalid".to_string());
        decode_bytes_to_i64(bytes);
    }

    #[test]
    #[should_panic]
    fn no_bytes_decode_bytes_to_i64() {
        let bytes: Vec<u8> = vec![];
        decode_bytes_to_i64(bytes);
    }

    #[test]
    fn valid_f64_decode_bytes_to_f64() {
        let bytes: Vec<u8> = Vec::from("1.00".to_string());
        assert_eq!(1.00, decode_bytes_to_f64(bytes).unwrap())
    }

    #[test]
    fn valid_f64_max_value_decode_bytes_to_f64() {
        let bytes: Vec<u8> = Vec::from(f64::MAX.to_string());
        assert_eq!(f64::MAX, decode_bytes_to_f64(bytes).unwrap())
    }

    #[test]
    #[should_panic]
    fn invalid_bytes_decode_bytes_to_f64() {
        let bytes: Vec<u8> = Vec::from("invalid".to_string());
        decode_bytes_to_f64(bytes);
    }

    #[test]
    #[should_panic]
    fn no_bytes_decode_bytes_to_f64() {
        let bytes: Vec<u8> = vec![];
        decode_bytes_to_f64(bytes);
    }
}
