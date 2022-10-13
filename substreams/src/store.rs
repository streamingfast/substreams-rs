//! Store Implementation for Substreams.
//!
//! This crate implements the different Stores which can be used in your Substreams
//! handlers.
//!

use {
    crate::{
        pb::substreams::StoreDelta,
        scalar::{BigFloat, BigInt},
        state, {pb, proto},
    },
    prost,
    std::i64,
    std::marker::PhantomData,
    std::str::FromStr,
    substreams_macro::StoreWriter,
};

// todo: add delete prefix on all the stores

/// `StoreSet` is a trait which is implemented on any type of typed StoreSet
pub trait StoreSet<V> {
    fn new() -> Self;
    /// Set a given key to a given value, if the key existed before, it will be replaced.  
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: V);
    /// Set many keys to a given values, if the key existed before, it will be replaced.
    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V);
}

/// `StoreSetRaw` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `string`
///     `StoreSetRaw` implements AsRef<[u8]> to give the client the flexibility
///     to either use the API with &Vec[...] or Vec[...].
#[derive(StoreWriter)]
pub struct StoreSetRaw {}
impl<V: AsRef<[u8]>> StoreSet<V> for StoreSetRaw {
    fn new() -> Self {
        Self {}
    }

    /// Set a given key to a given value, if the key existed before, it will be replaced.
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set(ord as i64, key, value.as_ref());
    }

    /// Set many keys to a given values, if the key existed before, it will be replaced.
    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V) {
        for key in keys {
            state::set(ord as i64, key, value.as_ref());
        }
    }
}

/// `StoreSetString` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `string`
///     `StoreSetString` implements AsRef<str> to give the client the flexibility
///     to either use the API with &String or String.
#[derive(StoreWriter)]
pub struct StoreSetString {}
impl<V: AsRef<str>> StoreSet<V> for StoreSetString {
    fn new() -> Self {
        Self {}
    }

    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set(ord as i64, key, value.as_ref().as_bytes());
    }

    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V) {
        for key in keys {
            state::set(ord as i64, key, value.as_ref().as_bytes());
        }
    }
}

/// `StoreSetI64` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `int64`
///     `StoreSetI64` implements AsRef<i64> to give the client the flexibility
///     to either use the API with &i64 or i64.
#[derive(StoreWriter)]
pub struct StoreSetI64 {}
impl StoreSet<i64> for StoreSetI64 {
    fn new() -> Self {
        Self {}
    }

    /// Set a given key to a given value, if the key existed before, it will be replaced.
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: i64) {
        state::set(ord as i64, key, value.to_string().as_bytes());
    }

    /// Set many keys to a given values, if the key existed before, it will be replaced.
    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: i64) {
        for key in keys {
            state::set(ord as i64, key, value.to_string().as_bytes());
        }
    }
}

/// `StoreSetFloat64` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `int64`
///     `StoreSetFloat64` implements AsRef<i64> to give the client the flexibility
///     to either use the API with &f64 or f64.
#[derive(StoreWriter)]
pub struct StoreSetFloat64 {}
impl StoreSet<f64> for StoreSetFloat64 {
    fn new() -> Self {
        Self {}
    }

    /// Set a given key to a given value, if the key existed before, it will be replaced.
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: f64) {
        state::set(ord as i64, key, value.to_string().as_bytes());
    }

    /// Set many keys to a given values, if the key existed before, it will be replaced.
    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: f64) {
        for key in keys {
            state::set(ord as i64, key, value.to_string().as_bytes());
        }
    }
}

/// `StoreSetBigFloat` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `bigfloat`
///     `StoreSetBigFloat` implements AsRef<BigFloat> to give the client the flexibility
///     to either use the API with &BigFloat or BigFloat.
#[derive(StoreWriter)]
pub struct StoreSetBigFloat {}
impl<V: AsRef<BigFloat>> StoreSet<V> for StoreSetBigFloat {
    fn new() -> Self {
        Self {}
    }

    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set(ord as i64, key, value.as_ref().to_string().as_bytes())
    }

    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V) {
        for key in keys {
            state::set(ord as i64, key, value.as_ref().to_string().as_bytes())
        }
    }
}

