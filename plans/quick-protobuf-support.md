# Implementation Plan: Quick-Protobuf Support

## Ultimate Goal

Add quick-protobuf as a second supported protobuf library in substreams-rs, alongside existing Prost support, with:
1. A generic output function that works with both libraries
2. Configurable macro-generated map/store code supporting quick-protobuf via a config option
3. Full backward compatibility - existing Prost users should continue to work without code changes

## Current State Analysis

### 1. Output Function (`substreams/src/lib.rs:177-193`)

```rust
pub fn output<M: prost::Message>(msg: M) {
    #[cfg(target_arch = "wasm32")]
    {
        let (ptr, len, buffer) = proto::encode_to_ptr(&msg).unwrap_or_else(...);
        std::mem::forget(buffer);
        unsafe { externs::output(ptr, len as u32) }
    }
}
```

**Key observation**: The output function is tightly coupled to `prost::Message` trait bound.

### 2. Proto Module (`substreams/src/proto.rs`)

- `decode<T: Default + prost::Message>` - decodes bytes to Prost message
- `decode_ptr<T: Default + prost::Message>` - decodes from raw pointer
- `encode<M: prost::Message>` - encodes Prost message to bytes
- `encode_to_ptr<M: prost::Message>` - encodes to pointer

### 3. Store Module (`substreams/src/store.rs`)

Proto-related stores using `prost::Message`:
- `StoreSetProto<V: Default + prost::Message>`
- `StoreSetIfNotExistsProto<V: Default + prost::Message>`
- `StoreGetProto<T: Default + prost::Message>`
- `DeltaProto<T: Default + prost::Message + PartialEq>`

### 4. Macro Code Generation (`substreams-macro/src/handler.rs`)

The macro generates code that calls:
- `substreams::proto::decode_ptr(ptr, len)` for decoding inputs
- `substreams::output(result)` for encoding outputs

### 5. Quick-Protobuf API

From research:
- **Serialization trait**: `MessageWrite` with `write_message<W: WriterBackend>(&self, w: &mut Writer<W>)`
- **Deserialization trait**: `MessageRead<'a>` with `fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self>`
- **Convenience function**: `quick_protobuf::writer::serialize_into_vec<M: MessageWrite>(message: &M) -> Result<Vec<u8>>`

Key differences from Prost:
- `MessageRead` has a lifetime parameter `'a` for borrowed data
- Uses `BytesReader` + byte slice instead of just bytes
- Different error types

---

## Implementation Plan

### Phase 1: Create Generic Protobuf Traits

Create a new module `substreams/src/pb_compat.rs` with abstraction traits:

```rust
/// Trait for types that can be encoded to protobuf bytes
pub trait ProtoEncode {
    fn encode_to_vec(&self) -> Result<Vec<u8>, EncodeError>;
}

/// Trait for types that can be decoded from protobuf bytes
pub trait ProtoDecode: Sized {
    fn decode_from_slice(bytes: &[u8]) -> Result<Self, DecodeError>;
}
```

#### 1.1 Implement for Prost (default, always available)

```rust
impl<T: prost::Message> ProtoEncode for T {
    fn encode_to_vec(&self) -> Result<Vec<u8>, EncodeError> {
        // Use existing proto::encode
    }
}

impl<T: prost::Message + Default> ProtoDecode for T {
    fn decode_from_slice(bytes: &[u8]) -> Result<Self, DecodeError> {
        prost::Message::decode(bytes)
    }
}
```

#### 1.2 Implement for Quick-Protobuf (behind feature flag)

```rust
#[cfg(feature = "quick-protobuf")]
impl<T: quick_protobuf::MessageWrite> ProtoEncode for T {
    fn encode_to_vec(&self) -> Result<Vec<u8>, EncodeError> {
        quick_protobuf::writer::serialize_into_vec(self)
            .map_err(|e| EncodeError::from(e))
    }
}

#[cfg(feature = "quick-protobuf")]
impl<'a, T: quick_protobuf::MessageRead<'static> + Default> ProtoDecode for T {
    fn decode_from_slice(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut reader = quick_protobuf::BytesReader::from_bytes(bytes);
        T::from_reader(&mut reader, bytes)
            .map_err(|e| DecodeError::from(e))
    }
}
```

