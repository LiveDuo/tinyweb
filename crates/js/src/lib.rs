
extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use std::mem::ManuallyDrop;
use std::sync::Mutex;

pub struct RawParts<T> {
    /// A non-null pointer to a buffer of `T`.
    ///
    /// This pointer is the same as the value returned by [`Vec::as_mut_ptr`] in
    /// the source vector.
    pub ptr: *mut T,
    /// The number of elements in the source vector, also referred to as its
    /// "length".
    ///
    /// This value is the same as the value returned by [`Vec::len`] in the
    /// source vector.
    pub length: usize,
    /// The number of elements the source vector can hold without reallocating.
    ///
    /// This value is the same as the value returned by [`Vec::capacity`] in the
    /// source vector.
    pub capacity: usize,
}

impl<T> RawParts<T> {
    /// Construct the raw components of a `Vec<T>` by decomposing it.
    ///
    /// Returns a struct containing the raw pointer to the underlying data, the
    /// length of the vector (in elements), and the allocated capacity of the
    /// data (in elements).
    ///
    /// After calling this function, the caller is responsible for the memory
    /// previously managed by the `Vec`. The only way to do this is to convert
    /// the raw pointer, length, and capacity back into a `Vec` with the
    /// [`Vec::from_raw_parts`] function or the [`into_vec`] function, allowing
    /// the destructor to perform the cleanup.
    ///
    /// [`into_vec`]: Self::into_vec
    ///
    /// # Examples
    ///
    /// ```
    /// use raw_parts::RawParts;
    ///
    /// let v: Vec<i32> = vec![-1, 0, 1];
    ///
    /// let RawParts { ptr, length, capacity } = RawParts::from_vec(v);
    ///
    /// let rebuilt = unsafe {
    ///     // We can now make changes to the components, such as
    ///     // transmuting the raw pointer to a compatible type.
    ///     let ptr = ptr as *mut u32;
    ///     let raw_parts = RawParts { ptr, length, capacity };
    ///
    ///     RawParts::into_vec(raw_parts)
    /// };
    /// assert_eq!(rebuilt, [4294967295, 0, 1]);
    /// ```
    #[must_use]
    pub fn from_vec(vec: Vec<T>) -> RawParts<T> {
        // TODO: convert to `Vec::into_raw_parts` once it is stabilized.
        // See: https://doc.rust-lang.org/1.56.0/src/alloc/vec/mod.rs.html#717-720
        //
        // https://github.com/rust-lang/rust/issues/65816
        let mut me = ManuallyDrop::new(vec);
        let (ptr, length, capacity) = (me.as_mut_ptr(), me.len(), me.capacity());

        Self {
            ptr,
            length,
            capacity,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExternRef {
    pub value: i64,
}

extern "C" {
    fn externref_drop(extern_ref: i64);
}

impl From<i64> for ExternRef {
    fn from(value: i64) -> Self {
        ExternRef { value }
    }
}

impl Into<i64> for &ExternRef {
    fn into(self) -> i64 {
        self.value
    }
}

impl Drop for ExternRef {
    fn drop(&mut self) {
        unsafe {
            externref_drop(self.value);
        }
    }
}

pub const JS_UNDEFINED: ExternRef = ExternRef { value: 0 };
pub const JS_NULL: ExternRef = ExternRef { value: 1 };
pub const DOM_SELF: ExternRef = ExternRef { value: 2 };
pub const DOM_WINDOW: ExternRef = ExternRef { value: 2 };
pub const DOM_DOCUMENT: ExternRef = ExternRef { value: 3 };
pub const DOM_BODY: ExternRef = ExternRef { value: 4 };

extern "C" {
    fn js_register_function(start: f64, len: f64) -> f64;
    fn js_invoke_function(
        fn_handle: f64,
        parameters_start: *const u8,
        parameters_length: usize,
    ) -> f64;
    fn js_invoke_function_and_return_object(
        fn_handle: f64,
        parameters_start: *const u8,
        parameters_length: usize,
    ) -> i64;
    fn js_invoke_function_and_return_bigint(
        fn_handle: f64,
        parameters_start: *const u8,
        parameters_length: usize,
    ) -> i64;
    fn js_invoke_function_and_return_string(
        fn_handle: f64,
        parameters_start: *const u8,
        parameters_length: usize,
    ) -> usize;
    fn js_invoke_function_and_return_array_buffer(
        fn_handle: f64,
        parameters_start: *const u8,
        parameters_length: usize,
    ) -> usize;
    fn js_invoke_function_and_return_bool(
        fn_handle: f64,
        parameters_start: *const u8,
        parameters_length: usize,
    ) -> f64;
}

#[derive(Copy, Clone)]
pub struct JSFunction {
    pub fn_handle: f64,
}

//convert invoke parameters into bytes
//assuming each parameter is preceded by a 32 bit integer indicating its type
//0 = undefined
//1 = null
//2 = float-64
//3 = bigint
//4 = string (followed by 32-bit start and size of string in memory)
//5 = extern ref
//6 = array of float-64 (followed by 32-bit start and size of string in memory)
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
    fn from(f: f64) -> Self {
        InvokeParam::Float64(f)
    }
}

impl From<i32> for InvokeParam<'_> {
    fn from(i: i32) -> Self {
        InvokeParam::Float64(i as f64)
    }
}

impl From<usize> for InvokeParam<'_> {
    fn from(i: usize) -> Self {
        InvokeParam::Float64(i as f64)
    }
}

