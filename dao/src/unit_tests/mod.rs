mod dao;
mod release;

pub const DURATION_1Y_S: u32 = 31_536_000;
pub const DURATION_2Y_S: u32 = 63_072_000;
pub const DURATION_3Y_S: u32 = 94_608_000;

pub const RELEASE_TIME: u64 = 63_072_000_000_000_000;
pub const DURATION_ONE_WEEK: u64 = 604_800_000_000_000;
pub const DURATION_1Y: u64 = 31_536_000_000_000_000;
pub const DURATION_2Y: u64 = 63_072_000_000_000_000;
pub const DURATION_3Y: u64 = 94_608_000_000_000_000;

#[macro_export]
macro_rules! create_val_to_percent_closure {
    ($e:expr, $t:ty) => {
        |p| {
            ($e as u128 * p / 100) as $t
        }
    };
}