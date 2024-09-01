
use std::mem::ManuallyDrop;

use crate::utils::params::*;

#[derive(Debug, Clone)]
pub struct ExternRef { pub value: i64, }

#[derive(Copy, Clone)]
pub struct JSFunction {
    pub fn_handle: f64,
}

extern "C" {
    fn js_register_function(ptr: f64, len: f64) -> f64;
    fn js_invoke_function(fn_handle: f64, ptr: *const u8, len: usize) -> f64;
    fn js_invoke_function_and_return_object(fn_handle: f64, ptr: *const u8, len: usize) -> i64;
    fn js_invoke_function_and_return_bigint(fn_handle: f64, ptr: *const u8, len: usize) -> i64;
    fn js_invoke_function_and_return_string(fn_handle: f64, ptr: *const u8, len: usize) -> usize;
    fn js_invoke_function_and_return_array_buffer(fn_handle: f64, ptr: *const u8, len: usize) -> usize;
    fn js_invoke_function_and_return_bool(fn_handle: f64, ptr: *const u8, len: usize) -> f64;
}

pub fn register_function(code: &str) -> JSFunction {
    let start = code.as_ptr();
    let len = code.len();
    unsafe { JSFunction { fn_handle: js_register_function(start as usize as f64, len as f64), } }
}

impl JSFunction {
    pub fn invoke(&self, params: &[InvokeParam]) -> f64 {
        let param_bytes = param_to_bytes(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let (ptr, length, _capacity) = (me.as_mut_ptr(), me.len(), me.capacity());
        unsafe { js_invoke_function(self.fn_handle, ptr, length) }
    }

    pub fn invoke_and_return_object(&self, params: &[InvokeParam]) -> ExternRef {
        let param_bytes = param_to_bytes(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let (ptr, length, _capacity) = (me.as_mut_ptr(), me.len(), me.capacity());
        let handle = unsafe { js_invoke_function_and_return_object(self.fn_handle, ptr, length) };
        ExternRef { value: handle }
    }

    pub fn invoke_and_return_bigint(&self, params: &[InvokeParam]) -> i64 {
        let param_bytes = param_to_bytes(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let (ptr, length, _capacity) = (me.as_mut_ptr(), me.len(), me.capacity());
        unsafe { js_invoke_function_and_return_bigint(self.fn_handle, ptr, length) }
    }

    pub fn invoke_and_return_string(&self, params: &[InvokeParam]) -> String {
        let param_bytes = param_to_bytes(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let (ptr, length, _capacity) = (me.as_mut_ptr(), me.len(), me.capacity());
        let allocation_id =
            unsafe { js_invoke_function_and_return_string(self.fn_handle, ptr, length) };
        crate::utils::allocations::extract_string_from_memory(allocation_id)
    }

    pub fn invoke_and_return_array_buffer(&self, params: &[InvokeParam]) -> Vec<u8> {
        let param_bytes = param_to_bytes(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let (ptr, length, _capacity) = (me.as_mut_ptr(), me.len(), me.capacity());
        let allocation_id =
            unsafe { js_invoke_function_and_return_array_buffer(self.fn_handle, ptr, length) };
        crate::utils::allocations::extract_vec_from_memory(allocation_id)
    }

    pub fn invoke_and_return_bool(&self, params: &[InvokeParam]) -> bool {
        let param_bytes = param_to_bytes(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let (ptr, length, _capacity) = (me.as_mut_ptr(), me.len(), me.capacity());
        let ret = unsafe { js_invoke_function_and_return_bool(self.fn_handle, ptr, length) };
        ret != 0.0
    }
}
