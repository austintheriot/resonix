/// Creates a linear ramp from 0.0 -> 1.0 -> 0.0
/// 
/// The `current_index` is assumed to range from 0.0 -> 1.0
/// 
/// This method of generating envelope values is much faster than using a sine wave calculation.
pub fn generate_triangle_envelope_value_from_percent(current_index: f32) -> f32 {
    (((current_index - 0.5).abs() * -1.0) + 0.5) * 2.0
}

pub fn i16_array_to_f32(data: Vec<i16>) -> Vec<f32> {
    data.into_iter().map(|el| el as f32 / 65536.0).collect()
}