
use std::cell::RefCell;

thread_local! {
    pub static ALLOCATIONS: RefCell<Vec<Option<Vec<u8>>>> = RefCell::new(Vec::new());
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
        s.push(Some(buf));
        s.len() - 1
    }) as u32
}

#[no_mangle]
pub fn get_allocation(allocation_id: u32) -> *const u8 {
    ALLOCATIONS.with_borrow(|s| {
        let allocation = s.get(allocation_id as usize).unwrap();
        let vec = allocation.as_ref().unwrap();
        vec.as_ptr()
    })
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocation() {

        // create allocation
        let id = create_allocation(1);
        let allocation = ALLOCATIONS.with_borrow(|s| s[id as usize].clone());
        assert_eq!(allocation.is_some(), true);
        assert_eq!(get_allocation(id).is_null(), false);

        // create another allocation
        let id2 = create_allocation(1);
        let allocation = ALLOCATIONS.with_borrow(|s| s[id2 as usize].clone());
        assert_eq!(allocation.is_some(), true);

    }

    #[test]
    fn test_memory() {

        // test string
        let text = "hello";
        let id = create_allocation(1);
        ALLOCATIONS.with_borrow_mut(|s| {
            s[id as usize] = Some(text.as_bytes().to_vec());
        });
        let memory_text = get_string_from_allocation(id);
        assert_eq!(memory_text, text);

        // test vec
        let vec = vec![1, 2];
        let id = create_allocation(1);
        ALLOCATIONS.with_borrow_mut(|s| {
            s[id as usize] = Some(vec.clone());
        });
        let memory_vec = get_vec_from_allocation(id);
        assert_eq!(memory_vec, vec);
    }
}
