mod tests {
    use embassy_time::Duration;
    use myrtio_light_composer::color::{Rgb, blend_colors, kelvin_to_rgb, mirror_half};

    const RED: Rgb = Rgb { r: 255, g: 0, b: 0 };
    const BLUE: Rgb = Rgb { r: 0, g: 0, b: 255 };
    const BLACK: Rgb = Rgb { r: 0, g: 0, b: 0 };
    const WHITE: Rgb = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };

    #[test]
    fn test_blend_colors() {
        assert_eq!(blend_colors(RED, BLUE, 0), RED);
        assert_eq!(blend_colors(RED, BLUE, 255), BLUE);
        assert_eq!(
            blend_colors(RED, BLUE, 128),
            Rgb {
                r: 127,
                g: 0,
                b: 128
            }
        );

        assert_eq!(
            blend_colors(BLACK, WHITE, 128),
            Rgb {
                r: 128,
                g: 128,
                b: 128
            }
        );
        assert_eq!(blend_colors(WHITE, BLACK, 255), BLACK);
        assert_eq!(blend_colors(WHITE, BLACK, 0), WHITE);
    }

    #[test]
    fn test_mirror_half() {
        let mut leds = [RED; 4];
        mirror_half(&mut leds);
        assert_eq!(leds, [RED, RED, RED, RED]);

        leds[0] = BLUE;
        leds[1] = WHITE;
        mirror_half(&mut leds);
        assert_eq!(leds, [BLUE, WHITE, WHITE, BLUE]);

        let mut leds = [BLUE, WHITE, RED, RED, RED];
        mirror_half(&mut leds);
        assert_eq!(leds, [BLUE, WHITE, RED, WHITE, BLUE]);
    }

    #[test]
    fn test_kelvin_to_rgb() {
        assert_eq!(kelvin_to_rgb(1000), (255, 136, 0));
        assert_eq!(kelvin_to_rgb(40000), (151, 185, 255));
    }
}
