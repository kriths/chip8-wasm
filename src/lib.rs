use wasm_bindgen::__rt::std::fs::read;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(num1: i32, num2: i32) -> i32 {
    num1 + num2
}
