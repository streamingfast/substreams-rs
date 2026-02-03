# Implementation Plan: Substreams Test Macro

## ULTIMATE GOAL

Enable easy unit testing of Substreams handler functions by providing macro-based tooling that allows users to test their handlers with normal Rust types (not WASM pointers) without requiring them to write separate wrapper functions. The solution should work seamlessly with the existing `#[substreams::handlers::map]` macro pattern and minimize changes to existing user code.

## Status: Completed

---

## Design Decisions (Confirmed)

1. **WASM Export Naming**: Keep original name for WASM export (manifest compatibility required)
2. **Testable Function Naming**: Use `__impl_<name>` prefix for the testable function
3. **Scope for v1**: Only support `String` params and mapper inputs (protobuf messages) - NO store support yet
4. **Test Harness Macro**: Function-like macro `substreams::test_map!(handler(args))` that rewrites to `__impl_handler(args)`
5. **Opt-in via attribute**: Feature is opt-in initially, will become default later

### Opt-in Attribute

```rust
// Opt-in to generate __impl_ function (v1 behavior)
#[substreams::handlers::map(testable)]
fn map_transfers(blk: eth::Block) -> Result<Events, Error> { ... }

// Without attribute - current behavior (no __impl_ generated)
#[substreams::handlers::map]
fn map_transfers(blk: eth::Block) -> Result<Events, Error> { ... }
```

**Rollout plan:**
1. **Phase 1 (now)**: `testable` is opt-in, default is current behavior
2. **Phase 2 (later)**: `testable` becomes default, users can opt-out with `testable = false` if needed

### Why Function-Like Macro (Not Attribute)

Rust attribute macros (`#[attr] expr`) on expressions are unstable. We must use a function-like macro:

```rust
// This is NOT valid stable Rust:
#[substreams::testing::map]
map_events(blk)

// This IS valid - function-like macro:
substreams::test_map!(map_events(blk))
```

---

## Phase 1: Codebase Analysis (Completed)

### 1.1 Handler Macro Analysis - DONE

**Location**: `substreams-macro/src/handler.rs` and `substreams-macro/src/lib.rs`

**Key Findings**:

1. **Current Macro Behavior**: The `#[substreams::handlers::map]` and `#[substreams::handlers::store]` macros completely replace the user's function signature:

   ```rust
   // User writes:
   fn map_transfers(blk: eth::Block) -> Result<pb::Custom, Error> { ... }

   // Macro generates:
   #[no_mangle]
   pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
       substreams::register_panic_hook();
       let func = || -> Result<pb::Custom, Error> {
           let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len).unwrap_or_else(...);
           // user's code body
       };
       let result = func();
       // output handling based on return type
       substreams::output(result);
   }
   ```

2. **Input Types Handled**:
   - Protobuf messages (decoded via `proto::decode_ptr`)
   - Writable stores (e.g., `StoreAddInt64`) - instantiated via `::new()`
   - Readable stores (e.g., `StoreGetProto<T>`) - instantiated via `::new(store_idx)`
   - FoundationalStore - instantiated via `::new(store_idx)`
   - Deltas - decoded from `StoreDeltas` protobuf
   - String - reconstructed from raw parts

3. **Output Types Supported**:
   - `Result<T, Error>` - panics on error, outputs T
   - `Result<Option<T>, Error>` - panics on error, outputs T if Some
   - `Option<T>` - outputs T if Some
   - `T` (plain value) - outputs directly
   - `void` (for store handlers only)

4. **Key Functions Used**:
   - `substreams::register_panic_hook()` - sets up panic handler
   - `substreams::proto::decode_ptr(ptr, len)` - decodes protobuf from pointer
   - `substreams::output(msg)` - writes output to WASM host
   - `substreams::skip_empty_output()` - signals no output expected

### 1.2 Output Mechanism Analysis - DONE

**Location**: `substreams/src/lib.rs` (lines 168-201), `substreams/src/externs.rs`

**Key Findings**:

