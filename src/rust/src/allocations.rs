
use std::cell::RefCell;

thread_local! {
    pub(crate) static ALLOCATIONS: RefCell<Vec<Option<Vec<u8>>>> = RefCell::new(Vec::new());
}

pub fn get_string_from_allocation(allocation_id: u32) -> String {
    ALLOCATIONS.with_borrow(|s| {
        let allocation = s.get(allocation_id as usize).unwrap();
        let vec = allocation.as_ref().unwrap();
        String::from_utf8(vec.clone()).unwrap()
    })
}

pub fn get_vec_from_allocation(allocation_id: u32) -> Vec<u8> {
    ALLOCATIONS.with_borrow(|s| {
        let allocation = s.get(allocation_id as usize).unwrap();
        let vec = allocation.as_ref().unwrap();
        vec.clone()
    })
}

#[no_mangle]
pub fn create_allocation(size: u32) -> u32 {
    let mut buf = Vec::with_capacity(size as usize);
    buf.resize(size as usize, 0);

    ALLOCATIONS.with_borrow_mut(|s| {
        let i = s.len();
        s.push(Some(buf));
        i
    }) as u32
}

#[no_mangle]
pub fn allocation_ptr(allocation_id: u32) -> *const u8 {
    ALLOCATIONS.with_borrow(|s| {
        let allocation = s.get(allocation_id as usize).unwrap();
        let vec = allocation.as_ref().unwrap();
        vec.as_ptr()
    })
}

#[no_mangle]
pub fn allocation_len(allocation_id: u32) -> f64 {
    ALLOCATIONS.with_borrow(|s| {
        let allocation = s.get(allocation_id as usize).unwrap();
        let vec = allocation.as_ref().unwrap();
        vec.len() as f64
    })
}

pub fn clear_allocation(allocation_id: u32) {
    ALLOCATIONS.with_borrow_mut(|s| {
        s[allocation_id as usize] = None;
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
        
        let memory_text = get_string_from_allocation(id);
        assert_eq!(memory_text, text);
        
        // test vec
        let id = create_allocation(1);
        
        let vec = vec![1, 2];
        ALLOCATIONS.with_borrow_mut(|s| {
            s[id] = Some(vec.clone());
        });
        
        let memory_vec = get_vec_from_allocation(id);
        assert_eq!(memory_vec, vec);
    }
}