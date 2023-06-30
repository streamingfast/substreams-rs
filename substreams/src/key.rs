use std::io::BufRead;

/// The key module contains functions for working extracting segments from key.
///
/// In a lot of use cases, you will encode data into your keys into segmented parts, adding a prefix
/// as namespace for example `user` and `<address>` joined together using a separator.  The `key` module
/// expects keys to use the `:` segment separator so keys looks like `<segment>[:<segment>]*`. Concrete
/// examples are `user:0x1234` or `user:0x1234:balance`.
///
/// You can extract various parts from a key:
///
/// ```rust
/// use substreams::key;
/// use substreams::store::{Delta, Deltas, DeltaBigDecimal};
///
/// fn db_out(deltas: Deltas<DeltaBigDecimal>) {
///     for delta in deltas {
///         let kind = key::first_segment(delta.get_key());
///         let address = key::segment_at(delta.get_key(), 1);
///         // Do something for this kind and address
///     }
/// }
/// ```
///
/// Those method panics if the key is not properly formatted or refering an invalid part. Use
/// the `try_` version if you want to handle errors:
///
/// ```rust
/// use substreams::key;
/// use substreams::store::{Delta, Deltas, DeltaBigDecimal};
///
/// fn db_out(deltas: Deltas<DeltaBigDecimal>) {
///     for delta in deltas {
///         let kind = key::try_first_segment(delta.get_key()).expect("invalid key: kind");
///         let address = key::try_segment_at(delta.get_key(), 1).expect("invalid key: address");
///         // Do something for this kind and address
///     }
/// }
/// ```
use crate::prelude::Delta;

pub fn segment_at(key: &String, index: usize) -> &str {
    try_segment_at(key, index).unwrap()
}

pub fn segment_at_owned(key: String, index: usize) -> String {
    let mut parts = std::io::Cursor::new(key.into_bytes()).split(b':');

    // Use of unwrap because those who want to check errors must use the try_ version
    let segment_result = parts.nth(index).unwrap();

    // Use of unwrap because I/O is infallible as we own the memory location already (no external I/O is done)
    let segment = segment_result.unwrap();

    String::from_utf8(segment).unwrap()
}

pub fn first_segment(key: &String) -> &str {
    segment_at(key, 0)
}

pub fn last_segment(key: &String) -> &str {
    try_last_segment(key).unwrap()
}

pub fn try_segment_at(key: &String, index: usize) -> Option<&str> {
    let val = key.split(":").nth(index);
    match val {
        Some(val) => Some(val),
        None => None,
    }
}

pub fn try_first_segment(key: &String) -> Option<&str> {
    try_segment_at(key, 0)
}

pub fn try_last_segment(key: &String) -> Option<&str> {
    match key.split(":").last() {
        Some(val) => Some(val),
        None => None,
    }
}

pub struct SegmentAtEq<I, S>
where
    I: Iterator,
    S: AsRef<str>,
{
    segment: S,
    // Some(x) means we are looking for the xth segment, None means check last segment of key
    at: Option<usize>,
    underlying: I,
}

impl<I, S> SegmentAtEq<I, S>
where
    I: Iterator,
    I::Item: Delta,
    S: AsRef<str>,
{
    pub(crate) fn new(segment: S, at: Option<usize>, underlying: I) -> Self {
        Self {
            segment,
            at,
            underlying,
        }
    }
}

impl<I, S> Iterator for SegmentAtEq<I, S>
where
    I: Iterator,
    I::Item: Delta,
    S: AsRef<str>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(x) = self.underlying.next() {
            let part = match self.at {
                Some(at) => segment_at(x.get_key(), at),
                None => last_segment(x.get_key()),
            };

            if self.segment.as_ref() == part {
                return Some(x);
            }
        }
        None
    }
}

pub struct SegmentAtIn<I, S, V>
where
    I: Iterator,
    S: AsRef<str>,
    V: AsRef<[S]>,
{
    segments: V,
    // Some(x) means we are looking for the xth segment, None means check last segment of key
    at: Option<usize>,
    underlying: I,
    phantom: std::marker::PhantomData<S>,
}

impl<I, S, V> SegmentAtIn<I, S, V>
where
    I: Iterator,
    I::Item: Delta,
    S: AsRef<str>,
    V: AsRef<[S]>,
{
    pub(crate) fn new(segments: V, at: Option<usize>, underlying: I) -> Self {
        Self {
            segments,
            at,
            underlying,
            phantom: std::marker::PhantomData,
        }
    }
}

impl<I, S, V> Iterator for SegmentAtIn<I, S, V>
where
    I: Iterator,
    I::Item: Delta,
    S: AsRef<str>,
    V: AsRef<[S]>,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.segments.as_ref().is_empty() {
            return None;
        }

        while let Some(x) = self.underlying.next() {
            let part = match self.at {
                Some(at) => segment_at(x.get_key(), at),
                None => last_segment(x.get_key()),
            };

            if self.segments.as_ref().iter().any(|x| x.as_ref() == part) {
                return Some(x);
            }
        }
        None
    }
}
