
use std::mem::ManuallyDrop;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExternRef { pub value: u64, }


// preceded by a 32 bit integer indicating its type
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

impl From<f64> for InvokeParam<'_> {
    fn from(f: f64) -> Self { InvokeParam::Float64(f) }
}

impl From<i32> for InvokeParam<'_> {
    fn from(i: i32) -> Self { InvokeParam::Float64(i as f64) }
}

impl From<u32> for InvokeParam<'_> {
    fn from(i: u32) -> Self { InvokeParam::Float64(i as f64) }
}

impl From<i64> for InvokeParam<'_> {
    fn from(i: i64) -> Self { InvokeParam::BigInt(i) }
}

impl<'a> From<&'a str> for InvokeParam<'a> {
    fn from(s: &'a str) -> Self { InvokeParam::String(s) }
}

impl<'a> From<&'a ExternRef> for InvokeParam<'a> {
    fn from(i: &'a ExternRef) -> Self { InvokeParam::ExternRef(i) }
}

impl<'a> From<&'a [f32]> for InvokeParam<'a> {
    fn from(a: &'a [f32]) -> Self { InvokeParam::Float32Array(a) }
}

impl<'a> From<&'a [f64]> for InvokeParam<'a> {
    fn from(a: &'a [f64]) -> Self { InvokeParam::Float64Array(a) }
}

impl From<bool> for InvokeParam<'_> {
    fn from(b: bool) -> Self { InvokeParam::Bool(b) }
}

impl<'a> From<&'a [u32]> for InvokeParam<'a> {
    fn from(a: &'a [u32]) -> Self { InvokeParam::Uint32Array(a) }
}

