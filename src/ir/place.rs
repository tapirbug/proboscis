#[derive(Debug, Clone, Copy)]
pub struct PlaceAddress(i32);

impl PlaceAddress {
    pub fn new_unsafe(from: i32) -> Self {
        Self(from)
    }
}
