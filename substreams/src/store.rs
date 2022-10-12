//! Store Implementation for Substreams.
//!
//! This crate implements the different Stores which can be used in your Substreams
//! handlers.
//!

use std::i64;
use std::str::FromStr;
use {
    crate::{
        pb::substreams::StoreDelta,
        scalar::{BigDecimal, BigInt},
        state, {pb, proto},
    },
    prost,
    std::marker::PhantomData,
    substreams_macro::StoreWriter,
};

/// StoreSet is a trait which is implemented on any type of typed StoreSet
pub trait StoreSet<T> {
    fn new() -> Self;
    /// Set a given key to a given value, if the key existed before, it will be replaced.  
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &T);
    /// Set many keys to a given values, if the key existed before, it will be replaced.
    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &T);
}

/// RawStoreSet is a struct representing a `store` with `updatePolicy` equal to `set`
#[derive(StoreWriter)]
pub struct StoreSetRaw {}
impl StoreSet<Vec<u8>> for StoreSetRaw {
    fn new() -> Self {
        Self {}
    }

    /// Set a given key to a given value, if the key existed before, it will be replaced.
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &Vec<u8>) {
        state::set(ord as i64, key, value);
    }

    /// Set many keys to a given values, if the key existed before, it will be replaced.
    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &Vec<u8>) {
        for key in keys {
            state::set(ord as i64, key, value);
        }
    }
}

/// I64StoreSet is a struct representing a `store` with `updatePolicy` equal to `set`
#[derive(StoreWriter)]
pub struct StoreSetI64 {}
impl StoreSet<i64> for StoreSetI64 {
    fn new() -> Self {
        Self {}
    }

    /// Set a given key to a given value, if the key existed before, it will be replaced.
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &i64) {
        state::set(ord as i64, key, Vec::from(value.to_string()));
    }

    /// Set many keys to a given values, if the key existed before, it will be replaced.
    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &i64) {
        for key in keys {
            state::set(ord as i64, key, Vec::from(value.to_string()));
        }
    }
}

/// BigIntStoreSet is a struct representing a `store` with `updatePolicy` equal to `set`
///     on a `valueType` equal to `bigint`
#[derive(StoreWriter)]
pub struct StoreSetBigInt {}
impl StoreSet<BigInt> for StoreSetBigInt {
    fn new() -> Self {
        Self {}
    }

    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &BigInt) {
        state::set(ord as i64, key, &Vec::from(value.to_string()));
    }

    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &BigInt) {
        for key in keys {
            state::set(ord as i64, key, &Vec::from(value.to_string()));
        }
    }
}

#[derive(StoreWriter)]
pub struct StoreSetBigDecimal {}
impl StoreSet<BigDecimal> for StoreSetBigDecimal {
    fn new() -> Self {
        Self {}
    }
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &BigDecimal) {
        state::set(ord as i64, key, &Vec::from(value.to_string().as_str()))
    }

    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &BigDecimal) {
        for key in keys {
            state::set(ord as i64, key, &Vec::from(value.to_string().as_str()))
        }
    }
}

#[allow(dead_code)]
pub struct StoreSetProto<T: Default + prost::Message> {
    casper: PhantomData<T>,
}

impl<T: Default + prost::Message> StoreSet<T> for StoreSetProto<T> {
    fn new() -> Self {
        Self {
            //Adding a PhantomData<T> field to your type tells the compiler that your type acts as though it stores a value of type T, even though it doesn't really. This information is used when computing certain safety properties.
            // For a more in-depth explanation of how to use PhantomData<T>
            casper: PhantomData,
        }
    }

    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &T) {
        match proto::encode(value) {
            Ok(bytes) => state::set(ord as i64, key, &bytes),
            Err(_) => panic!("failed to encode message"),
        }
    }

    fn set_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &T) {
        for key in keys {
            match proto::encode(value) {
                Ok(bytes) => state::set(ord as i64, key, &bytes),
                Err(_) => panic!("failed to encode message"),
            }
        }
    }
}

