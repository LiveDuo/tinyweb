
use std::mem::ManuallyDrop;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExternRef { pub value: u32, }

pub enum InvokeParam<'a> {
    Undefined,
    Null,
    Float64(f64),
    BigInt(i64),
    String(&'a str),
    ExternRef(&'a ExternRef),
    Float32Array(&'a [f32]),
    Float64Array(&'a [f64]),
    Bool(bool),
    Uint32Array(&'a [u32]),
}

impl<'a> InvokeParam<'a> {

    // preceded by 1 byte integer indicating its type
    pub fn serialize(&self) -> Vec<u8> {
        let mut param_bytes = Vec::new();
        match self {
            InvokeParam::Undefined => {
                param_bytes.push(0);
            }
            InvokeParam::Null => {
                param_bytes.push(1);
            }
            InvokeParam::Float64(f) => {
                param_bytes.push(2);
                param_bytes.extend_from_slice(&f.to_le_bytes());
            }
            InvokeParam::BigInt(i) => {
                param_bytes.push(3);
                param_bytes.extend_from_slice(&i.to_le_bytes());
            }
            InvokeParam::String(s) => {
                param_bytes.push(4);
                let ptr = s.as_ptr() as u32;
                let len = s.len();
                param_bytes.extend_from_slice(&ptr.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
            InvokeParam::ExternRef(i) => {
                param_bytes.push(5);
                param_bytes.extend_from_slice(&i.value.to_le_bytes());
            }
            InvokeParam::Float32Array(a) => {
                param_bytes.push(6);
                let ptr = a.as_ptr() as u32;
                let len = a.len();
                param_bytes.extend_from_slice(&ptr.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
            InvokeParam::Bool(b) => {
                param_bytes.push(if *b { 7 } else { 8 });
            }
            InvokeParam::Float64Array(a) => {
                param_bytes.push(9);
                let ptr = a.as_ptr() as u32;
                let len = a.len();
                param_bytes.extend_from_slice(&ptr.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
            InvokeParam::Uint32Array(a) => {
                param_bytes.push(10);
                let ptr = a.as_ptr() as u32;
                let len = a.len();
                param_bytes.extend_from_slice(&ptr.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
        }
        param_bytes
    }
}

pub fn serialize_params(params: &[InvokeParam]) -> Vec<u8> {
    let mut param_bytes = Vec::new();
    for param in params {
        let bytes = param.serialize();
        param_bytes.extend_from_slice(&bytes);
    }
    param_bytes
}

#[cfg(not(test))]
extern "C" {
    fn __invoke_and_return_number(c_ptr: *const u8, c_len: u32, p_ptr: *const u8, p_len: u32) -> u32;
    fn __invoke_and_return_object(c_ptr: *const u8, c_len: u32, p_ptr: *const u8, p_len: u32) -> u64;
    fn __invoke_and_return_bigint(c_ptr: *const u8, c_len: u32, p_ptr: *const u8, p_len: u32) -> i64;
    fn __invoke_and_return_string(c_ptr: *const u8, c_len: u32, p_ptr: *const u8, p_len: u32) -> u32;
    fn __invoke_and_return_array_buffer(c_ptr: *const u8, c_len: u32, p_ptr: *const u8, p_len: u32) -> u32;
    fn __invoke_and_return_bool(c_ptr: *const u8, c_len: u32, p_ptr: *const u8, p_len: u32) -> u32;
}

#[cfg(test)]
unsafe fn __invoke_and_return_number(_c_ptr: *const u8, _c_len: u32, _p_ptr: *const u8, _p_len: u32) -> u32 { 0 }
#[cfg(test)]
unsafe fn __invoke_and_return_object(_c_ptr: *const u8, _c_len: u32, _p_ptr: *const u8, _p_len: u32) -> u64 { 0 }
#[cfg(test)]
unsafe fn __invoke_and_return_bigint(_c_ptr: *const u8, _c_len: u32, _p_ptr: *const u8, _p_len: u32) -> i64 { 0 }
#[cfg(test)]
unsafe fn __invoke_and_return_string(_c_ptr: *const u8, _c_len: u32, _p_ptr: *const u8, _p_len: u32) -> u32 { 0 }
#[cfg(test)]
unsafe fn __invoke_and_return_array_buffer(_c_ptr: *const u8, _c_len: u32, _p_ptr: *const u8, _p_len: u32) -> u32 { 0 }
#[cfg(test)]
unsafe fn __invoke_and_return_bool(_c_ptr: *const u8, _c_len: u32, _p_ptr: *const u8, _p_len: u32) -> u32 { 0 }

pub fn invoke_and_return_number(code: &str, params: &[InvokeParam]) -> u32 {
    let param_bytes = ManuallyDrop::new(serialize_params(params));
    unsafe { __invoke_and_return_number(code.as_ptr(), code.len() as u32, param_bytes.as_ptr(), param_bytes.len() as u32) }
}

pub fn invoke_and_return_object(code: &str, params: &[InvokeParam]) -> ExternRef {
    let param_bytes = ManuallyDrop::new(serialize_params(params));
    let handle = unsafe { __invoke_and_return_object(code.as_ptr(), code.len() as u32, param_bytes.as_ptr(), param_bytes.len() as u32) };
    ExternRef { value: handle as u32 }
}

pub fn invoke_and_return_bigint(code: &str, params: &[InvokeParam]) -> i64 {
    let param_bytes = ManuallyDrop::new(serialize_params(params));
    unsafe { __invoke_and_return_bigint(code.as_ptr(), code.len() as u32, param_bytes.as_ptr(), param_bytes.len() as u32) }
}

pub fn invoke_and_return_string(code: &str, params: &[InvokeParam]) -> String {
    let param_bytes = ManuallyDrop::new(serialize_params(params));
    let allocation_id = unsafe { __invoke_and_return_string(code.as_ptr(), code.len() as u32, param_bytes.as_ptr(), param_bytes.len() as u32) };
    crate::allocations::get_string_from_allocation(allocation_id)
}

pub fn invoke_and_return_array_buffer(code: &str, params: &[InvokeParam]) -> Vec<u8> {
    let param_bytes = ManuallyDrop::new(serialize_params(params));
    let allocation_id = unsafe { __invoke_and_return_array_buffer(code.as_ptr(), code.len() as u32, param_bytes.as_ptr(), param_bytes.len() as u32) };
    crate::allocations::get_vec_from_allocation(allocation_id)
}

pub fn invoke_and_return_bool(code: &str, params: &[InvokeParam]) -> bool {
    let param_bytes = ManuallyDrop::new(serialize_params(params));
    let result = unsafe { __invoke_and_return_bool(code.as_ptr(), code.len() as u32, param_bytes.as_ptr(), param_bytes.len() as u32) };
    result != 0
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_params() {

        // undefined
        assert_eq!(InvokeParam::Undefined.serialize(), vec![0]);

        // null
        assert_eq!(InvokeParam::Null.serialize(), vec![1]);

        // bigint
        assert_eq!(InvokeParam::BigInt(42).serialize(), [vec![3], 42u64.to_le_bytes().to_vec()].concat());

        // string
        let text = "hello";
        let text_ptr = text.as_ptr() as u32;
        let text_len = text.len() as u64;
        let expected = [vec![4], text_ptr.to_le_bytes().to_vec(), text_len.to_le_bytes().to_vec()].concat();
        assert_eq!(InvokeParam::String(text).serialize(), expected);

        // extern ref
        assert_eq!(InvokeParam::ExternRef(&ExternRef { value: 42 }).serialize(), [vec![5], 42u32.to_le_bytes().to_vec()].concat());

        // float32 array
        let array = [1.0, 2.0];
        let array_ptr = array.as_ptr() as u32;
        let array_len = array.len() as u64;
        let expected = [vec![6], array_ptr.to_le_bytes().to_vec(), array_len.to_le_bytes().to_vec()].concat();
        assert_eq!(InvokeParam::Float32Array(&array).serialize(), expected);

        // float64 array
        let array = [1.0, 2.0];
        let array_ptr = array.as_ptr() as u32;
        let array_len = array.len() as u64;
        let expected = [vec![9], array_ptr.to_le_bytes().to_vec(), array_len.to_le_bytes().to_vec()].concat();
        assert_eq!(InvokeParam::Float64Array(&array).serialize(), expected);

        // bool
        assert_eq!(InvokeParam::Bool(true).serialize(), vec![7]);

        // u32 array
        let array = [1, 2];
        let array_ptr = array.as_ptr() as u32;
        let array_len = array.len() as u64;
        let expected = [vec![10], array_ptr.to_le_bytes().to_vec(), array_len.to_le_bytes().to_vec()].concat();
        assert_eq!(InvokeParam::Uint32Array(&array).serialize(), expected);

    }

    #[test]
    fn test_invoke() {

        // invoke
        let result = invoke_and_return_number("", &[]);
        assert_eq!(result, 0);

        // invoke and return object
        let result = invoke_and_return_object("", &[]);
        assert_eq!(result, ExternRef { value: 0 });

        // invoke and return bigint
        let result = invoke_and_return_bigint("", &[]);
        assert_eq!(result, 0);

        // invoke and return string
        let text = "hello";
        crate::allocations::ALLOCATIONS.with_borrow_mut(|s| {
            *s = vec![Some(text.as_bytes().to_vec())];
        });
        let result = invoke_and_return_string("", &[]);
        assert_eq!(result, "hello".to_owned());

        // invoke and return array buffer
        let vec = vec![1, 2];
        crate::allocations::ALLOCATIONS.with_borrow_mut(|s| {
            *s = vec![Some(vec.clone())];
        });
        let result = invoke_and_return_array_buffer("", &[]);
        assert_eq!(result, vec);

        // invoke and return bool
        let result = invoke_and_return_bool("", &[]);
        assert_eq!(result, false);
    }

}
