use crate::scalar::{BigDecimal, BigInt};
#[cfg(target_arch = "wasm32")]
use crate::{externs, memory};

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn get_at<K: AsRef<str>>(store_idx: u32, ord: i64, key: K) -> Option<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();

        unsafe {
            let key_bytes = key.as_bytes();
            let output_ptr = memory::alloc(8);
            let found = externs::state::get_at(
                store_idx,
                ord,
                key_bytes.as_ptr(),
                key_bytes.len() as u32,
                output_ptr as u32,
            );
            return if found == 1 {
                Some(memory::get_output_data(output_ptr))
            } else {
                None
            };
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    None
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn get_last<K: AsRef<str>>(store_idx: u32, key: K) -> Option<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();

        unsafe {
            let key_bytes = key.as_bytes();
            let output_ptr = memory::alloc(8);
            let found = externs::state::get_last(
                store_idx,
                key_bytes.as_ptr(),
                key_bytes.len() as u32,
                output_ptr as u32,
            );

            return if found == 1 {
                Some(memory::get_output_data(output_ptr))
            } else {
                None
            };
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    None
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn get_first<K: AsRef<str>>(store_idx: u32, key: K) -> Option<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();

        unsafe {
            let key_bytes = key.as_bytes();
            let output_ptr = memory::alloc(8);

            let found = externs::state::get_first(
                store_idx,
                key_bytes.as_ptr(),
                key_bytes.len() as u32,
                output_ptr as u32,
            );

            return if found == 1 {
                Some(memory::get_output_data(output_ptr))
            } else {
                None
            };
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    None
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn set<K, V>(ord: i64, key: K, value: V)
where
    K: AsRef<str>,
    V: AsRef<[u8]>,
{
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();
        let value = value.as_ref();

        unsafe {
            externs::state::set(
                ord,
                key.as_ptr(),
                key.len() as u32,
                value.as_ptr(),
                value.len() as u32,
            )
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn set_if_not_exists<K, V>(ord: i64, key: K, value: V)
where
    K: AsRef<str>,
    V: AsRef<[u8]>,
{
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();
        let value = value.as_ref();

        unsafe {
            externs::state::set_if_not_exists(
                ord,
                key.as_ptr(),
                key.len() as u32,
                value.as_ptr(),
                value.len() as u32,
            )
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn append<K, V>(ord: i64, key: K, value: V)
where
    K: AsRef<str>,
    V: AsRef<[u8]>,
{
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();
        let value = value.as_ref();

        unsafe {
            externs::state::append(
                ord,
                key.as_ptr(),
                key.len() as u32,
                value.as_ptr(),
                value.len() as u32,
            )
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn delete_prefix<K: AsRef<str>>(ord: i64, prefix: K) {
    #[cfg(target_arch = "wasm32")]
    {
        let prefix = prefix.as_ref();

        unsafe { externs::state::delete_prefix(ord, prefix.as_ptr(), prefix.len() as u32) }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn add_bigint<K, V>(ord: i64, key: K, value: V)
where
    K: AsRef<str>,
    V: AsRef<BigInt>,
{
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();
        let big_int = value.as_ref();
        let data: String = big_int.into();

        unsafe {
            externs::state::add_bigint(
                ord,
                key.as_ptr(),
                key.len() as u32,
                data.as_ptr(),
                data.len() as u32,
            )
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn add_int64<K: AsRef<str>>(ord: i64, key: K, value: i64) {
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();

        unsafe { externs::state::add_int64(ord, key.as_ptr(), key.len() as u32, value) }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn add_float64<K: AsRef<str>>(ord: i64, key: K, value: f64) {
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();

        unsafe { externs::state::add_float64(ord, key.as_ptr(), key.len() as u32, value) }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn add_bigdecimal<K, V>(ord: i64, key: K, value: V)
where
    K: AsRef<str>,
    V: AsRef<BigDecimal>,
{
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();
        let big_decimal = value.as_ref();
        let data: String = big_decimal.into();

        unsafe {
            externs::state::add_bigdecimal(
                ord,
                key.as_ptr(),
                key.len() as u32,
                data.as_ptr(),
                data.len() as u32,
            )
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn set_min_int64<K: AsRef<str>>(ord: i64, key: K, value: i64) {
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();

        unsafe { externs::state::set_min_int64(ord, key.as_ptr(), key.len() as u32, value) }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn set_min_bigint<K, V>(ord: i64, key: K, value: V)
where
    K: AsRef<str>,
    V: AsRef<BigInt>,
{
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();
        let big_int = value.as_ref();
        let data: String = big_int.into();

        unsafe {
            externs::state::set_min_bigint(
                ord,
                key.as_ptr(),
                key.len() as u32,
                data.as_ptr(),
                data.len() as u32,
            )
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn set_min_float64<K: AsRef<str>>(ord: i64, key: K, value: f64) {
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();

        unsafe { externs::state::set_min_float64(ord, key.as_ptr(), key.len() as u32, value) }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn set_min_bigdecimal<K, V>(ord: i64, key: K, value: V)
where
    K: AsRef<str>,
    V: AsRef<BigDecimal>,
{
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();
        let big_decimal = value.as_ref();
        let data: String = big_decimal.into();

        unsafe {
            externs::state::set_min_bigdecimal(
                ord,
                key.as_ptr(),
                key.len() as u32,
                data.as_ptr(),
                data.len() as u32,
            )
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn set_max_int64<K: AsRef<str>>(ord: i64, key: K, value: i64) {
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();

        unsafe { externs::state::set_max_int64(ord, key.as_ptr(), key.len() as u32, value) }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn set_max_bigint<K, V>(ord: i64, key: K, value: V)
where
    K: AsRef<str>,
    V: AsRef<BigInt>,
{
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();
        let big_int = value.as_ref();
        let data: String = big_int.into();

        unsafe {
            externs::state::set_max_bigint(
                ord,
                key.as_ptr(),
                key.len() as u32,
                data.as_ptr(),
                data.len() as u32,
            )
        }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn set_max_float64<K: AsRef<str>>(ord: i64, key: K, value: f64) {
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();

        unsafe { externs::state::set_max_float64(ord, key.as_ptr(), key.len() as u32, value) }
    }
}

#[cfg_attr(not(target_arch = "wasm32"), allow(unused_variables))]
pub fn set_max_bigdecimal<K, V>(ord: i64, key: K, value: V)
where
    K: AsRef<str>,
    V: AsRef<BigDecimal>,
{
    #[cfg(target_arch = "wasm32")]
    {
        let key = key.as_ref();
        let big_decimal = value.as_ref();
        let data: String = big_decimal.into();

        unsafe {
            externs::state::set_max_bigdecimal(
                ord,
                key.as_ptr(),
                key.len() as u32,
                data.as_ptr(),
                data.len() as u32,
            )
        }
    }
}