/// `StoreSetBigInt` is a struct representing a `store` with `updatePolicy` equal to `set` on a `valueType` equal to `bigint`
///     `StoreSetBigInt` implements AsRef<BigInt> to give the client the flexibility
///     to either use the API with &BigInt or BigInt.
#[derive(StoreWriter)]
pub struct StoreSetBigInt {}
impl<V: AsRef<BigInt>> StoreSet<V> for StoreSetBigInt {
    fn new() -> Self {
        Self {}
    }

    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set(ord as i64, key, value.as_ref().to_string().as_bytes());
    }

    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V) {
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

impl<V: Default + prost::Message> StoreSetProto<V> {
    pub fn new() -> Self {
        Self {
            // Adding a PhantomData<T> field to your type tells the compiler that
            // your type acts as though it stores a value of type T, even though
            // it doesn't really. This information is used when computing certain
            // safety properties. For a more in-depth explanation of how to use
            // PhantomData<T>
            casper: PhantomData,
        }
    }

    pub fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &V) {
        match proto::encode(value) {
            Ok(bytes) => state::set(ord as i64, key, &bytes),
            Err(_) => panic!("failed to encode message"),
        }
    }

    pub fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &V) {
        for key in keys {
            match proto::encode(value) {
                Ok(bytes) => state::set(ord as i64, key, &bytes),
                Err(_) => panic!("failed to encode message"),
            }
        }
    }
}

/// `StoreSetIfNotExists` is a struct for which other structs
pub trait StoreSetIfNotExists<V> {
    /// Initializes a new StoreSetIfNotExists
    fn new() -> Self;
    /// Set a given key to a given value, if the key existed before, it will be ignored and not set.  
    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: V);
    /// Set given keys to given values, if the key existed before, it will be ignored and not set.
    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V);
}

/// `StoreSetIfNotExistsRaw` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `String`
///     `StoreSetIfNotExistsRaw` implements AsRef<[u8]> to give the client the flexibility
///     to either use the API with &Vec[...] or Vec[...].
#[derive(StoreWriter)]
pub struct StoreSetIfNotExistsRaw {}
impl<V: AsRef<[u8]>> StoreSetIfNotExists<V> for StoreSetIfNotExistsRaw {
    fn new() -> Self {
        Self {}
    }

    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set_if_not_exists(ord as i64, key, value.as_ref());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V) {
        for key in keys {
            state::set_if_not_exists(ord as i64, key, value.as_ref());
        }
    }
}

/// `StoreSetIfNotExistsString` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `String`
///     `StoreSetIfNotExistsString` implements AsRef<str> to give the client the flexibility
///     to either use the API with &String or String.
#[derive(StoreWriter)]
pub struct StoreSetIfNotExistsString {}
impl<V: AsRef<str>> StoreSetIfNotExists<V> for StoreSetIfNotExistsString {
    fn new() -> Self {
        Self {}
    }

    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set_if_not_exists(ord as i64, key, value.as_ref().as_bytes());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V) {
        for key in keys {
            state::set_if_not_exists(ord as i64, key, value.as_ref().as_bytes());
        }
    }
}

/// `StoreSetIfNotExistsBigFloat` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `bigfloat`
///     `StoreSetIfNotExistsBigFloat` implements AsRef<BigFloat> to give the client the flexibility
///     to either use the API with &BigFloat or BigFloat.
#[derive(StoreWriter)]
pub struct StoreSetIfNotExistsBigFloat {}
impl<V: AsRef<BigFloat>> StoreSetIfNotExists<V> for StoreSetIfNotExistsBigFloat {
    fn new() -> Self {
        Self {}
    }

    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set_if_not_exists(ord as i64, key, value.as_ref().to_string().as_bytes());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V) {
        for key in keys {
            state::set_if_not_exists(ord as i64, key, value.as_ref().to_string().as_bytes());
        }
    }
}

/// `StoreSetIfNotExistsBigInt` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `bigint`
///     `StoreSetIfNotExistsBigInt` implements AsRef<BigInt> to give the client the flexibility
///     to either use the API with &BigInt or BigInt.
#[derive(StoreWriter)]
pub struct StoreSetIfNotExistsBigInt {}
impl<V: AsRef<BigInt>> StoreSetIfNotExists<V> for StoreSetIfNotExistsBigInt {
    fn new() -> Self {
        Self {}
    }

    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set_if_not_exists(ord as i64, key, value.as_ref().to_string().as_bytes());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V) {
        for key in keys {
            state::set_if_not_exists(ord as i64, key, value.as_ref().to_string().as_bytes());
        }
    }
}

/// `StoreSetIfNotExistsI64` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `int64`
#[derive(StoreWriter)]
pub struct StoreSetIfNotExistsI64 {}
impl StoreSetIfNotExists<i64> for StoreSetIfNotExistsI64 {
    fn new() -> Self {
        Self {}
    }

    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: i64) {
        state::set_if_not_exists(ord as i64, key, value.to_string().as_bytes());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: i64) {
        for key in keys {
            state::set_if_not_exists(ord as i64, key, value.to_string().as_bytes());
        }
    }
}

