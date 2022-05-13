/// Calculates percents from given values.
/// Rounds > 0.5 up.
/// Invariants that must be uphold by caller:
/// - `total` >= value
/// - `total` <= 10^32
/// Panics or gives invalid results otherwise.
/// Tested in wasm32 environment via workspaces-rs in Sandbox and Testnet.
pub fn calculate_percent_u128(value: u128, total: u128) -> u8 {
    (((value * 10_000) / total) as f64 / 100.0).round() as u8
}
