mod tests {
    use embassy_time::{Duration, Instant};
    use myrtio_light_composer::{color::Rgb, transition::ValueTransition};

    #[test]
    fn test_value_transition_u8() {
        let mut transition = ValueTransition::new_u8(0);
        assert_eq!(transition.current(), 0);
        assert_eq!(transition.is_transitioning(), false);
        transition.set(100, Duration::from_millis(100), Instant::from_millis(0));
        assert_eq!(transition.is_transitioning(), true);

        transition.tick(Instant::from_millis(50));
        assert_eq!(transition.current(), 50);

        transition.tick(Instant::from_millis(100));
        assert_eq!(transition.current(), 100);
        assert_eq!(transition.is_transitioning(), false);
    }

    #[test]
    fn test_value_transition_rgb() {
        let mut transition = ValueTransition::new_rgb(Rgb::new(0, 0, 0));
        assert_eq!(transition.current(), Rgb::new(0, 0, 0));
        assert_eq!(transition.is_transitioning(), false);
        transition.set(
            Rgb::new(255, 255, 255),
            Duration::from_millis(100),
            Instant::from_millis(0),
        );
        assert_eq!(transition.is_transitioning(), true);
    }
}
