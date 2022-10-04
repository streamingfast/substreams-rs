//! Store Implementation for Substreams.
//!
//! This crate implements the different Stores which can be used in your Substreams
//! handlers.
//!

use crate::pb::substreams::StoreDelta;
use crate::scalar::{BigDecimal, BigInt};
use crate::state;
use crate::{pb, proto};
use prost;
use std::marker::PhantomData;
use substreams_macro::StoreWriter;

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
pub struct RawStoreSet {}
impl StoreSet<Vec<u8>> for RawStoreSet {
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

/// BigIntStoreSet is a struct representing a `store` with `updatePolicy` equal to `set`
///     on a `valueType` equal to `bigint`
#[derive(StoreWriter)]
pub struct BigIntStoreSet {}
impl StoreSet<BigInt> for BigIntStoreSet {
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
pub struct BigDecimalStoreSet {}
impl StoreSet<BigDecimal> for BigDecimalStoreSet {
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
pub struct ProtoStoreSet<T: Default + prost::Message> {
    casper: PhantomData<T>,
}

impl<T: Default + prost::Message> StoreSet<T> for ProtoStoreSet<T> {
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
pub struct RawStoreSetIfNotExists {}
impl StoreSetIfNotExists<&Vec<u8>> for RawStoreSetIfNotExists {
    fn new() -> Self {
        RawStoreSetIfNotExists {}
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
pub struct ProtoStoreSetIfNotExists<T> {
    store: RawStoreSetIfNotExists,
    hack: Option<T>,
}

impl<T: Default + prost::Message> StoreSetIfNotExists<T> for ProtoStoreSetIfNotExists<T> {
    fn new() -> Self {
        ProtoStoreSetIfNotExists {
            store: RawStoreSetIfNotExists {},
            hack: None,
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

/// StoreAppend is a struct representing a `store` with
/// `updatePolicy` equal to `append`
#[derive(StoreWriter)]
pub struct StoreAppend {}
impl StoreAppend {
    /// Concatenates a given value at the end of the key's current value
    pub fn append<K: AsRef<str>>(&self, ord: u64, key: K, value: &String) {
        state::append(ord as i64, key, &value.as_bytes().to_vec());
    }

    /// Concatenates a given value at the end of the key's current value
    pub fn append_bytes<K: AsRef<str>>(&self, ord: u64, key: K, value: &Vec<u8>) {
        state::append(ord as i64, key, value);
    }
}

pub struct BigDecimalStoreGet(RawStoreGet);
impl StoreGet<BigDecimal> for BigDecimalStoreGet {
    fn new(idx: u32) -> BigDecimalStoreGet {
        BigDecimalStoreGet {
            0: RawStoreGet { idx },
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

pub struct BigIntStoreGet(RawStoreGet);
impl BigIntStoreGet {
    pub fn new(idx: u32) -> BigIntStoreGet {
        BigIntStoreGet {
            0: RawStoreGet { idx },
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

/// StoreGet is a trait which is implemented on any type of typed StoreGet
pub trait StoreGet<T> {
    fn new(idx: u32) -> Self;
    fn get_at<K: AsRef<str>>(&self, ord: u64, key: K) -> Option<T>;
    fn get_last<K: AsRef<str>>(&self, key: K) -> Option<T>;
    fn get_first<K: AsRef<str>>(&self, key: K) -> Option<T>;
}

/// RawStoreGet is a struct representing a read only store `store`
pub struct RawStoreGet {
    idx: u32,
}

impl StoreGet<Vec<u8>> for RawStoreGet {
    /// Return a StoreGet object with a store index set
    fn new(idx: u32) -> RawStoreGet {
        RawStoreGet { idx }
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

#[allow(dead_code)]
pub struct ProtoStoreGet<T> {
    store: RawStoreGet,
    hack: Option<T>,
}

impl<T: Default + prost::Message> ProtoStoreGet<T> {
    pub fn must_get_last<K: AsRef<str>>(&self, key: K) -> T {
        match self.get_last(key.as_ref().clone()) {
            None => {
                panic!("pool does not exist skipping pool {:?}", &key.as_ref());
            }
            Some(value) => value,
        }
    }
}

impl<T> StoreGet<T> for ProtoStoreGet<T>
where
    T: Default + prost::Message,
{
    /// Return a StoreGet object with a store index set
    fn new(idx: u32) -> ProtoStoreGet<T> {
        ProtoStoreGet {
            store: RawStoreGet { idx },
            hack: None,
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

pub struct BigDecimalDelta {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: BigDecimal,
    pub new_value: BigDecimal,
}
impl Delta for BigDecimalDelta {
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

pub struct BigIntDelta {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: BigInt,
    pub new_value: BigInt,
}

impl Delta for BigIntDelta {
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

pub struct I64Delta {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: i64,
    pub new_value: i64,
}

impl Delta for I64Delta {
    fn new(d: &StoreDelta) -> I64Delta {
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
pub struct StringDelta {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: String,
    pub new_value: String,
}

impl Delta for StringDelta {
    fn new(d: &StoreDelta) -> StringDelta {
        Self {
            operation: convert_i32_to_operation(d.operation),
            ordinal: d.ordinal.clone(),
            key: d.key.clone(),
            old_value: String::from_utf8(d.old_value.clone()).unwrap(),
            new_value: String::from_utf8(d.new_value.clone()).unwrap(),
        }
    }
}

pub struct ProtoDelta<T> {
    pub operation: pb::substreams::store_delta::Operation,
    pub ordinal: u64,
    pub key: String,
    pub old_value: T,
    pub new_value: T,
}

impl<T> Delta for ProtoDelta<T>
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