/// `StoreSetIfNotExistsFloat64` is a struct representing a `store` module with `updatePolicy` equal to `set_if_not_exists` and a `valueType` equal to `float64`
#[derive(StoreWriter)]
pub struct StoreSetIfNotExistsFloat64 {}
impl StoreSetIfNotExists<f64> for StoreSetIfNotExistsFloat64 {
    fn new() -> Self {
        Self {}
    }

    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: f64) {
        state::set_if_not_exists(ord as i64, key, value.to_string().as_bytes());
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: f64) {
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

impl<T: Default + prost::Message> StoreSetIfNotExistsProto<T> {
    pub fn new() -> Self {
        StoreSetIfNotExistsProto {
            store: StoreSetIfNotExistsRaw {},
            casper: PhantomData,
        }
    }

    pub fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: &T) {
        match proto::encode(value) {
            Ok(bytes) => self.store.set_if_not_exists(ord, key, &bytes),
            Err(_) => panic!("failed to encode message"),
        }
    }

    pub fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &T) {
        for key in keys {
            match proto::encode(value) {
                Ok(bytes) => self.store.set_if_not_exists(ord, key, &bytes),
                Err(_) => panic!("failed to encode message"),
            }
        }
    }
}

/// `StoreAdd` is a trait which is implemented on any type of types StoreAdd
pub trait StoreAdd<V> {
    /// Add a given value to an already existing key
    fn add<K: AsRef<str>>(&self, ord: u64, key: K, value: V);
    /// Add multiple values to an already existing key
    fn add_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V);
}

/// `StoreAddInt64` is a struct representing a `store` module with `updatePolicy` equal to `add` and a valueType of `int64`
#[derive(StoreWriter)]
pub struct StoreAddInt64 {}
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
#[derive(StoreWriter)]
pub struct StoreAddFloat64 {}
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

/// `StoreAddBigFloat` is a struct representing a `store` module with `updatePolicy` equal to `add` and a valueType of `bigfloat`
///     `StoreAddBigFloat` implements AsRef<BigInt> to give the client the flexibility
///     to either use the API with &BigFloat or BigFloat.
#[derive(StoreWriter)]
pub struct StoreAddBigFloat {}
impl<V: AsRef<BigFloat>> StoreAdd<V> for StoreAddBigFloat {
    fn add<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::add_bigfloat(ord as i64, key, value.as_ref());
    }

    fn add_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: V) {
        for key in keys {
            state::add_bigfloat(ord as i64, key, value.as_ref());
        }
    }
}

/// `StoreAddBigInt` is a struct representing a `store` module with `updatePolicy` equal to `add` and a valueType of `bigint`
///     `StoreAddBigInt` implements AsRef<BigInt> to give the client the flexibility
///     to either use the API with &BigInt or BigInt.
#[derive(StoreWriter)]
pub struct StoreAddBigInt {}
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
pub trait StoreMax<V> {
    /// max will set the provided key in the store only if the value received in
    /// parameter is bigger than the one already present in the store, with
    /// a default of the zero value when the key is absent.
    fn max<K: AsRef<str>>(&self, ord: u64, key: K, value: V);
}

/// `StoreMaxInt64` is a struct representing a `store` module with `updatePolicy` equal to `max` and a valueType of `int64`
#[derive(StoreWriter)]
pub struct StoreMaxInt64 {}
impl StoreMax<i64> for StoreMaxInt64 {
    fn max<K: AsRef<str>>(&self, ord: u64, key: K, value: i64) {
        state::set_max_int64(ord as i64, key, value);
    }
}

/// `StoreMaxBigInt` is a struct representing a `store` module with `updatePolicy` equal to `max` and a valueType of `bigint`
///     `StoreMaxBigInt` implements AsRef<BigInt> to give the client the flexibility
///     to either use the API with &BigInt or BigInt.
#[derive(StoreWriter)]
pub struct StoreMaxBigInt {}
impl<V: AsRef<BigInt>> StoreMax<V> for StoreMaxBigInt {
    fn max<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set_max_bigint(ord as i64, key, value.as_ref());
    }
}

/// `StoreMaxFloat64` is a struct representing a `store` module with `updatePolicy` equal to `max` and a valueType of `float64`
#[derive(StoreWriter)]
pub struct StoreMaxFloat64 {}
impl StoreMax<f64> for StoreMaxFloat64 {
    fn max<K: AsRef<str>>(&self, ord: u64, key: K, value: f64) {
        state::set_max_float64(ord as i64, key, value);
    }
}

