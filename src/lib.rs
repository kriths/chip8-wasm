use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(num1: i32, num2: i32) -> i32 {
    num1 + num2
}

#[cfg(test)]
mod tests {
    use super::add;

    #[test]
    fn it_works() {
        assert_eq!(add(2, 2), 4);
    }
}
