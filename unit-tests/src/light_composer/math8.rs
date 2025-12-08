#[cfg(test)]
mod tests {
    use embassy_time::Duration;
    use myrtio_light_composer::math8::{blend8, progress8, scale8};

    #[test]
    fn test_scale8() {
        assert_eq!(scale8(255, 128), 128);
        assert_eq!(scale8(0, 128), 0);
        assert_eq!(scale8(128, 128), 64);
        assert_eq!(scale8(128, 255), 128);
        assert_eq!(scale8(128, 0), 0);
    }

    #[test]
    fn test_blend8() {
        assert_eq!(blend8(255, 128, 128), 191);
        assert_eq!(blend8(0, 128, 255), 128);
        assert_eq!(blend8(255, 0, 128), 127);
        assert_eq!(blend8(255, 128, 0), 255);
    }

    #[test]
    fn test_progress8() {
        assert_eq!(progress8(Duration::from_millis(0), Duration::from_millis(100)), 0);
        assert_eq!(progress8(Duration::from_millis(50), Duration::from_millis(100)), 127);
        assert_eq!(progress8(Duration::from_millis(100), Duration::from_millis(100)), 255);
    }
}