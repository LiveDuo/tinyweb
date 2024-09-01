
use std::sync::Mutex;

pub(crate) static ALLOCATIONS: Mutex<Vec<Option<Vec<u8>>>> = Mutex::new(Vec::new());

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
pub fn allocation_ptr(allocation_id: usize) -> *const u8 {
    let allocations = ALLOCATIONS.lock().unwrap();
    let allocation = allocations.get(allocation_id).unwrap();
    let vec = allocation.as_ref().unwrap();
    vec.as_ptr()
}

#[no_mangle]
pub fn allocation_len(allocation_id: usize) -> f64 {
    let allocations = ALLOCATIONS.lock().unwrap();
    let allocation = allocations.get(allocation_id).unwrap();
    let vec = allocation.as_ref().unwrap();
    vec.len() as f64
}

pub fn clear_allocation(allocation_id: usize) {
    let mut allocations = ALLOCATIONS.lock().unwrap();
    allocations[allocation_id] = None;
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation() {
        
        let id = create_allocation(1);
        let allocation = ALLOCATIONS.lock().map(|s| s[id].clone()).unwrap();
        assert_eq!(allocation.is_some(), true);

        let ptr = allocation_ptr(id);
        assert_eq!(ptr.is_null(), false);
        
        let len = allocation_len(id);
        assert_eq!(len, 1f64);
        
        let id2 = create_allocation(1);
        let allocation = ALLOCATIONS.lock().map(|s| s[id2].clone()).unwrap();
        assert_eq!(allocation.is_some(), true);

        clear_allocation(id);

        let allocation = ALLOCATIONS.lock().map(|s| s[id].clone()).unwrap();
        assert_eq!(allocation.is_some(), false);

    }

    #[test]
    fn test_memory() {
        
        // test string
        let id = create_allocation(1);
        
        let text = "hello";
        ALLOCATIONS.lock().map(|mut s| {
            s[id] = Some(text.as_bytes().to_vec());
        }).unwrap();
        
        let memory_text = extract_string_from_memory(id);
        assert_eq!(memory_text, text);
        
        // test vec
        let id = create_allocation(1);
        
        let vec = vec![1, 2];
        ALLOCATIONS.lock().map(|mut s| {
            s[id] = Some(vec.clone());
        }).unwrap();
        
        let memory_vec = extract_vec_from_memory(id);
        assert_eq!(memory_vec, vec);
    }
}