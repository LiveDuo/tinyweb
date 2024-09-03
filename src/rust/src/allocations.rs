
use std::cell::RefCell;

thread_local! {
    pub(crate) static ALLOCATIONS: RefCell<Vec<Option<Vec<u8>>>> = RefCell::new(Vec::new());
}

pub fn extract_string_from_memory(allocation_id: usize) -> String {
    ALLOCATIONS.with_borrow(|s| {
        let allocation = s.get(allocation_id).unwrap();
        let vec = allocation.as_ref().unwrap();
        String::from_utf8(vec.clone()).unwrap()
    })
}

pub fn extract_vec_from_memory(allocation_id: usize) -> Vec<u8> {
    ALLOCATIONS.with_borrow(|s| {
        let allocation = s.get(allocation_id).unwrap();
        let vec = allocation.as_ref().unwrap();
        vec.clone()
    })
}

#[no_mangle]
pub fn create_allocation(size: usize) -> usize {
    let mut buf = Vec::with_capacity(size as usize);
    buf.resize(size, 0);

    ALLOCATIONS.with_borrow_mut(|s| {
        let i = s.len();
        s.push(Some(buf));
        i
    })
}

#[no_mangle]
pub fn allocation_ptr(allocation_id: usize) -> *const u8 {
    ALLOCATIONS.with_borrow(|s| {
        let allocation = s.get(allocation_id).unwrap();
        let vec = allocation.as_ref().unwrap();
        vec.as_ptr()
    })
}

#[no_mangle]
pub fn allocation_len(allocation_id: usize) -> f64 {
    ALLOCATIONS.with_borrow(|s| {
        let allocation = s.get(allocation_id).unwrap();
        let vec = allocation.as_ref().unwrap();
        vec.len() as f64
    })
}

pub fn clear_allocation(allocation_id: usize) {
    ALLOCATIONS.with_borrow_mut(|s| {
        s[allocation_id] = None;
    });
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