#[derive(Debug, Clone)]
pub struct Ewma {
    alpha: f32,
    value: Option<f32>,
}

impl Ewma {
    pub fn new(alpha: f32) -> Self {
        debug_assert!((0.0..=1.0).contains(&alpha));
        Self { alpha, value: None }
    }

    pub fn update(&mut self, sample: f32) -> f32 {
        let next = match self.value {
            Some(current) => current + self.alpha * (sample - current),
            None => sample,
        };
        self.value = Some(next);
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_value_is_not_biased_toward_zero() {
        let mut filter = Ewma::new(0.35);
        assert_eq!(filter.update(72.0), 72.0);
    }

    #[test]
    fn subsequent_values_are_smoothed() {
        let mut filter = Ewma::new(0.35);
        assert_eq!(filter.update(20.0), 20.0);
        assert!((filter.update(100.0) - 48.0).abs() < 0.001);
    }
}
