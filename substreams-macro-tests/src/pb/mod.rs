//! Quick-protobuf message types for testing.
//!
//! These are hand-written quick-protobuf compatible types for testing the macro.

use quick_protobuf::{
    sizeofs::{sizeof_len, sizeof_varint},
    BytesReader, MessageRead, MessageWrite, Result, Writer, WriterBackend,
};

/// A simple input message for testing decoding.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TestInput {
    /// Field 1: a string value
    pub name: String,
    /// Field 2: an integer value
    pub value: u64,
}

impl<'a> MessageRead<'a> for TestInput {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes)? {
                10 => msg.name = r.read_string(bytes)?.to_owned(),
                16 => msg.value = r.read_uint64(bytes)?,
                t => {
                    r.read_unknown(bytes, t)?;
                }
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for TestInput {
    fn get_size(&self) -> usize {
        let mut size = 0;
        if !self.name.is_empty() {
            size += 1 + sizeof_len(self.name.len());
        }
        if self.value != 0 {
            size += 1 + sizeof_varint(self.value);
        }
        size
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if !self.name.is_empty() {
            w.write_with_tag(10, |w| w.write_string(&self.name))?;
        }
        if self.value != 0 {
            w.write_with_tag(16, |w| w.write_uint64(self.value))?;
        }
        Ok(())
    }
}

/// A simple output message for testing encoding.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TestOutput {
    /// Field 1: a result string
    pub result: String,
    /// Field 2: a count
    pub count: u32,
}

impl<'a> MessageRead<'a> for TestOutput {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes)? {
                10 => msg.result = r.read_string(bytes)?.to_owned(),
                16 => msg.count = r.read_uint32(bytes)?,
                t => {
                    r.read_unknown(bytes, t)?;
                }
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for TestOutput {
    fn get_size(&self) -> usize {
        let mut size = 0;
        if !self.result.is_empty() {
            size += 1 + sizeof_len(self.result.len());
        }
        if self.count != 0 {
            size += 1 + sizeof_varint(self.count as u64);
        }
        size
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if !self.result.is_empty() {
            w.write_with_tag(10, |w| w.write_string(&self.result))?;
        }
        if self.count != 0 {
            w.write_with_tag(16, |w| w.write_uint32(self.count))?;
        }
        Ok(())
    }
}

/// A borrowed message type that actually borrows from the input buffer.
/// This type demonstrates the lifetime issue with quick-protobuf.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BorrowedMessage<'a> {
    /// Field 1: a borrowed string slice
    pub name: std::borrow::Cow<'a, str>,
    /// Field 2: borrowed bytes
    pub data: std::borrow::Cow<'a, [u8]>,
}

impl<'a> MessageRead<'a> for BorrowedMessage<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes)? {
                10 => msg.name = std::borrow::Cow::Borrowed(r.read_string(bytes)?),
                18 => msg.data = std::borrow::Cow::Borrowed(r.read_bytes(bytes)?),
                t => {
                    r.read_unknown(bytes, t)?;
                }
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for BorrowedMessage<'a> {
    fn get_size(&self) -> usize {
        let mut size = 0;
        if !self.name.is_empty() {
            size += 1 + sizeof_len(self.name.len());
        }
        if !self.data.is_empty() {
            size += 1 + sizeof_len(self.data.len());
        }
        size
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if !self.name.is_empty() {
            w.write_with_tag(10, |w| w.write_string(&self.name))?;
        }
        if !self.data.is_empty() {
            w.write_with_tag(18, |w| w.write_bytes(&self.data))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_roundtrip() {
        let input = TestInput {
            name: "hello".to_string(),
            value: 42,
        };

        // Encode
        let mut buf = Vec::new();
        let mut writer = Writer::new(&mut buf);
        input.write_message(&mut writer).unwrap();

        // Decode
        let mut reader = BytesReader::from_bytes(&buf);
        let decoded = TestInput::from_reader(&mut reader, &buf).unwrap();

        assert_eq!(input, decoded);
    }

    #[test]
    fn test_output_roundtrip() {
        let output = TestOutput {
            result: "success".to_string(),
            count: 100,
        };

        // Encode
        let mut buf = Vec::new();
        let mut writer = Writer::new(&mut buf);
        output.write_message(&mut writer).unwrap();

        // Decode
        let mut reader = BytesReader::from_bytes(&buf);
        let decoded = TestOutput::from_reader(&mut reader, &buf).unwrap();

        assert_eq!(output, decoded);
    }

    #[test]
    fn test_borrowed_message_roundtrip() {
        let borrowed = BorrowedMessage {
            name: std::borrow::Cow::Borrowed("borrowed"),
            data: std::borrow::Cow::Borrowed(b"data"),
        };

        // Encode
        let mut buf = Vec::new();
        let mut writer = Writer::new(&mut buf);
        borrowed.write_message(&mut writer).unwrap();

        // Decode
        let mut reader = BytesReader::from_bytes(&buf);
        let decoded = BorrowedMessage::from_reader(&mut reader, &buf).unwrap();

        assert_eq!(borrowed, decoded);
    }
}