impl From<i64> for InvokeParam<'_> {
    fn from(i: i64) -> Self {
        InvokeParam::BigInt(i)
    }
}

impl<'a> From<&'a str> for InvokeParam<'a> {
    fn from(s: &'a str) -> Self {
        InvokeParam::String(s)
    }
}

impl<'a> From<&'a ExternRef> for InvokeParam<'a> {
    fn from(i: &'a ExternRef) -> Self {
        InvokeParam::ExternRef(i)
    }
}

impl<'a> From<&'a [f32]> for InvokeParam<'a> {
    fn from(a: &'a [f32]) -> Self {
        InvokeParam::Float32Array(a)
    }
}

impl<'a> From<&'a [f64]> for InvokeParam<'a> {
    fn from(a: &'a [f64]) -> Self {
        InvokeParam::Float64Array(a)
    }
}

impl From<bool> for InvokeParam<'_> {
    fn from(b: bool) -> Self {
        InvokeParam::Bool(b)
    }
}

impl<'a> From<&'a [u32]> for InvokeParam<'a> {
    fn from(a: &'a [u32]) -> Self {
        InvokeParam::Uint32Array(a)
    }
}

fn param_to_bytes(params: &[InvokeParam]) -> Vec<u8> {
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
                let start = s.as_ptr() as usize;
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
                let start = a.as_ptr() as usize;
                let len = a.len();
                param_bytes.extend_from_slice(&start.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
            InvokeParam::Bool(b) => {
                if *b {
                    param_bytes.push(7);
                } else {
                    param_bytes.push(8);
                }
            }
            InvokeParam::Float64Array(a) => {
                param_bytes.push(9);
                let start = a.as_ptr() as usize;
                let len = a.len();
                param_bytes.extend_from_slice(&start.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
            InvokeParam::Uint32Array(a) => {
                param_bytes.push(10);
                let start = a.as_ptr() as usize;
                let len = a.len();
                param_bytes.extend_from_slice(&start.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
        }
    }
    param_bytes
}

impl JSFunction {
    pub fn invoke(&self, params: &[InvokeParam]) -> f64
where {
        let param_bytes = param_to_bytes(params);
        let RawParts {
            ptr,
            length,
            capacity: _,
        } = RawParts::from_vec(param_bytes);
        unsafe { js_invoke_function(self.fn_handle, ptr, length) }
    }

    pub fn invoke_and_return_object(&self, params: &[InvokeParam]) -> ExternRef
where {
        let param_bytes = param_to_bytes(params);
        let RawParts {
            ptr,
            length,
            capacity: _,
        } = RawParts::from_vec(param_bytes);
        let handle = unsafe { js_invoke_function_and_return_object(self.fn_handle, ptr, length) };
        ExternRef { value: handle }
    }

    pub fn invoke_and_return_bigint(&self, params: &[InvokeParam]) -> i64
where {
        let param_bytes = param_to_bytes(params);
        let RawParts {
            ptr,
            length,
            capacity: _,
        } = RawParts::from_vec(param_bytes);
        unsafe { js_invoke_function_and_return_bigint(self.fn_handle, ptr, length) }
    }

    pub fn invoke_and_return_string(&self, params: &[InvokeParam]) -> String
where {
        let param_bytes = param_to_bytes(params);
        let RawParts {
            ptr,
            length,
            capacity: _,
        } = RawParts::from_vec(param_bytes);
        let allocation_id =
            unsafe { js_invoke_function_and_return_string(self.fn_handle, ptr, length) };
        extract_string_from_memory(allocation_id)
    }

    pub fn invoke_and_return_array_buffer(&self, params: &[InvokeParam]) -> Vec<u8>
where {
        let param_bytes = param_to_bytes(params);
        let RawParts {
            ptr,
            length,
            capacity: _,
        } = RawParts::from_vec(param_bytes);
        let allocation_id =
            unsafe { js_invoke_function_and_return_array_buffer(self.fn_handle, ptr, length) };
        extract_vec_from_memory(allocation_id)
    }

    pub fn invoke_and_return_bool(&self, params: &[InvokeParam]) -> bool {
        let param_bytes = param_to_bytes(params);
        let RawParts {
            ptr,
            length,
            capacity: _,
        } = RawParts::from_vec(param_bytes);
        let ret = unsafe { js_invoke_function_and_return_bool(self.fn_handle, ptr, length) };
        ret != 0.0
    }
}

pub fn register_function(code: &str) -> JSFunction {
    let start = code.as_ptr();
    let len = code.len();
    unsafe {
        JSFunction {
            fn_handle: js_register_function(start as usize as f64, len as f64),
        }
    }
}

static ALLOCATIONS: Mutex<Vec<Option<Vec<u8>>>> = Mutex::new(Vec::new());

pub fn extract_string_from_memory(allocation_id: usize) -> String {
    let allocations = ALLOCATIONS.lock().unwrap();
    let allocation = allocations.get(allocation_id).unwrap();
    let vec = allocation.as_ref().unwrap();
    let s = String::from_utf8(vec.clone()).unwrap();
    s
}

pub fn extract_vec_from_memory(allocation_id: usize) -> Vec<u8> {
    let allocations = ALLOCATIONS.lock().unwrap();
    let allocation = allocations.get(allocation_id).unwrap();
    let vec = allocation.as_ref().unwrap();
    vec.clone()
}

#[no_mangle]
pub fn create_allocation(size: usize) -> usize {
    let mut buf = Vec::with_capacity(size as usize);
    buf.resize(size, 0);
    let mut allocations = ALLOCATIONS.lock().unwrap();
    let i = allocations.len();
    allocations.push(Some(buf));
    i
}

#[no_mangle]
pub fn allocation_ptr(allocation_id: i32) -> *const u8 {
    let allocations = ALLOCATIONS.lock().unwrap();
    let allocation = allocations.get(allocation_id as usize).unwrap();
    let vec = allocation.as_ref().unwrap();
    vec.as_ptr()
}

#[no_mangle]
pub fn allocation_len(allocation_id: i32) -> f64 {
    let allocations = ALLOCATIONS.lock().unwrap();
    let allocation = allocations.get(allocation_id as usize).unwrap();
    let vec = allocation.as_ref().unwrap();
    vec.len() as f64
}

pub fn clear_allocation(allocation_id: usize) {
    let mut allocations = ALLOCATIONS.lock().unwrap();
    allocations[allocation_id] = None;
}

#[macro_export]
macro_rules! js {
    ($e:expr) => {{
        static mut FN: Option<f64> = None;
        unsafe {
            if FN.is_none() {
                FN = Some(js::register_function($e).fn_handle);
            }
            JSFunction {
                fn_handle: FN.unwrap(),
            }
        }
    }};
}
