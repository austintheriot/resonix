use std::ops::{Add, Mul};

/// Creates a linear ramp from 0.0 -> 1.0 -> 0.0
///
/// The `percent` is assumed to range from 0.0 -> 1.0
///
/// This method of generating envelope values is much faster than using a sine wave calculation.
/// Generates a minimum output of 0 and a maximum output of 1.0.
pub fn generate_triangle_envelope_value_from_percent(percent: f32) -> f32 {
    if percent < 0.5 {
        percent.mul(2.0)
    } else {
        percent.mul(-2.0).add(2.0)
    }
    .max(0.0)
}
