use crate::Thermistor;

#[test]
fn kelvin_to_celsius() {
    let thermistor = Thermistor::new_celsius(0., 0., 0.);

    assert_eq!(thermistor.ref_temp_kelvin, 273.15);
}

macro_rules! assert_f32_eq {
    ($epsilon:expr, $lhs:expr, $rhs:expr) => {
        if ($lhs - $rhs).abs() > $epsilon {
            assert_eq!($lhs, $rhs)
        }
    };
    ($lhs:expr, $rhs:expr) => {
        assert_f32_eq!(f32::EPSILON, $lhs, $rhs)
    }
}

/// Real panasonic NTC thermistor - test values based on datasheet
///
/// https://industrial.panasonic.com/cdbs/www-data/pdf/AUA0000/AUA0000C8.pdf
// The datasheet's Page 5 table uses 3375 as $\beta_{25/50}$ and 3435 as $\beta_{25/85}$ but
// Page 4 with part ID uses 3380 as $\beta_{25/50}$
mod ertjvg103_a {
    use crate::Thermistor;

    #[test]
    fn beta_25_50() {
        let thermistor = Thermistor::new_celsius(1e4, 25., 3375.);

        assert_f32_eq!(0.1, 25., thermistor.get_temp_celsius(1e4));
        assert_f32_eq!(0.1, 50., thermistor.get_temp_celsius(1e4 / 0.4165));
    }

    #[test]
    fn beta_25_85() {
        let thermistor = Thermistor::new_celsius(1e4, 25., 3435.);
        assert_f32_eq!(0.1, 25., thermistor.get_temp_celsius(1e4));
        assert_f32_eq!(0.1, 85., thermistor.get_temp_celsius(1e4 / 0.1451));
    }
}