1. **`substreams::output<M: prost::Message>(msg: M)`**:
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
   - On WASM: encodes message and calls host `output` function
   - On non-WASM: **function does nothing** (empty body)

2. **`substreams::skip_empty_output()`**:
   - On WASM: calls `externs::skip_empty_output()`
   - On non-WASM: **function does nothing**

3. **External Functions** (`substreams/src/externs.rs`):
   - Only linked when `target_arch = "wasm32"`
   - `output(ptr, len)` - writes output to host
   - `skip_empty_output()` - signals empty output acceptable

### 1.3 WASM vs Non-WASM Patterns - DONE

**Location**: `substreams/src/state.rs`, `substreams/src/lib.rs`, `substreams/src/externs.rs`

**Pattern Used Throughout**:

```rust
#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn some_function(args...) {
    #[cfg(target_arch = "wasm32")]
    {
        // WASM implementation
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Non-WASM stub (often returns None/false/empty)
    }
}
```

**Key Observations**:
- All store operations (`get_at`, `set`, etc.) are no-ops on non-WASM
- `output()` is a no-op on non-WASM
- `register_panic_hook()` is a no-op on non-WASM
- Proto encoding/decoding works on both targets

### 1.4 Current Testing Limitations - DONE

**The Problem**:
1. Generated WASM handlers have `extern "C"` signatures with raw pointers
2. These cannot be called from regular Rust tests
3. Users must extract logic into separate functions manually:
   ```rust
   // User workaround pattern:
   fn _map_transfers_impl(blk: eth::Block) -> Result<pb::Custom, Error> {
       // actual logic
   }

   #[substreams::handlers::map]
   fn map_transfers(blk: eth::Block) -> Result<pb::Custom, Error> {
       _map_transfers_impl(blk)
   }

   #[test]
   fn test_map_transfers() {
       let block = eth::Block { ... };
       let result = _map_transfers_impl(block);
       assert!(result.is_ok());
   }
   ```

---

## Phase 2: Design Options

### Final Design: Dual Function + Test Macro

**Concept**: Modify the `#[substreams::handlers::map]` macro to generate TWO functions, plus provide a convenient test macro.

**Generated Code Example**:
```rust
// User writes (with testable attribute):
#[substreams::handlers::map(testable)]
fn map_transfers(blk: eth::Block) -> Result<pb::Custom, Error> {
    // logic
}

// Macro generates:

// 1. Testable function with __impl_ prefix (always generated when testable)
pub fn __impl_map_transfers(blk: eth::Block) -> Result<pb::Custom, Error> {
    // user's original logic body
}

// 2. WASM export with original name (only on wasm32)
#[cfg(target_arch = "wasm32")]
#[no_mangle]
pub extern "C" fn map_transfers(blk_ptr: *mut u8, blk_len: usize) {
    substreams::register_panic_hook();
    let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len).unwrap_or_else(...);
    let result = __impl_map_transfers(blk);  // calls the testable function
    // handle result and output
}

// Without testable attribute - current behavior unchanged:
#[substreams::handlers::map]
fn other_handler(blk: eth::Block) -> Result<pb::Custom, Error> { ... }
// Generates only the WASM export (current behavior)
```

**Test Harness Macro**:
```rust
// In tests, user writes:
let result = substreams::test_map!(map_transfers(blk));

// Macro expands to:
let result = __impl_map_transfers(blk);
```

**Advantages**:
- WASM export keeps original name (manifest compatibility)
- Clean test syntax via macro
- No serialization overhead in tests
- Extensible for future store support

**Scope for v1**:
- Only protobuf message inputs (mapper inputs)
- Only `String` params
- NO store inputs (deferred to v2)

---

## Implementation Tasks

### Priority 1: Macro Modifications (Core)

- [x] **Task 1.1**: Parse `testable` attribute in handler macro
  - Add attribute parsing for `#[substreams::handlers::map(testable)]`
  - When `testable` is present: generate dual functions
  - When `testable` is absent: current behavior (no change)
  - Prepare for future: easy to flip default and support `testable = false`

