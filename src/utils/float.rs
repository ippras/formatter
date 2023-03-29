pub(crate) trait FloatExt {
    fn is_approx_integer(&self) -> bool;

    fn is_approx_zero(&self) -> bool;
}

impl FloatExt for f64 {
    fn is_approx_integer(&self) -> bool {
        self.fract().abs() < f64::EPSILON
    }

    fn is_approx_zero(&self) -> bool {
        self.abs() < f64::EPSILON
    }
}
