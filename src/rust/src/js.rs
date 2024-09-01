
use std::mem::ManuallyDrop;
use std::hash::{Hash, Hasher};

use crate::params::*;

extern "C" {
    fn js_register_function(ptr: f64, len: f64) -> f64;
    fn js_invoke_function(fn_handle: f64, ptr: *const u8, len: usize) -> f64;
    fn js_invoke_function_and_return_object(fn_handle: f64, ptr: *const u8, len: usize) -> i64;
    fn js_invoke_function_and_return_bigint(fn_handle: f64, ptr: *const u8, len: usize) -> i64;
    fn js_invoke_function_and_return_string(fn_handle: f64, ptr: *const u8, len: usize) -> usize;
    fn js_invoke_function_and_return_array_buffer(fn_handle: f64, ptr: *const u8, len: usize) -> usize;
    fn js_invoke_function_and_return_bool(fn_handle: f64, ptr: *const u8, len: usize) -> f64;
}

#[derive(Debug, Clone)]
pub struct ExternRef { pub value: i64, }

impl PartialEq for ExternRef {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for ExternRef {}

impl Hash for ExternRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

#[derive(Copy, Clone)]
pub struct JSFunction {
    pub fn_handle: f64,
}

impl JSFunction {

    pub fn register(code: &str) -> JSFunction {
        unsafe { JSFunction { fn_handle: js_register_function(code.as_ptr() as usize as f64, code.len() as f64), } }
    }

    pub fn invoke(&self, params: &[InvokeParam]) -> f64 {
        let param_bytes = param_to_bytes(params);
        let mut me = ManuallyDrop::new(param_bytes);
        unsafe { js_invoke_function(self.fn_handle, me.as_mut_ptr(), me.len()) }
    }

    pub fn invoke_and_return_object(&self, params: &[InvokeParam]) -> ExternRef {
        let param_bytes = param_to_bytes(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let handle = unsafe { js_invoke_function_and_return_object(self.fn_handle, me.as_mut_ptr(), me.len()) };
        ExternRef { value: handle }
    }

    pub fn invoke_and_return_bigint(&self, params: &[InvokeParam]) -> i64 {
        let param_bytes = param_to_bytes(params);
        let mut me = ManuallyDrop::new(param_bytes);
        unsafe { js_invoke_function_and_return_bigint(self.fn_handle, me.as_mut_ptr(), me.len()) }
    }

    pub fn invoke_and_return_string(&self, params: &[InvokeParam]) -> String {
        let param_bytes = param_to_bytes(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let allocation_id =
            unsafe { js_invoke_function_and_return_string(self.fn_handle, me.as_mut_ptr(), me.len()) };
        crate::allocations::extract_string_from_memory(allocation_id)
    }

    pub fn invoke_and_return_array_buffer(&self, params: &[InvokeParam]) -> Vec<u8> {
        let param_bytes = param_to_bytes(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let allocation_id =
            unsafe { js_invoke_function_and_return_array_buffer(self.fn_handle, me.as_mut_ptr(), me.len()) };
        crate::allocations::extract_vec_from_memory(allocation_id)
    }

    pub fn invoke_and_return_bool(&self, params: &[InvokeParam]) -> bool {
        let param_bytes = param_to_bytes(params);
        let mut me = ManuallyDrop::new(param_bytes);
        let ret = unsafe { js_invoke_function_and_return_bool(self.fn_handle, me.as_mut_ptr(), me.len()) };
        ret != 0.0
    }
}
