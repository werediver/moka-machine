use core::ops::{Range, RangeInclusive};

use num::Num;

struct Deadband<N: Num> {
    boundaries: RangeInclusive<N>,
}

impl<N: Num> Deadband<N> {
    fn new(boundaries: RangeInclusive<N>) -> Self {
        Self { boundaries }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