**Note**: The lifetime handling for `MessageRead<'a>` is tricky. We need `'static` for owned data.

### Phase 2: Refactor Output Function

#### 2.1 New Generic Output Function

```rust
/// Output a protobuf message. Works with both Prost and quick-protobuf types.
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn output<M: ProtoEncode>(msg: M) {
    #[cfg(target_arch = "wasm32")]
    {
        let buffer = msg.encode_to_vec().unwrap_or_else(|_| {
            panic!("Unable to encode message to Protobuf data")
        });
        let ptr = buffer.as_ptr();
        let len = buffer.len();
        std::mem::forget(buffer);
        unsafe { externs::output(ptr, len as u32) }
    }
}
```

#### 2.2 Backward Compatibility

Since `prost::Message` types will implement `ProtoEncode`, existing code using:
```rust
substreams::output(my_prost_message);
```
Will continue to work without changes.

### Phase 3: Refactor Proto Module

#### 3.1 Add Generic Decode Functions

```rust
/// Generic decode from bytes
pub fn decode<T: ProtoDecode>(buf: &[u8]) -> Result<T, DecodeError> {
    T::decode_from_slice(buf)
}

/// Generic decode from pointer
pub fn decode_ptr<T: ProtoDecode>(ptr: *mut u8, size: usize) -> Result<T, DecodeError> {
    unsafe {
        let input_data = Vec::from_raw_parts(ptr, size, size);
        let obj = T::decode_from_slice(&input_data);
        std::mem::forget(input_data);
        obj
    }
}

/// Generic encode
pub fn encode<M: ProtoEncode>(msg: &M) -> Result<Vec<u8>, EncodeError> {
    msg.encode_to_vec()
}
```

#### 3.2 Keep Prost-Specific Functions (deprecated)

For backward compatibility, keep old functions but mark them deprecated:

```rust
#[deprecated(since = "0.8.0", note = "Use decode<T: ProtoDecode> instead")]
pub fn decode_prost<T: Default + prost::Message>(buf: &Vec<u8>) -> Result<T, prost::DecodeError> {
    ::prost::Message::decode(&buf[..])
}
```

### Phase 4: Update Store Types

#### 4.1 Refactor Proto Stores to Use Generic Traits

```rust
/// Generic proto store that works with both Prost and quick-protobuf
pub struct StoreSetProto<V: ProtoEncode + ProtoDecode> {
    casper: PhantomData<V>,
}

impl<V: ProtoEncode + ProtoDecode> StoreSet<V> for StoreSetProto<V> {
    fn set<K: AsRef<str>>(&self, ord: u64, key: K, value: &V) {
        let bytes = value.encode_to_vec()
            .unwrap_or_else(|_| panic!("Unable to encode store message"));
        state::set(ord as i64, key, &bytes)
    }
}
```

#### 4.2 Similar Updates for:
- `StoreSetIfNotExistsProto`
- `StoreGetProto`
- `DeltaProto`

### Phase 5: Update Macro Code Generation

#### 5.1 Add Macro Option for quick-protobuf

Update `substreams-macro/src/config.rs`:

```rust
#[derive(Clone, Copy, Default)]
pub struct HandlerOptions {
    pub keep_empty_output: bool,
    pub no_testable: bool,
    pub quick_protobuf: bool,  // NEW
}

impl HandlerOptions {
    pub fn parse(args: &str) -> Result<Self, String> {
        // ... existing parsing ...
        for part in args.split(',') {
            match part.trim() {
                "quick_protobuf" => options.quick_protobuf = true,
                // ... existing options ...
            }
        }
        Ok(options)
    }
}
```

#### 5.2 Update Handler Code Generation

In `substreams-macro/src/handler.rs`, modify proto decoding based on option:

