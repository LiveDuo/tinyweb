
use std::cell::RefCell;

thread_local! {
    pub static ALLOCATIONS: RefCell<Vec<Vec<u8>>> = RefCell::new(Vec::new());
}

#[no_mangle]
pub fn create_allocation(size: usize) -> usize {
    ALLOCATIONS.with_borrow_mut(|s| { s.push(vec![0; size]); s.len() - 1 })
}

#[no_mangle]
pub fn get_allocation(allocation_id: usize) -> *const u8 {
    ALLOCATIONS.with_borrow(|s| s.get(allocation_id).unwrap().as_ptr())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation() {

        // test string
        let text = "hello";
        let id = create_allocation(1);
        ALLOCATIONS.with_borrow_mut(|s| { s[id as usize] = text.as_bytes().to_vec(); });
        let allocation_data = ALLOCATIONS.with_borrow(|s| s.get(id as usize).unwrap().to_owned());
        let memory_text = String::from_utf8(allocation_data).unwrap();
        assert_eq!(memory_text, text);

        // test vec
        let vec = vec![1, 2];
        let id = create_allocation(1);
        ALLOCATIONS.with_borrow_mut(|s| { s[id as usize] = vec.clone(); });
        let memory_vec = ALLOCATIONS.with_borrow(|s| s.get(id as usize).unwrap().to_owned());
        assert_eq!(memory_vec, vec);
    }
}
