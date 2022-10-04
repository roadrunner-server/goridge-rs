#![warn(rust_2018_idioms)]
pub mod bit_operations;
pub mod errors;
pub mod relay;
mod frame;
pub mod pipe;

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        println!("hello");
    }
}