use super::js::ExternRef;


// convert invoke parameters into bytes
// assuming each parameter is preceded by a 32 bit integer indicating its type
// 0 = undefined
// 1 = null
// 2 = float-64
// 3 = bigint
// 4 = string (followed by 32-bit start and size of string in memory)
// 5 = extern ref
// 6 = array of float-64 (followed by 32-bit start and size of string in memory)
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
    fn from(f: f64) -> Self { InvokeParam::Float64(f) }
}

impl From<i32> for InvokeParam<'_> {
    fn from(i: i32) -> Self { InvokeParam::Float64(i as f64) }
}

impl From<usize> for InvokeParam<'_> {
    fn from(i: usize) -> Self { InvokeParam::Float64(i as f64) }
}

impl From<i64> for InvokeParam<'_> {
    fn from(i: i64) -> Self { InvokeParam::BigInt(i) }
}

impl<'a> From<&'a str> for InvokeParam<'a> {
    fn from(s: &'a str) -> Self { InvokeParam::String(s) }
}

impl<'a> From<&'a ExternRef> for InvokeParam<'a> {
    fn from(i: &'a ExternRef) -> Self { InvokeParam::ExternRef(i) }
}

impl<'a> From<&'a [f32]> for InvokeParam<'a> {
    fn from(a: &'a [f32]) -> Self { InvokeParam::Float32Array(a) }
}

impl<'a> From<&'a [f64]> for InvokeParam<'a> {
    fn from(a: &'a [f64]) -> Self { InvokeParam::Float64Array(a) }
}

impl From<bool> for InvokeParam<'_> {
    fn from(b: bool) -> Self { InvokeParam::Bool(b) }
}

impl<'a> From<&'a [u32]> for InvokeParam<'a> {
    fn from(a: &'a [u32]) -> Self { InvokeParam::Uint32Array(a) }
}

pub fn param_to_bytes(params: &[InvokeParam]) -> Vec<u8> {
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
                let start = s.as_ptr() as u32;
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
                let start = a.as_ptr() as u32;
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
                let start = a.as_ptr() as u32;
                let len = a.len();
                param_bytes.extend_from_slice(&start.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
            InvokeParam::Uint32Array(a) => {
                param_bytes.push(10);
                let start = a.as_ptr() as u32;
                let len = a.len();
                param_bytes.extend_from_slice(&start.to_le_bytes());
                param_bytes.extend_from_slice(&len.to_le_bytes());
            }
        }
    }
    param_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params() {
        
        // undefined
        assert_eq!(param_to_bytes(&[InvokeParam::Undefined]), vec![0]);

        // null
        assert_eq!(param_to_bytes(&[InvokeParam::Null]), vec![1]);

        // bigint
        assert_eq!(param_to_bytes(&[InvokeParam::BigInt(42)]), [vec![3], 42u64.to_le_bytes().to_vec()].concat());

        // string
        let text = "hello";
        let text_ptr = text.as_ptr() as u32;
        let text_len = text.len() as u64;
        let expected = [vec![4], text_ptr.to_le_bytes().to_vec(), vec![1, 0, 0, 0], text_len.to_le_bytes().to_vec()].concat();
        assert_eq!(param_to_bytes(&[InvokeParam::String(text)]), expected);

        // extern ref
        assert_eq!(param_to_bytes(&[InvokeParam::ExternRef(&ExternRef { value: 42 })]), [vec![5], 42u64.to_le_bytes().to_vec()].concat());
        
        // float32 array
        let array = [1.0, 2.0];
        let array_ptr = array.as_ptr() as u32;
        let array_len = array.len() as u64;
        let expected = [vec![6], array_ptr.to_le_bytes().to_vec(), vec![0, 112, 0, 0], array_len.to_le_bytes().to_vec()].concat();
        assert_eq!(param_to_bytes(&[InvokeParam::Float32Array(&array)]), expected);
        
        // float64 array
        let array = [1.0, 2.0];
        let array_ptr = array.as_ptr() as u32;
        let array_len = array.len() as u64;
        let expected = [vec![9], array_ptr.to_le_bytes().to_vec(), vec![0, 112, 0, 0], array_len.to_le_bytes().to_vec()].concat();
        assert_eq!(param_to_bytes(&[InvokeParam::Float64Array(&array)]), expected);
        
        // bool
        assert_eq!(param_to_bytes(&[InvokeParam::Bool(true)]), vec![7]);
        
        // u32 array
        let array = [1, 2];
        let array_ptr = array.as_ptr() as u32;
        let array_len = array.len() as u64;
        let expected = [vec![10], array_ptr.to_le_bytes().to_vec(), vec![0, 112, 0, 0], array_len.to_le_bytes().to_vec()].concat();
        assert_eq!(param_to_bytes(&[InvokeParam::Uint32Array(&array)]), expected);

    }
}