- [x] **Task 1.2**: Generate dual functions when `testable` is set
  - Generate `__impl_<name>` function with original signature
  - Generate WASM export with original name (only on `wasm32`)
  - WASM export calls `__impl_<name>` internally
  - Use `#[cfg(target_arch = "wasm32")]` for WASM-only parts
  - Preserve all existing output type handling

- [x] **Task 1.3**: Support only v1 input types (when `testable`)
  - Protobuf messages (mapper inputs) - decode and pass to `__impl_`
  - `String` params - reconstruct and pass to `__impl_`
  - **Compile error** if handler has store parameters: `"Store parameters not yet supported for testing. Extract your logic into a separate function to test it."`

- [x] **Task 1.4**: Handle all output types correctly
  - `Result<T, Error>` - `__impl_` returns this, WASM wrapper handles panic/output
  - `Result<Option<T>, Error>` - same pattern
  - `Option<T>` - same pattern
  - Plain `T` - same pattern

### Priority 2: Test Harness Macro

- [x] **Task 2.1**: Create `substreams::test_map!` macro in `substreams-macro`
  - Parse call expression: `test_map!(handler(arg1, arg2))`
  - Transform to: `__impl_handler(arg1, arg2)`
  - Simple token transformation, no type analysis needed

- [ ] **Task 2.2**: Add compile-time validation (optional) - DEFERRED
  - Emit helpful error if macro used on non-existent `__impl_` function
  - Consider: should macro verify handler exists at compile time?

### Priority 3: Documentation

- [x] **Task 3.1**: Update handler macro documentation
  - Document `__impl_` function generation
  - Document `test_map!` macro usage
  - Show examples of unit testing handlers

- [x] **Task 3.2**: Add inline examples in macro docs
  - Example handler with test
  - Common testing patterns

### Future (v2) - Store Support

- [ ] **Task F.1**: Create mock store infrastructure
  - Define `MockStoreGet<T>` that wraps a `HashMap`
  - Define `MockStoreSet` traits for writable stores
  - Allow test code to inject store state

- [ ] **Task F.2**: Extend `__impl_` functions to accept generic store params
  - For writable stores: accept `impl StoreSet<T>` or concrete mock
  - For readable stores: accept `impl StoreGet<T>` or concrete mock

- [ ] **Task F.3**: Extend `test_map!` to support store injection
  - `test_map!(handler(blk), stores: { my_store: mock_data })`

---

## Technical Details

### Dual Function Generation (handler.rs)

```rust
// In handler.rs, build_map_handler modification:

fn build_map_handler(...) -> TokenStream {
    let func_name = &item_fn.sig.ident;
    let impl_func_name = format_ident!("__impl_{}", func_name);

    // Generate the testable function (always, all targets)
    let testable_func = quote! {
        pub fn #impl_func_name(#original_args) #return_type {
            #body
        }
    };

    // Generate the WASM export (only on wasm32)
    let wasm_export = quote! {
        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        pub extern "C" fn #func_name(#wasm_args) {
            substreams::register_panic_hook();
            #decodings  // decode protobuf from pointers
            let result = #impl_func_name(#call_args);
            #output_handler  // handle Result/Option, call substreams::output()
        }
    };

    quote! {
        #testable_func
        #wasm_export
    }
}
```

### Test Macro Implementation (substreams-macro)

```rust
// substreams-macro/src/lib.rs

/// Transforms `test_map!(handler(args))` into `__impl_handler(args)`
#[proc_macro]
pub fn test_map(input: TokenStream) -> TokenStream {
    let expr: syn::ExprCall = syn::parse(input)
        .expect("expected function call expression");

    // Extract function name from the call
    let func_path = match &*expr.func {
        syn::Expr::Path(path) => path,
        _ => panic!("expected path expression"),
    };

    // Get the last segment (function name)
    let mut new_path = func_path.clone();
    if let Some(last) = new_path.path.segments.last_mut() {
        let new_name = format_ident!("__impl_{}", last.ident);
        last.ident = new_name;
    }

    // Reconstruct the call with modified name
    let args = &expr.args;
    quote! {
        #new_path(#args)
    }.into()
}
```

