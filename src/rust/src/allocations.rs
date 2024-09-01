
use std::sync::Mutex;

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
