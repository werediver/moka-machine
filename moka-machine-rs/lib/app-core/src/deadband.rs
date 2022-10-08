use core::{
    marker::PhantomData,
    ops::{Bound, RangeBounds},
};

#[derive(Copy, Clone, Debug)]
pub struct Deadband<R, T>
where
    T: ?Sized,
    R: RangeBounds<T>,
{
    bounds: R,
    _n: PhantomData<*const T>,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum DeadbandComparisonResult {
    Low,
    Hold,
    High,
}

impl<R, T> Deadband<R, T>
where
    R: RangeBounds<T>,
{
    pub fn new(bounds: R) -> Self {
        Self {
            bounds,
            _n: Default::default(),
        }
    }

    pub fn compare<U>(&self, value: &U) -> DeadbandComparisonResult
    where
        T: PartialOrd<U>,
        U: ?Sized + PartialOrd<T>,
    {
        match value {
            x if self.is_below(x) => DeadbandComparisonResult::Low,
            x if self.is_above(x) => DeadbandComparisonResult::High,
            _ => DeadbandComparisonResult::Hold,
        }
    }

    fn is_below<U>(&self, value: &U) -> bool
    where
        T: PartialOrd<U>,
        U: ?Sized + PartialOrd<T>,
    {
        match self.bounds.start_bound() {
            Bound::Included(bound) => value < bound,
            Bound::Excluded(bound) => value <= bound,
            Bound::Unbounded => false,
        }
    }

    fn is_above<U>(&self, value: &U) -> bool
    where
        T: PartialOrd<U>,
        U: ?Sized + PartialOrd<T>,
    {
        match self.bounds.end_bound() {
            Bound::Included(bound) => value > bound,
            Bound::Excluded(bound) => value >= bound,
            Bound::Unbounded => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison() {
        let d = Deadband::new(0.9..=1.1f32);
        assert_eq!(d.compare(&0.5), DeadbandComparisonResult::Low);
        assert_eq!(d.compare(&1.0), DeadbandComparisonResult::Hold);
        assert_eq!(d.compare(&1.5), DeadbandComparisonResult::High);
    }

    #[test]
    fn boundary_conditions() {
        assert_eq!(
            Deadband::new(..1.0f32).compare(&1.0),
            DeadbandComparisonResult::High
        );
        assert_eq!(
            Deadband::new(..=1.0f32).compare(&1.0),
            DeadbandComparisonResult::Hold
        );
        assert_eq!(
            Deadband::new(..).compare(&1.0),
            DeadbandComparisonResult::Hold
        );
    }
}