```rust
if options.quick_protobuf {
    proto_decodings.push(quote! {
        let #mutability #var_name: #argument_type = {
            let bytes = unsafe { std::slice::from_raw_parts(#var_ptr, #var_len) };
            let mut reader = quick_protobuf::BytesReader::from_bytes(bytes);
            quick_protobuf::MessageRead::from_reader(&mut reader, bytes)
                .unwrap_or_else(|_| panic!("Unable to decode..."))
        };
    });
} else {
    // Existing prost decoding
    proto_decodings.push(quote! {
        let #mutability #var_name: #argument_type = substreams::proto::decode_ptr(#var_ptr, #var_len)...
    });
}
```

### Phase 6: Feature Flags in Cargo.toml

```toml
[features]
default = []
quick-protobuf = ["dep:quick-protobuf"]

[dependencies]
prost = "0.13"
quick-protobuf = { version = "0.8", optional = true }
```

---

## Alternative: Simpler Implementation (Recommended)

Given that `output_raw(Vec<u8>)` already exists, we can take a simpler approach that requires fewer changes:

### Minimal Changes Approach

**1. Keep `output<M: prost::Message>` unchanged** - No breaking changes for Prost users

**2. Add `output_quick<M>` for quick-protobuf:**
```rust
#[cfg(feature = "quick-protobuf")]
pub fn output_quick<M: quick_protobuf::MessageWrite>(msg: &M) {
    #[cfg(target_arch = "wasm32")]
    {
        let buffer = quick_protobuf::serialize_into_vec(msg)
            .unwrap_or_else(|_| panic!("Unable to encode message"));
        output_raw(buffer);
    }
}
```

**3. Macro generates different output call based on option:**
```rust
// For prost (default)
substreams::output(result);

// For quick_protobuf option
substreams::output_quick(&result);
```

**4. Add quick-protobuf decode helper:**
```rust
#[cfg(feature = "quick-protobuf")]
pub mod quick {
    pub fn decode<'a, T: quick_protobuf::MessageRead<'a>>(bytes: &'a [u8]) -> Result<T, Error> {
        let mut reader = quick_protobuf::BytesReader::from_bytes(bytes);
        T::from_reader(&mut reader, bytes)
    }

    pub fn decode_ptr<T: for<'a> quick_protobuf::MessageRead<'a>>(
        ptr: *mut u8,
        size: usize
    ) -> Result<T, Error> {
        unsafe {
            let bytes = std::slice::from_raw_parts(ptr, size);
            decode(bytes)
        }
    }
}
```

This approach:
- Zero breaking changes for existing Prost users
- Minimal code additions (just new functions, no trait refactoring)
- Clean separation via feature flag
- Macro handles the switching

---

## Implementation Order (Full Approach)

1. **Phase 1**: Create `pb_compat.rs` with `ProtoEncode` and `ProtoDecode` traits
2. **Phase 2**: Refactor `output()` function to use `ProtoEncode`
3. **Phase 3**: Refactor `proto.rs` to use generic traits
4. **Phase 4**: Update store types to use generic traits
5. **Phase 5**: Add macro option and update code generation
6. **Phase 6**: Add feature flag and optional dependency

## Implementation Order (Simpler Approach - Recommended)

1. **Phase 1**: Add `quick-protobuf` feature flag and optional dependency
2. **Phase 2**: Add `output_quick()` function behind feature flag
3. **Phase 3**: Add `substreams::quick::decode_ptr()` helper behind feature flag
4. **Phase 4**: Update macro to add `quick_protobuf` option
5. **Phase 5**: Macro generates appropriate code based on option

## Testing Strategy

1. **Unit tests**: Test `ProtoEncode`/`ProtoDecode` impls for both libraries
2. **Integration tests**: Test handlers with both prost and quick-protobuf messages
3. **Backward compatibility tests**: Ensure existing prost-based code compiles and works
4. **Feature flag tests**: Test with and without `quick-protobuf` feature

## Migration Notes for Users

