pub mod sketch;
mod hash;

pub trait MinHashVector {
    fn get_register(index: usize) -> u32;

    fn set_register(index: usize, value: u32);
}
