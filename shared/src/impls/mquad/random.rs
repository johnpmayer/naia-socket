/// Container for cross-platform Random methods
#[derive(Debug)]
pub struct Random {}

impl Random {
    /// returns a random f32 value between an upper & lower bound
    pub fn gen_range_f32(lower: f32, upper: f32) -> f32 {
        return 0.0;
    }

    /// returns a random u32 value between an upper & lower bound
    pub fn gen_range_u32(lower: u32, upper: u32) -> u32 {
        return 0;
    }

    /// returns a random boolean value between an upper & lower bound
    pub fn gen_bool() -> bool {
        true
    }
}
