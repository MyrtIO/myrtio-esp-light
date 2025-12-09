mod gradient;
mod utils;

use smart_leds::RGB8;
use smart_leds::hsv::Hsv as HSV;

pub use gradient::{GradientDirection, fill_gradient_fp, fill_gradient_three_fp};
pub use utils::{blend_colors, hsv2rgb, mirror_half, rgb_from_u32};

pub type Rgb = RGB8;
pub type Hsv = HSV;
