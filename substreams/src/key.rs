use crate::pb;
use crate::prelude::Delta;

pub fn key_segment_in<T: Delta>(idx: usize, key_segment: &str) -> impl FnMut(&&T) -> bool + '_ {
    move |delta| segment(delta.get_key(), idx) == key_segment
}

pub fn key_first_segment_in<T: Delta>(key_segment: &str) -> impl FnMut(&&T) -> bool + '_ {
    move |delta| first_segment(delta.get_key()) == key_segment
}

pub fn key_last_segment_in<T: Delta>(key_segment: &str) -> impl FnMut(&&T) -> bool + '_ {
    move |delta| last_segment(delta.get_key()) == key_segment
}

pub fn key_first_segments_in<T: Delta>(idx: Vec<&str>) -> impl FnMut(&&T) -> bool + '_ {
    move |delta| idx.contains(&first_segment(delta.get_key()))
}

pub fn key_last_segments_in<T: Delta>(idx: Vec<&str>) -> impl FnMut(&&T) -> bool + '_ {
    move |delta| idx.contains(&last_segment(delta.get_key()))
}
pub fn operations_eq<T: Delta>(
    operation: pb::substreams::store_delta::Operation,
) -> impl FnMut(&&T) -> bool {
    move |delta| delta.get_operation() as i32 == operation as i32
}

pub fn operations_ne<T: Delta>(
    operation: pb::substreams::store_delta::Operation,
) -> impl FnMut(&&T) -> bool {
    move |delta| delta.get_operation() as i32 != operation as i32
}

pub fn first_segment(key: &String) -> &str {
    return segment(key, 0);
}

pub fn last_segment(key: &String) -> &str {
    return try_last_segment(key).unwrap();
}

pub fn try_last_segment(key: &String) -> Option<&str> {
    let val = key.split(":").last();
    match val {
        Some(val) => Some(val),
        None => None,
    }
}

pub fn segment(key: &String, index: usize) -> &str {
    return try_segment(key, index).unwrap();
}

pub fn try_segment(key: &String, index: usize) -> Option<&str> {
    let val = key.split(":").nth(index);
    match val {
        Some(val) => Some(val),
        None => None,
    }
}