/// `StoreMaxBigFloat` is a struct representing a `store` module with `updatePolicy` equal to `max` and a valueType of `bigfloat`
///     `StoreMaxBigFloat` implements AsRef<BigFloat> to give the client the flexibility
///     to either use the API with &BigFloat or BigFloat.
#[derive(StoreWriter)]
pub struct StoreMaxBigFloat {}
impl<V: AsRef<BigFloat>> StoreMax<V> for StoreMaxBigFloat {
    fn max<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set_max_bigfloat(ord as i64, key, value.as_ref());
    }
}

/// `StoreMin` is a trait which is implemented on any typed StoreMin
pub trait StoreMin<V> {
    /// Will set the provided key in the store only if the value received in
    /// parameter is smaller than the one already present in the store, with
    /// a default of the zero value when the key is absent.
    fn min<K: AsRef<str>>(&self, ord: u64, key: K, value: V);
}

/// `StoreMinInt64` is a struct representing a `store` module with `updatePolicy` equal to `min` and a valueType of `int64`
#[derive(StoreWriter)]
pub struct StoreMinInt64 {}
impl StoreMin<i64> for StoreMinInt64 {
    fn min<K: AsRef<str>>(&self, ord: u64, key: K, value: i64) {
        state::set_min_int64(ord as i64, key, value);
    }
}

/// `StoreMinBigInt` is a struct representing a `store` module with `updatePolicy` equal to `min` and a valueType of `bigint`
///     `StoreMinBigInt` implements AsRef<BigInt> to give the client the flexibility
///     to either use the API with &BigInt or BigInt.
#[derive(StoreWriter)]
pub struct StoreMinBigInt {}
impl<V: AsRef<BigInt>> StoreMin<V> for StoreMinBigInt {
    fn min<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set_min_bigint(ord as i64, key, value.as_ref());
    }
}

/// `StoreMinFloat64` is a struct representing a `store` module with `updatePolicy` equal to `min` and a valueType of `float64`
#[derive(StoreWriter)]
pub struct StoreMinFloat64 {}
impl StoreMin<f64> for StoreMinFloat64 {
    fn min<K: AsRef<str>>(&self, ord: u64, key: K, value: f64) {
        state::set_min_float64(ord as i64, key, value);
    }
}

/// `StoreMinBigFloat` is a struct representing a `store` module with `updatePolicy` equal to `min` and a valueType of `bigfloat`
///     `StoreMinBigFloat` implements AsRef<BigFloat> to give the client the flexibility
///     to either use the API with &BigFloat or BigFloat.
#[derive(StoreWriter)]
pub struct StoreMinBigFloat {}
impl<V: AsRef<BigFloat>> StoreMin<V> for StoreMinBigFloat {
    fn min<K: AsRef<str>>(&self, ord: u64, key: K, value: V) {
        state::set_min_bigfloat(ord as i64, key, value.as_ref());
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

pub struct StoreGetBigFloat(StoreGetRaw);
impl StoreGet<BigFloat> for StoreGetBigFloat {
    fn new(idx: u32) -> Self {
        Self {
            0: StoreGetRaw { idx },
        }
    }

    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<BigFloat> {
        let bytes_option: Option<Vec<u8>> = state::get_at(self.0.idx, ord as i64, key);
        match bytes_option {
            None => None,
            Some(bytes) => Some(BigFloat::from_store_bytes(bytes)),
        }
    }

    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<BigFloat> {
        let bytes_option: Option<Vec<u8>> = state::get_last(self.0.idx, key);
        match bytes_option {
            None => None,
            Some(bytes) => Some(BigFloat::from_store_bytes(bytes)),
        }
    }

    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<BigFloat> {
        let bytes_option: Option<Vec<u8>> = state::get_first(self.0.idx, key);
        match bytes_option {
            None => None,
            Some(bytes) => Some(BigFloat::from_store_bytes(bytes)),
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

pub struct DeltaBigFloat {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: BigFloat,
    pub new_value: BigFloat,
}

impl Delta for DeltaBigFloat {
    fn new(d: &StoreDelta) -> Self {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal.clone(),
            key: d.key.clone(),
            old_value: BigFloat::from_store_bytes(d.old_value.clone()),
            new_value: BigFloat::from_store_bytes(d.new_value.clone()),
        }
    }
}

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
        if d.old_value.len() != 0 {
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
        if d.old_value.len() != 0 {
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
