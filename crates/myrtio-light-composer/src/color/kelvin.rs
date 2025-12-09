use super::Rgb;

#[allow(clippy::approx_constant)]
const LN_LUT: [f32; 57] = [
    2.302_585, 2.397_895, 2.484_907, 2.564_949, 2.639_057, 2.707_606, 2.772_589, 2.833_213,
    2.890_372, 2.944_438, 2.995_732, 3.044_522, 3.091_042, 3.135_494, 3.178_054, 3.218_876,
    3.258_097, 3.295_837, 3.332_205, 3.367_296, 3.401_197, 3.433_987, 3.465_736, 3.496_508,
    3.526_361, 3.555_348, 3.583_519, 3.610_918, 3.637_586, 3.663_562, 3.688_879, 3.713_572,
    3.737_67, 3.761_2, 3.784_19, 3.806_662, 3.828_641, 3.850_148, 3.871_201, 3.891_82, 3.912_023,
    3.931_825, 3.951_244, 3.970_292, 3.988_984, 4.007_333, 4.025_352, 4.043_051, 4.060_443,
    4.077_537, 4.094_345, 4.110_874, 4.127_134, 4.143_134, 4.158_883, 4.174_387, 4.189_654,
];

#[inline]
/// Convert a Kelvin temperature to an RGB color
///
/// Supports temperatures between 1000K and 40000K.
#[allow(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn kelvin_to_rgb(kelvin: u16) -> Rgb {
    let mut temp = (kelvin as f32 / 100.0).clamp(10.0, 400.0);
    let original_temp = temp;

    let red = if temp <= 66.0 {
        255.0
    } else {
        temp -= 60.0;
        let result = 329.698_73 * libm::powf(temp, -0.133_204_76);
        result.clamp(0.0, 255.0)
    };

    let green = if original_temp <= 66.0 {
        let ln = if (original_temp as usize) < LN_LUT.len() {
            LN_LUT[original_temp as usize]
        } else {
            libm::log(original_temp as f64) as f32
        };
        99.470_8 * ln - 161.119_57
    } else {
        temp = original_temp - 60.0;
        288.122_17 * libm::powf(temp, -0.075_514_85)
    }
    .clamp(0.0, 255.0);

    let blue = if original_temp >= 66.0 {
        255.0
    } else if original_temp <= 19.0 {
        0.0
    } else {
        temp = original_temp - 10.0;
        let ln = if (temp as usize) < LN_LUT.len() {
            LN_LUT[temp as usize]
        } else {
            libm::log(temp as f64) as f32
        };
        138.517_73 * ln - 305.044_8
    }
    .clamp(0.0, 255.0);

    Rgb {
        r: red as u8,
        g: green as u8,
        b: blue as u8,
    }
}