### Prost Users (No Changes Required)
Existing code continues to work:
```rust
#[substreams::handlers::map]
fn map_blocks(blk: Block) -> Result<MyOutput, Error> {
    // Works exactly as before
}
```

### Quick-Protobuf Users
Add the macro option:
```rust
#[substreams::handlers::map(quick_protobuf)]
fn map_blocks(blk: Block) -> Result<MyOutput, Error> {
    // Uses quick-protobuf for decoding/encoding
}
```

---

## Key Challenges & Mitigations

### Challenge 1: Lifetime in `MessageRead<'a>`

Quick-protobuf's `MessageRead<'a>` has a lifetime parameter for borrowing from input bytes. For owned data in substreams context, we need `'static`.

**Mitigation**: Use `MessageRead<'static>` bound, which works for types with no borrowed fields (common in generated code).

### Challenge 2: Conflicting Trait Implementations

Can't have both blanket impls:
```rust
impl<T: prost::Message> ProtoEncode for T { ... }
impl<T: quick_protobuf::MessageWrite> ProtoEncode for T { ... }
```

**Recommended Solution: Avoid Blanket Impls**

Instead of blanket implementations, require users to explicitly derive or implement the traits. This is cleaner and avoids coherence issues:

```rust
// In substreams crate - define the traits
pub trait ProtoEncode {
    fn encode_to_vec(&self) -> Result<Vec<u8>, EncodeError>;
}

pub trait ProtoDecode: Sized {
    fn decode_from_slice(bytes: &[u8]) -> Result<Self, DecodeError>;
}

// Provide derive macros or helper macros for common cases
// Users add to their generated protobuf types:
impl ProtoEncode for MyProstMessage {
    fn encode_to_vec(&self) -> Result<Vec<u8>, EncodeError> {
        substreams::proto::prost::encode(self)
    }
}
```

**Alternative: Extension Trait Pattern**

Keep the original `output<M: prost::Message>` and add a new function:

```rust
// Original - unchanged for backward compatibility
pub fn output<M: prost::Message>(msg: M) { ... }

// New - for quick-protobuf users
pub fn output_quick<M: quick_protobuf::MessageWrite>(msg: &M) { ... }

// Or use output_raw for any pre-encoded bytes
pub fn output_raw(data: Vec<u8>) { ... }  // Already exists!
```

**Simplest Practical Approach (Recommended)**:

Since `output_raw(Vec<u8>)` already exists, quick-protobuf users can:
```rust
let bytes = quick_protobuf::serialize_into_vec(&my_message)?;
substreams::output_raw(bytes);
```

The macro can generate this pattern when `quick_protobuf` option is set.

### Challenge 3: Macro Hygiene

The macro generates code that references `substreams::proto::decode_ptr`. Need to handle both backends.

**Mitigation**: The macro option `quick_protobuf` controls which decoding code is generated.

---

## Files to Modify

| File | Changes |
|------|---------|
| `substreams/Cargo.toml` | Add optional `quick-protobuf` dependency, feature flag |
| `substreams/src/lib.rs` | Export new traits, update `output()` |
| `substreams/src/pb_compat.rs` | NEW: `ProtoEncode`, `ProtoDecode` traits + impls |
| `substreams/src/proto.rs` | Add generic functions, deprecate prost-specific ones |
| `substreams/src/store.rs` | Update `*Proto` types to use generic traits |
| `substreams-macro/src/config.rs` | Add `quick_protobuf` option |
| `substreams-macro/src/handler.rs` | Generate different code based on option |

---

## Concrete Code Changes (Simpler Approach)

### 1. `substreams/Cargo.toml`

```toml
[features]
default = []
quick-protobuf = ["dep:quick-protobuf"]

[dependencies]
prost = "0.13"
quick-protobuf = { version = "0.8", optional = true }
```

### 2. `substreams/src/lib.rs` - Add quick-protobuf output

