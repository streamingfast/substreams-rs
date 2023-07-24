//! Error proxy for Substreams.
//!
//! This crate implements Substreams error that you can
//! return in your Substreams handler.
//!
//! The Substreams [Error] implementation is simply a type alias to [anyhow::Error].
//! Anyhow is a crate that make it easy to create generic error as well as attaching
//! context to error from library.
//!
//! For example, let's say you want to return an error when the decoding fail with
//! in context the input data that failed to decode. You can do it like this:
//!
//! ```rust
//! # pub type Block = ();
//! # mod pb { pub type Custom = (); };
//! // **Important** Brings in scope `.context` and `.with_context` methods.
//! use anyhow::Context;
//!
//! fn map_handler(params: String, block: Block) -> Result<pb::Custom, substreams::errors::Error> {
//!     let address = substreams::Hex::decode(&params).with_context(|| format!("failed to decode address: {}", params))?;
//!
//!     unimplemented!("do something");
//! }
//!```
//!
//! If you want to return a plain error, you can use the `anyhow::anyhow!` macro:
//!
//! ```rust
//! # pub type Block = ();
//! # mod pb { pub type Custom = (); };
//! use anyhow::anyhow;
//!
//! fn map_handler(params: String, block: Block) -> Result<pb::Custom, substreams::errors::Error> {
//!     if params.len() != 42 {;
//!         return Err(anyhow!("invalid address length"));
//!     }
//!
//!     unimplemented!("do something");
//! }
//!```

/// Error proxy for Substreams, simply a type alias to [anyhow::Error].
///
/// See module [crate::errors] level documentation for more information.
pub type Error = anyhow::Error;

#[cfg(test)]
mod test {
    use super::Error;

    #[test]
    fn test_from_std_error() {
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let actual = Error::from(error);

        assert_eq!(format!("{:?}", actual), "test");
    }

    #[test]
    fn test_from_with_context() {
        let err = anyhow::anyhow!("source");
        let converted = Error::from(err);

        assert_eq!(format!("{:?}", converted), "source");
    }
}