/// StoreSetIfNotExists is a struct for which other structs
pub trait StoreSetIfNotExists<T> {
    /// Initializes a new StoreSetIfNotExists
    fn new() -> Self;
    /// Set a given key to a given value, if the key existed before, it will be ignored and not set.  
    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: T);
    /// Set given keys to given values, if the key existed before, it will be ignored and not set.
    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: T);
}

/// StoreSetIfNotExists is a struct representing a `store` module with
/// `updatePolicy` equal to `set_if_not_exists`
#[derive(StoreWriter)]
pub struct StoreSetIfNotExistsRaw {}
impl StoreSetIfNotExists<&Vec<u8>> for StoreSetIfNotExistsRaw {
    fn new() -> Self {
        StoreSetIfNotExistsRaw {}
    }

    /// Set a given key to a given value, if the key existed before, it will be ignored and not set.
    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: &Vec<u8>) {
        state::set_if_not_exists(ord as i64, key, value);
    }

    /// Set given keys to given values, if the key existed before, it will be ignored and not set.
    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: &Vec<u8>) {
        for key in keys {
            state::set_if_not_exists(ord as i64, key, value);
        }
    }
}

#[allow(dead_code)]
pub struct StoreSetIfNotExistsProto<T> {
    store: StoreSetIfNotExistsRaw,
    casper: PhantomData<T>,
}

impl<T: Default + prost::Message> StoreSetIfNotExists<T> for StoreSetIfNotExistsProto<T> {
    fn new() -> Self {
        StoreSetIfNotExistsProto {
            store: StoreSetIfNotExistsRaw {},
            casper: PhantomData,
        }
    }

    fn set_if_not_exists<K: AsRef<str>>(&self, ord: u64, key: K, value: T) {
        match proto::encode(&value) {
            Ok(bytes) => self.store.set_if_not_exists(ord, key, &bytes),
            Err(_) => panic!("failed to encode message"),
        }
    }

    fn set_if_not_exists_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: T) {
        for key in keys {
            match proto::encode(&value) {
                Ok(bytes) => self.store.set_if_not_exists(ord, key, &bytes),
                Err(_) => panic!("failed to encode message"),
            }
        }
    }
}

/// StoreAddInt64 is a struct representing a `store` module with
/// `updatePolicy` equal to `add` and a valueType of `int64`
#[derive(StoreWriter)]
pub struct StoreAddInt64 {}
impl StoreAddInt64 {
    /// Will add the value to the already present value at the key (or default to
    /// zero if the key was not set)
    pub fn add<K: AsRef<str>>(&self, ord: u64, key: K, value: i64) {
        state::add_int64(ord as i64, key, value);
    }

    /// Will add the value to the already present value of the keys (or default to
    /// zero if the key was not set)
    pub fn add_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: i64) {
        for key in keys {
            state::add_int64(ord as i64, key, value);
        }
    }
}

/// StoreAddFloat64 is a struct representing a `store` module with
/// `updatePolicy` equal to `add` and a valueType of `float64`
#[derive(StoreWriter)]
pub struct StoreAddFloat64 {}
impl StoreAddFloat64 {
    /// Will add the value to the already present value at the key (or default to
    /// zero if the key was not set)
    pub fn add<K: AsRef<str>>(&self, ord: u64, key: K, value: f64) {
        state::add_float64(ord as i64, key, value);
    }

    /// Will add the value to the already present value of the keys (or default to
    /// zero if the key was not set)
    pub fn add_many<K: AsRef<str>>(&self, ord: u64, keys: &Vec<K>, value: f64) {
        for key in keys {
            state::add_float64(ord as i64, key, value);
        }
    }
}

