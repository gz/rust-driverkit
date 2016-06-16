#[macro_export]
macro_rules! is_bit_set {
    ($field:expr, $bit:expr) => (
        $field & (1 << $bit) > 0
    )
}

#[macro_export]
macro_rules! bit_get_fn {
    ($doc:meta, $fun:ident, $bit:expr) => (
        #[$doc]
        pub fn $fun(&self) -> bool {
            is_bit_set!(self.0, $bit)
        }
    )
}

#[macro_export]
macro_rules! bit_set_fn {
    ($doc:meta, $fun:ident, $bit:expr) => (
        #[$doc]
        pub fn $fun(&mut self) {
            self.0 |= 1 << $bit;
        }
    )
}

#[macro_export]
macro_rules! bit_clear_fn {
    ($doc:meta, $fun:ident, $bit:expr) => (
        #[$doc]
        pub fn $fun(&mut self) {
            self.0 &= !(1 << $bit);
        }
    )
}

/// Return a range of bits out of a 32-bit data-type.
pub fn bits_get(r: u32, from: usize, to: usize) -> u32 {
    assert!(from <= 31);
    assert!(to <= 31);
    assert!(from <= to);

    let mask = match to {
        31 => u32::max_value(),
        _ => (1 << (to+1)) - 1,
    };

    (r & mask) >> from
}

/// Set a range of bits in a 32-bit data-type.
pub fn bits_set(r: &mut u32, from: usize, to: usize, bits: u32) {
    assert!(from <= 31);
    assert!(to <= 31);
    assert!(from <= to);

    let mask = match to {
        31 => u32::max_value(),
        _ => ((1 << (to+1)) - 1) & !((1 << from) - 1),
    };

    *r = (*r & !mask) | ((bits << from) & mask);
}

#[cfg(test)]
mod tests {
    use bitops::*;
    
    #[test]
    fn bits_set_from_to() {
        for from in 0..32 {
            for to in from..32 {
                let mut r = 0;
                let all_ones: usize = (1 << (to - from + 1)) - 1;

                bits_set(&mut r, from, to, all_ones as u32);

                for check in 0..32 {
                    if check >= from && check <= to {
                        assert!(is_bit_set!(r, check));
                    }
                    else {
                        assert!(!is_bit_set!(r, check));
                    }
                }

                assert!(bits_get(r, from, to) == all_ones as u32);
            }
        }
    }
}