```rust
// Existing - unchanged
pub fn output<M: prost::Message>(msg: M) { ... }
pub fn output_raw(data: Vec<u8>) { ... }

// NEW - for quick-protobuf
#[cfg(feature = "quick-protobuf")]
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn output_quick<M: quick_protobuf::MessageWrite>(msg: &M) {
    #[cfg(target_arch = "wasm32")]
    {
        use quick_protobuf::Writer;
        let size = msg.get_size();
        let mut buffer = Vec::with_capacity(size);
        let mut writer = Writer::new(&mut buffer);
        msg.write_message(&mut writer)
            .unwrap_or_else(|_| panic!("Unable to encode quick-protobuf message"));
        output_raw(buffer);
    }
}

// NEW - quick-protobuf module
#[cfg(feature = "quick-protobuf")]
pub mod quick {
    use quick_protobuf::{BytesReader, MessageRead, Result as QpResult};

    pub fn decode<'a, T: MessageRead<'a>>(bytes: &'a [u8]) -> QpResult<T> {
        let mut reader = BytesReader::from_bytes(bytes);
        T::from_reader(&mut reader, bytes)
    }

    /// Decode from raw pointer - for WASM interop
    /// SAFETY: ptr must be valid for `size` bytes
    pub unsafe fn decode_ptr<T: for<'a> MessageRead<'a>>(
        ptr: *mut u8,
        size: usize,
    ) -> QpResult<T> {
        let bytes = std::slice::from_raw_parts(ptr, size);
        let mut reader = BytesReader::from_bytes(bytes);
        T::from_reader(&mut reader, bytes)
    }
}
```

### 3. `substreams-macro/src/config.rs` - Add option

```rust
#[derive(Clone, Copy, Default)]
pub struct HandlerOptions {
    pub keep_empty_output: bool,
    pub no_testable: bool,
    pub quick_protobuf: bool,  // NEW
}

impl HandlerOptions {
    pub fn parse(args: &str) -> Result<Self, String> {
        let mut options = Self::default();
        if args.is_empty() {
            return Ok(options);
        }
        for part in args.split(',') {
            match part.trim() {
                "no_testable" => options.no_testable = true,
                "keep_empty_output" => options.keep_empty_output = true,
                "quick_protobuf" => options.quick_protobuf = true,  // NEW
                other => return Err(format!("Unknown option '{}'", other)),
            }
        }
        Ok(options)
    }
}
```

### 4. `substreams-macro/src/handler.rs` - Generate different code

```rust
// In build_map_handler, modify proto_decodings based on options.quick_protobuf:

if options.quick_protobuf {
    proto_decodings.push(quote! {
        let #mutability #var_name: #argument_type = unsafe {
            substreams::quick::decode_ptr(#var_ptr, #var_len)
        }.unwrap_or_else(|_| panic!(
            "Unable to decode quick-protobuf data ({} bytes) to '{}' message",
            #var_len, stringify!(#argument_type)
        ));
    });
} else {
    // Existing prost decoding
    proto_decodings.push(quote! {
        let #mutability #var_name: #argument_type = substreams::proto::decode_ptr(#var_ptr, #var_len)
            .unwrap_or_else(|_| panic!(...));
    });
}

// For output handling:
let output_call = if options.quick_protobuf {
    quote! { substreams::output_quick(&result); }
} else {
    quote! { substreams::output(result); }
};
```

### 5. Usage Example

```rust
// Prost (default) - unchanged
#[substreams::handlers::map]
fn map_blocks(blk: Block) -> Result<MyOutput, Error> {
    Ok(MyOutput { ... })
}

// Quick-protobuf - new option
#[substreams::handlers::map(quick_protobuf)]
fn map_blocks(blk: Block) -> Result<MyOutput, Error> {
    Ok(MyOutput { ... })
}
```

---

## Status

- [x] Codebase analysis complete
- [x] Quick-protobuf API research complete
- [x] Design document complete
- [x] Phase 1: Add feature flag and dependency
- [x] Phase 2: Add output_quick() function
- [x] Phase 3: Add quick module with decode helpers
- [x] Phase 4: Update macro config
- [x] Phase 5: Update macro code generation
- [x] Testing (all 214 tests pass)
- [ ] Documentation (in-code documentation added)
