
use std::cell::RefCell;

thread_local! {
    pub static ALLOCATIONS: RefCell<Vec<Option<Vec<u8>>>> = Default::default();
}

pub fn extract_string_from_memory(allocation_id: usize) -> String {
    ALLOCATIONS.with_borrow(|s| {
        let s = s.get(allocation_id).cloned().unwrap();
        String::from_utf8(s.unwrap())
    }).unwrap()
}

pub fn extract_vec_from_memory(allocation_id: usize) -> Vec<u8> {
    ALLOCATIONS.with_borrow(|s| {
        s.get(allocation_id).cloned().unwrap()
    }).unwrap()
}

#[no_mangle]
pub fn create_allocation(size: usize) -> usize {
    let mut buf = Vec::with_capacity(size as usize);
    buf.resize(size, 0);

    ALLOCATIONS.with_borrow_mut(|s| {
        let len = s.len();
        s.push(Some(buf));
        len
    })
}

#[no_mangle]
pub fn allocation_ptr(allocation_id: usize) -> *const u8 {
    let vec = ALLOCATIONS.with_borrow(|s| {
        s.get(allocation_id).cloned().unwrap()
    }).unwrap();
    vec.as_ptr()
}

#[no_mangle]
pub fn allocation_len(allocation_id: usize) -> f64 {
    let vec = ALLOCATIONS.with_borrow(|s| {
        s.get(allocation_id).cloned().unwrap()
    }).unwrap();
    vec.len() as f64
}

pub fn clear_allocation(allocation_id: usize) {
    ALLOCATIONS.with_borrow_mut(|s| {
        s[allocation_id] = None;
    })
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation() {
        
        let id = create_allocation(1);
        let allocation = ALLOCATIONS.with_borrow(|s| s[id].clone());
        assert_eq!(allocation.is_some(), true);

        let ptr = allocation_ptr(id);
        assert_eq!(ptr.is_null(), false);
        
        let len = allocation_len(id);
        assert_eq!(len, 1f64);
        
        let id2 = create_allocation(1);
        let allocation = ALLOCATIONS.with_borrow(|s| s[id2].clone());
        assert_eq!(allocation.is_some(), true);

        clear_allocation(id);

        let allocation = ALLOCATIONS.with_borrow(|s| s[id].clone());
        assert_eq!(allocation.is_some(), false);

    }

    #[test]
    fn test_memory() {
        
        // test string
        let id = create_allocation(1);
        
        let text = "hello";
        ALLOCATIONS.with_borrow_mut(|s| {
            s[id] = Some(text.as_bytes().to_vec());
        });
        
        let memory_text = extract_string_from_memory(id);
        assert_eq!(memory_text, text);
        
        // test vec
        let id = create_allocation(1);
        
        let vec = vec![1, 2];
        ALLOCATIONS.with_borrow_mut(|s| {
            s[id] = Some(vec.clone());
        });
        
        let memory_vec = extract_vec_from_memory(id);
        assert_eq!(memory_vec, vec);
    }
}