### Input Type Handling (v1 Scope)

**Supported in v1:**
1. **Protobuf messages** - Type implements `prost::Message + Default`
   - `__impl_` receives decoded type directly
   - WASM wrapper decodes from pointer

2. **String params** - `String` type
   - `__impl_` receives `String` directly
   - WASM wrapper reconstructs from raw parts

**NOT supported in v1 (compile error):**
- Store types (`StoreAddInt64`, `StoreGetProto<T>`, etc.)
- Delta types
- FoundationalStore

Error message: `"Store parameters not yet supported for testing. Extract your logic into a separate function to test it."`

---

## Migration Guide (for Users)

### Before (Current)
```rust
// Users must manually split logic for testing
fn _map_transfers_impl(blk: eth::Block) -> Result<Events, Error> {
    // logic here
}

#[substreams::handlers::map]
fn map_transfers(blk: eth::Block) -> Result<Events, Error> {
    _map_transfers_impl(blk)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_map_transfers() {
        let blk = eth::Block::default();
        let result = _map_transfers_impl(blk);
        assert!(result.is_ok());
    }
}
```

### After (With This Feature)
```rust
// Add `testable` attribute to enable testing support
#[substreams::handlers::map(testable)]
fn map_transfers(blk: eth::Block) -> Result<Events, Error> {
    // logic here
}

#[cfg(test)]
mod tests {
    use substreams::test_map;

    #[test]
    fn test_map_transfers() {
        let blk = eth::Block::default();

        // Option 1: Use the test_map! macro (recommended)
        let result = test_map!(map_transfers(blk));
        assert!(result.is_ok());

        // Option 2: Call __impl_ directly (if you prefer)
        let result = __impl_map_transfers(blk);
        assert!(result.is_ok());
    }
}
```

### What Changes for Existing Code?

**No breaking changes in Phase 1**: The `testable` attribute is opt-in. Existing code without `testable` continues to work exactly as before.

**When you add `testable`**:
- A new `__impl_<name>` function is generated alongside the WASM export
- Use `test_map!(handler(args))` or `__impl_handler(args)` in tests

**Future (Phase 2)**: When `testable` becomes the default, users can opt-out with `testable = false` if needed.

---

## Resolved Questions

1. **WASM Export Naming**: ✅ Keep original name for WASM export (manifest compatibility required)
   - Testable function uses `__impl_<name>` prefix

2. **Store Support**: ✅ Deferred to v2
   - v1 only supports protobuf messages and String params
   - Emit warning/error if handler has store parameters

3. **Test Macro Syntax**: ✅ Function-like macro (attribute macros on expressions are unstable)
   - `substreams::test_map!(handler(args))`

## Open Questions (Remaining)

1. **Feature Flag**: Should test support require a feature flag?
   - Consideration: The `__impl_` functions are always generated, no feature flag needed
   - The `test_map!` macro is just token transformation, no runtime cost
   - **Likely answer**: No feature flag needed for v1

## Resolved (Additional)

2. **Store parameter handling in v1**: ✅ Option A - Compile error with helpful message
   - If handler has store parameters, emit: `"Store parameters not yet supported for testing. Extract your logic into a separate function to test it."`
   - This keeps v1 simple and gives users a clear workaround

---

## Completed Items

- [x] Analyzed handler macro implementation (`substreams-macro/src/handler.rs`)
- [x] Analyzed output mechanism (`substreams/src/lib.rs`)
- [x] Mapped WASM vs non-WASM conditional compilation patterns
- [x] Identified current testing limitations and workarounds
- [x] Designed final solution (dual function + test macro)
- [x] Created detailed implementation task list
- [x] Refined design based on user feedback

---

## Notes

- Analysis completed: 2026-01-30
- Design finalized: 2026-01-30
- The codebase already has good separation between WASM and non-WASM code paths
- The `proto::encode` and `proto::decode` functions work on both targets
- Since `__impl_` functions return the result directly (not calling `substreams::output()`), no output capture infrastructure is needed for v1