pub fn serialize(params: &[InvokeParam]) -> Vec<u8> {
    let mut param_bytes = Vec::new();
    for param in params {
        match param {
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
                let start = s.as_ptr() as u32;
                let len = s.len();
                param_bytes.extend_from_slice(&start.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
            InvokeParam::ExternRef(i) => {
                param_bytes.push(5);
                param_bytes.extend_from_slice(&i.value.to_le_bytes());
            }
            InvokeParam::Float32Array(a) => {
                param_bytes.push(6);
                let start = a.as_ptr() as u32;
                let len = a.len();
                param_bytes.extend_from_slice(&start.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
            InvokeParam::Bool(b) => {
                param_bytes.push(if *b { 7 } else { 8 });
            }
            InvokeParam::Float64Array(a) => {
                param_bytes.push(9);
                let start = a.as_ptr() as u32;
                let len = a.len();
                param_bytes.extend_from_slice(&start.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
            InvokeParam::Uint32Array(a) => {
                param_bytes.push(10);
                let start = a.as_ptr() as u32;
                let len = a.len();
                param_bytes.extend_from_slice(&start.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
        }
    }
    param_bytes
}

#[cfg(not(test))]
extern "C" {
    fn js_register_function(ptr: *const u8, len: u32) -> u32;
    fn js_invoke_function(fn_handle: u32, ptr: *const u8, len: u32) -> u32;
    fn js_invoke_function_and_return_object(fn_handle: u32, ptr: *const u8, len: u32) -> u64;
    fn js_invoke_function_and_return_bigint(fn_handle: u32, ptr: *const u8, len: u32) -> i64;
    fn js_invoke_function_and_return_string(fn_handle: u32, ptr: *const u8, len: u32) -> u32;
    fn js_invoke_function_and_return_array_buffer(fn_handle: u32, ptr: *const u8, len: u32) -> u32;
    fn js_invoke_function_and_return_bool(fn_handle: u32, ptr: *const u8, len: u32) -> u32;
}

#[cfg(test)]
fn js_register_function(_ptr: *const u8, _len: u32) -> u32 { 0 }
#[cfg(test)]
fn js_invoke_function(_fn_handle: u32, _ptr: *const u8, _len: u32) -> u32 { 0 }
#[cfg(test)]
fn js_invoke_function_and_return_object(_fn_handle: u32, _ptr: *const u8, _len: u32) -> u64 { 0 }
#[cfg(test)]
fn js_invoke_function_and_return_bigint(_fn_handle: u32, _ptr: *const u8, _len: u32) -> i64 { 0 }
#[cfg(test)]
fn js_invoke_function_and_return_string(_fn_handle: u32, _ptr: *const u8, _len: u32) -> u32 { 0 }
#[cfg(test)]
fn js_invoke_function_and_return_array_buffer(_fn_handle: u32, _ptr: *const u8, _len: u32) -> u32 { 0 }
#[cfg(test)]
fn js_invoke_function_and_return_bool(_fn_handle: u32, _ptr: *const u8, _len: u32) -> u32 { 0 }

#[derive(Copy, Clone)]
pub struct JsFunction {
    pub fn_handle: u32,
}

#[allow(unused_unsafe)]
impl JsFunction {

    pub fn register(code: &str) -> JsFunction {
        JsFunction { fn_handle: unsafe { js_register_function(code.as_ptr(), code.len() as u32) } }
    }

    pub fn invoke(&self, params: &[InvokeParam]) -> f32 {
        let param_bytes = serialize(params);
        let mut me = ManuallyDrop::new(param_bytes);
        unsafe { js_invoke_function(self.fn_handle, me.as_mut_ptr(), me.len() as u32) as f32 }
    }

    pub fn invoke_and_return_object(&self, params: &[InvokeParam]) -> ExternRef {
        let param_bytes = serialize(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let handle = unsafe { js_invoke_function_and_return_object(self.fn_handle, me.as_mut_ptr(), me.len() as u32) };
        ExternRef { value: handle as u64 }
    }

    pub fn invoke_and_return_bigint(&self, params: &[InvokeParam]) -> i64 {
        let param_bytes = serialize(params);
        let mut me = ManuallyDrop::new(param_bytes);
        unsafe { js_invoke_function_and_return_bigint(self.fn_handle, me.as_mut_ptr(), me.len() as u32) }
    }

    pub fn invoke_and_return_string(&self, params: &[InvokeParam]) -> String {
        let param_bytes = serialize(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let allocation_id =
            unsafe { js_invoke_function_and_return_string(self.fn_handle, me.as_mut_ptr(), me.len() as u32) };
        crate::allocations::get_string_from_allocation(allocation_id)
    }

    pub fn invoke_and_return_array_buffer(&self, params: &[InvokeParam]) -> Vec<u8> {
        let param_bytes = serialize(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let allocation_id =
            unsafe { js_invoke_function_and_return_array_buffer(self.fn_handle, me.as_mut_ptr(), me.len() as u32) };
        crate::allocations::get_vec_from_allocation(allocation_id)
    }

    pub fn invoke_and_return_bool(&self, params: &[InvokeParam]) -> bool {
        let param_bytes = serialize(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let ret = unsafe { js_invoke_function_and_return_bool(self.fn_handle, me.as_mut_ptr(), me.len() as u32) };
        ret != 0
    }
}



#[cfg(test)]
mod tests {
    
    use super::*;

    #[test]
    fn test_params() {
        
        // undefined
        assert_eq!(serialize(&[InvokeParam::Undefined]), vec![0]);

        // null
        assert_eq!(serialize(&[InvokeParam::Null]), vec![1]);

        // bigint
        assert_eq!(serialize(&[InvokeParam::BigInt(42)]), [vec![3], 42u64.to_le_bytes().to_vec()].concat());

        // string
        let text = "hello";
        let text_ptr = text.as_ptr() as u32;
        let text_len = text.len() as u64;
        let expected = [vec![4], text_ptr.to_le_bytes().to_vec(), text_len.to_le_bytes().to_vec()].concat();
        assert_eq!(serialize(&[InvokeParam::String(text)]), expected);

        // extern ref
        assert_eq!(serialize(&[InvokeParam::ExternRef(&ExternRef { value: 42 })]), [vec![5], 42u64.to_le_bytes().to_vec()].concat());
        
        // float32 array
        let array = [1.0, 2.0];
        let array_ptr = array.as_ptr() as u32;
        let array_len = array.len() as u64;
        let expected = [vec![6], array_ptr.to_le_bytes().to_vec(), array_len.to_le_bytes().to_vec()].concat();
        assert_eq!(serialize(&[InvokeParam::Float32Array(&array)]), expected);
        
        // float64 array
        let array = [1.0, 2.0];
        let array_ptr = array.as_ptr() as u32;
        let array_len = array.len() as u64;
        let expected = [vec![9], array_ptr.to_le_bytes().to_vec(), array_len.to_le_bytes().to_vec()].concat();
        assert_eq!(serialize(&[InvokeParam::Float64Array(&array)]), expected);
        
        // bool
        assert_eq!(serialize(&[InvokeParam::Bool(true)]), vec![7]);
        
        // u32 array
        let array = [1, 2];
        let array_ptr = array.as_ptr() as u32;
        let array_len = array.len() as u64;
        let expected = [vec![10], array_ptr.to_le_bytes().to_vec(), array_len.to_le_bytes().to_vec()].concat();
        assert_eq!(serialize(&[InvokeParam::Uint32Array(&array)]), expected);

    }

    #[test]
    fn test_register_invoke() {
        
        // register
        let func = JsFunction::register("");
        assert_eq!(func.fn_handle, 0);

        // invoke
        let result = func.invoke(&[]);
        assert_eq!(result, 0.0);
        
        // invoke and return object
        let result = func.invoke_and_return_object(&[]);
        assert_eq!(result, ExternRef { value: 0 });
        
        // invoke and return bigint
        let result = func.invoke_and_return_bigint(&[]);
        assert_eq!(result, 0);
        
        // invoke and return string
        let text = "hello";
        crate::allocations::ALLOCATIONS.with_borrow_mut(|s| {
            *s = vec![Some(text.as_bytes().to_vec())];
        });
        let result = func.invoke_and_return_string(&[]);
        assert_eq!(result, "hello".to_owned());
        
        // invoke and return array buffer
        let vec = vec![1, 2];
        crate::allocations::ALLOCATIONS.with_borrow_mut(|s| {
            *s = vec![Some(vec.clone())];
        });
        let result = func.invoke_and_return_array_buffer(&[]);
        assert_eq!(result, vec);
        
        // invoke and return bool
        let result = func.invoke_and_return_bool(&[]);
        assert_eq!(result, false);
    }

}