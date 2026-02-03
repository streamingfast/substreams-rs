#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    pub fn skip_empty_output();
    pub fn output(ptr: *const u8, len: u32);
    pub fn register_panic(
        msg_ptr: *const u8,
        msg_len: u32,
        file_ptr: *const u8,
        file_len: u32,
        line: u32,
        column: u32,
    );
}

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "logger")]
extern "C" {
    pub fn println(ptr: *const u8, len: usize);
}

// #[no_mangle]
// pub extern "C" fn map_handler(blk_ptr: *mut u8, blk_len: usize) {
//     let func = || -> Option<Custom> {
//         let blk: eth::Block = substreams::proto::decode_ptr(blk_ptr, blk_len).unwrap();
//         {
//             unimplemented!("do something");
//         }
//     };
//     let result = func();
//     if result.is_none() {
//         panic!("None returned from map function")
//     }
//     substreams::output(result.unwrap());
// }
// #[derive(Debug)]
// pub struct Custom(u8);
// impl prost::Message for Custom {
//     fn encode_raw(&self, _: &mut impl prost::bytes::BufMut)
//     where
//         Self: Sized,
//     {
//         todo!()
//     }
//     fn merge_field(
//         &mut self,
//         _: u32,
//         _: prost::encoding::WireType,
//         _: &mut impl prost::bytes::Buf,
//         _: prost::encoding::DecodeContext,
//     ) -> Result<(), prost::DecodeError>
//     where
//         Self: Sized,
//     {
//         todo!()
//     }
//     fn encoded_len(&self) -> usize {
//         todo!()
//     }
//     fn clear(&mut self) {
//         todo!()
//     }
// }

pub mod state {
    #[cfg(target_arch = "wasm32")]
    #[link(wasm_import_module = "state")]
    extern "C" {
        pub fn foundational_store_get_entries(store_index: u32, req_ptr: u32, req_len: u32) -> u64;
        pub fn foundational_store_get_first_entries(
            store_index: u32,
            req_ptr: u32,
            req_len: u32,
        ) -> u64;
        pub fn get_first(store_idx: u32, key_ptr: *const u8, key_len: u32, output_ptr: u32) -> u32;
        pub fn get_last(store_idx: u32, key_ptr: *const u8, key_len: u32, output_ptr: u32) -> u32;
        pub fn get_at(
            store_idx: u32,
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            output_ptr: u32,
        ) -> u32;
        pub fn has_first(store_idx: u32, key_ptr: *const u8, key_len: u32) -> u32;
        pub fn has_last(store_idx: u32, key_ptr: *const u8, key_len: u32) -> u32;
        pub fn has_at(store_idx: u32, ord: i64, key_ptr: *const u8, key_len: u32) -> u32;
        pub fn set(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
        pub fn set_if_not_exists(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
        pub fn append(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
        pub fn delete_prefix(ord: i64, prefix_ptr: *const u8, prefix_len: u32);
        pub fn add_bigint(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
        pub fn add_int64(ord: i64, key_ptr: *const u8, key_len: u32, value: i64);
        pub fn add_float64(ord: i64, key_ptr: *const u8, key_len: u32, value: f64);
        pub fn add_bigdecimal(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
        pub fn set_min_int64(ord: i64, key_ptr: *const u8, key_len: u32, value: i64);
        pub fn set_min_bigint(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
        pub fn set_min_float64(ord: i64, key_ptr: *const u8, key_len: u32, value: f64);
        pub fn set_min_bigdecimal(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
        pub fn set_max_int64(ord: i64, key_ptr: *const u8, key_len: u32, value: i64);
        pub fn set_max_bigint(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
        pub fn set_max_float64(ord: i64, key_ptr: *const u8, key_len: u32, value: f64);
        pub fn set_max_bigdecimal(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
        pub fn set_sum_bigint(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
        pub fn set_sum_bigdecimal(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
        pub fn set_sum_int64(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
        pub fn set_sum_float64(
            ord: i64,
            key_ptr: *const u8,
            key_len: u32,
            value_ptr: *const u8,
            value_len: u32,
        );
    }
}