/// StoreAddBigFloat is a struct representing a `store` module with
/// `updatePolicy` equal to `add` and a valueType of `bigfloat`
#[derive(StoreWriter)]
pub struct StoreAddBigFloat {}
impl StoreAddBigFloat {
    /// Will add the value to the already present value at the key (or default to
    /// zero if the key was not set)
    pub fn add<K, V>(&self, ord: u64, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<BigDecimal>,
    {
        state::add_bigfloat(ord as i64, key, value);
    }

    /// Will add the value to the already present value of the keys (or default to
    /// zero if the key was not set)
    pub fn add_many<K, V>(&self, ord: u64, keys: &Vec<K>, value: &V)
    where
        K: AsRef<str>,
        V: AsRef<BigDecimal>,
    {
        for key in keys {
            state::add_bigfloat(ord as i64, key, value);
        }
    }
}

/// StoreAddBigInt is a struct representing a `store` module with
/// `updatePolicy` equal to `add` and a valueType of `bigint`
#[derive(StoreWriter)]
pub struct StoreAddBigInt {}
impl StoreAddBigInt {
    /// Will add the value to the already present value of the keys (or default to
    /// zero if the key was not set)
    pub fn add<K, V>(&self, ord: u64, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<BigInt>,
    {
        state::add_bigint(ord as i64, key, value);
    }

    /// Will add the value to the already present value of the keys (or default to
    /// zero if the key was not set)
    pub fn add_many<K, V>(&self, ord: u64, keys: &Vec<K>, value: &V)
    where
        K: AsRef<str>,
        V: AsRef<BigInt>,
    {
        for key in keys {
            state::add_bigint(ord as i64, key, value);
        }
    }
}

/// StoreMaxInt64 is a struct representing a `store` module with
/// `updatePolicy` equal to `max` and a valueType of `int64`
#[derive(StoreWriter)]
pub struct StoreMaxInt64 {}
impl StoreMaxInt64 {
    /// max will set the provided key in the store only if the value received in
    /// parameter is bigger than the one already present in the store, with
    /// a default of the zero value when the key is absent.
    pub fn max<K: AsRef<str>>(&self, ord: u64, key: K, value: i64) {
        state::set_max_int64(ord as i64, key, value);
    }
}

/// StoreMaxBigInt is a struct representing a `store` module with
/// `updatePolicy` equal to `max` and a valueType of `bigint`
#[derive(StoreWriter)]
pub struct StoreMaxBigInt {}
impl StoreMaxBigInt {
    /// Will set the provided key in the store only if the value received in
    /// parameter is bigger than the one already present in the store, with
    /// a default of the zero value when the key is absent.
    pub fn max<K, V>(&self, ord: u64, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<BigInt>,
    {
        state::set_max_bigint(ord as i64, key, value);
    }
}

/// StoreMaxFloat64 is a struct representing a `store` module with
/// `updatePolicy` equal to `max` and a valueType of `float64`
#[derive(StoreWriter)]
pub struct StoreMaxFloat64 {}
impl StoreMaxFloat64 {
    /// Will set the provided key in the store only if the value received in
    /// parameter is bigger than the one already present in the store, with
    /// a default of the zero value when the key is absent.
    pub fn max<K: AsRef<str>>(&self, ord: u64, key: K, value: f64) {
        state::set_max_float64(ord as i64, key, value);
    }
}

/// StoreMaxBigFloat is a struct representing a `store` module with
/// `updatePolicy` equal to `max` and a valueType of `bigfloat`
#[derive(StoreWriter)]
pub struct StoreMaxBigFloat {}
impl StoreMaxBigFloat {
    /// Will set the provided key in the store only if the value received in
    /// parameter is bigger than the one already present in the store, with
    /// a default of the zero value when the key is absent.
    pub fn max<K, V>(&self, ord: u64, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<BigDecimal>,
    {
        state::set_max_bigfloat(ord as i64, key, value);
    }
}

/// `StoreMinInt64` is a struct representing a `store` module with
/// `updatePolicy` equal to `min` and a valueType of `int64`
#[derive(StoreWriter)]
pub struct StoreMinInt64 {}
impl StoreMinInt64 {
    /// Will set the provided key in the store only if the value received in
    /// parameter is smaller than the one already present in the store, with
    /// a default of the zero value when the key is absent.
    pub fn min<K: AsRef<str>>(&self, ord: u64, key: K, value: i64) {
        state::set_min_int64(ord as i64, key, value);
    }
}

/// StoreMinBigInt is a struct representing a `store` module with
/// `updatePolicy` equal to `min` and a valueType of `bigint`
#[derive(StoreWriter)]
pub struct StoreMinBigInt {}
impl StoreMinBigInt {
    /// Will set the provided key in the store only if the value received in
    /// parameter is smaller than the one already present in the store, with
    /// a default of the zero value when the key is absent.
    pub fn min<K, V>(&self, ord: u64, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<BigInt>,
    {
        state::set_min_bigint(ord as i64, key, value);
    }
}

/// StoreMinFloat64 is a struct representing a `store` module with
/// `updatePolicy` equal to `min` and a valueType of `float64`
#[derive(StoreWriter)]
pub struct StoreMinFloat64 {}
impl StoreMinFloat64 {
    /// Will set the provided key in the store only if the value received in
    /// parameter is smaller than the one already present in the store, with
    /// a default of the zero value when the key is absent.
    pub fn min<K: AsRef<str>>(&self, ord: u64, key: K, value: f64) {
        state::set_min_float64(ord as i64, key, value);
    }
}

/// StoreMinBigFloat is a struct representing a `store` module with
/// `updatePolicy` equal to `min` and a valueType of `bigfloat`
#[derive(StoreWriter)]
pub struct StoreMinBigFloat {}
impl StoreMinBigFloat {
    /// Will set the provided key in the store only if the value received in
    /// parameter is smaller than the one already present in the store, with
    /// a default of the zero value when the key is absent.
    pub fn min<K, V>(&self, ord: u64, key: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<BigDecimal>,
    {
        state::set_min_bigfloat(ord as i64, key, value);
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

pub struct StoreGetBigDecimal(StoreGetRaw);
impl StoreGet<BigDecimal> for StoreGetBigDecimal {
    fn new(idx: u32) -> StoreGetBigDecimal {
        StoreGetBigDecimal {
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
impl StoreGetBigInt {
    pub fn new(idx: u32) -> Self {
        Self {
            0: StoreGetRaw { idx },
        }
    }

    pub fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<BigInt> {
        let store_bytes: Option<Vec<u8>> = state::get_at(self.0.idx, ord as i64, key);
        match store_bytes {
            None => None,
            Some(bytes) => Some(BigInt::from_store_bytes(bytes)),
        }
    }

    pub fn get_last<K: AsRef<str>>(&self, key: K) -> Option<BigInt> {
        let store_bytes: Option<Vec<u8>> = state::get_last(self.0.idx, key);
        match store_bytes {
            None => None,
            Some(bytes) => Some(BigInt::from_store_bytes(bytes)),
        }
    }

    pub fn get_first<K: AsRef<str>>(&self, key: K) -> Option<BigInt> {
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
    fn decode(d: &pb::substreams::StoreDelta) -> T;
}

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
        let ov_string = String::from_utf8(d.old_value.clone()).unwrap();
        let nv_string = String::from_utf8(d.new_value.clone()).unwrap();

        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal.clone(),
            key: d.key.clone(),
            old_value: ov_string.parse::<i64>().unwrap(),
            new_value: nv_string.parse::<i64>().unwrap(),
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
            "value {} is not a value representation of an i64",
            int_as_string
        ),
    };
}

#[cfg(test)]
mod test {
    use crate::store::decode_bytes_to_i64;

    #[test]
    fn valid_int64_1_decode_bytes_to_i64() {
        let bytes: Vec<u8> = Vec::from("1".to_string());
        assert_eq!(1, decode_bytes_to_i64(bytes).unwrap())
    }

    #[test]
    fn valid_int64_max_value_decode_bytes_to_i64() {
        let bytes: Vec<u8> = Vec::from("9223372036854775807".to_string());
        assert_eq!(9223372036854775807, decode_bytes_to_i64(bytes).unwrap())
    }

    #[test]
    #[should_panic]
    fn invalid_bytes_decode_bytes_to_i64() {
        let bytes: Vec<u8> = Vec::from("invalid".to_string());
        decode_bytes_to_i64(bytes);
    }
}
