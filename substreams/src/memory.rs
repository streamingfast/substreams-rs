use std::{convert::TryInto, slice};

#[no_mangle]
pub fn alloc(size: usize) -> *mut u8 {
    let mut buf = Vec::with_capacity(size);
    let ptr = buf.as_mut_ptr();

    // take ownership of the memory block and
    // ensure the its destructor is not
    // called when the object goes out of scope
    // at the end of the function
    std::mem::forget(buf);
    // return the pointer so the runtime
    // can write data at this offset
    ptr
}

/// Retakes the pointer which allows its memory to be freed.
#[no_mangle]
pub unsafe fn dealloc(ptr: *mut u8, size: usize) {
    let data = Vec::from_raw_parts(ptr, size, size);
    std::mem::drop(data);
}

#[deprecated(
    since = "0.5.20",
    note = "Use 'read_output_data' instead with your own allocated output data"
)]
pub unsafe fn get_output_data(output_ptr: *mut u8) -> Vec<u8> {
    let value_ptr: u32 = read_u32(output_ptr, 4);
    let value_len: u32 = read_u32(output_ptr.add(4), 4);

    Vec::from_raw_parts(value_ptr as *mut u8, value_len as usize, value_len as usize)
}

unsafe fn read_u32(output_ptr: *mut u8, len: usize) -> u32 {
    let value_bytes = slice::from_raw_parts(output_ptr, len);
    let value_raw_bytes: [u8; 4] = value_bytes.try_into().expect("error reading raw bytes");
    return u32::from_le_bytes(value_raw_bytes);
}

/// Low-level API method that should not be used unless interacting directly with the Substreams
/// runtime intrinsics that writes to memory.
///
/// Substreams intrinsics that returns data are implemented by receiving a pointer to a 8 bytes
/// long memory segment; a [u8; 8] array that we are going to call the output data. Substreams
/// perform its operation and then allocate N bytes in memory to hold the data that needs to be returned
/// to the caller. It will then write the pointer address to the start of the output data and the length
/// of the data to the next 4 bytes.
///
/// This method reads the output data ptr (first 4 bytes) and the length of the data (next 4 bytes) and
/// returns a Vec<u8> with the data that was written to memory by the Substreams instrinsic.
///
/// # Safety
///
/// This method should only be called after a Substreams intrinsic that returns data has been called
/// and the received `output_data` was passed to the intrinsic as the output data pointer.
///
/// Any other usage will lead to undefined behavior.
pub unsafe fn read_output_data(output_data: &[u8; 8]) -> Vec<u8> {
    let value_ptr: u32 = u32::from_le_bytes(
        output_data[0..4]
            .try_into()
            .expect("error reading raw bytes for ptr"),
    );

    let value_len: u32 = u32::from_le_bytes(
        output_data[4..8]
            .try_into()
            .expect("error reading raw bytes for len"),
    );

    Vec::from_raw_parts(value_ptr as *mut u8, value_len as usize, value_len as usize)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_output_data() {
        let output_data: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
        let result = unsafe { read_output_data(&output_data) };
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_read_output_data_with_data() {
        // We cannot make a test with real data. This is because the function is expected to
        // run on 32 bits address space (pointer must be 4 bytes long) and we are running on 64 bits
        // address space (pointer is 8 bytes long). This is a limitation of the test environment.
    }
}
