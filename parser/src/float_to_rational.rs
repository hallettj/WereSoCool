pub mod helpers {
    use num_rational::{Ratio, Rational64};
    use std::str::FromStr;

    pub fn f32_to_rational(float_string: String) -> Rational64 {
        let decimal = float_string.split('.').collect::<Vec<&str>>()[1];
        let den = i64::pow(10, decimal.len() as u32);
        let num = i64::from_str(&float_string.replace('.', "")).unwrap();

        Ratio::new(num, den)
    }

    #[cfg(test)]
    pub mod tests {
        use super::*;

        #[test]
        fn test_float_to_rational() {
            let result = f32_to_rational("112.999".to_string());
            let expected = Ratio::new(112999, 1000);
            assert_eq!(result, expected);
        }
    }
}
