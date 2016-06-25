use ::Volatile;

#[macro_export]
macro_rules! is_bit_set {
    ($field:expr, $bit:expr) => (
        $field.get() & (1 << $bit) > 0
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
            let current = self.0.get();
            self.0.set(current | 1 << $bit);
        }
    )
}

#[macro_export]
macro_rules! bit_clear_fn {
    ($doc:meta, $fun:ident, $bit:expr) => (
        #[$doc]
        pub fn $fun(&mut self) {
            let current = self.0.get();
            self.0.set(current & !(1 << $bit)) ;
        }
    )
}

/// Return a range of bits out of a 32-bit data-type.
pub fn bits_get(r: &Volatile<u32>, from: usize, to: usize) -> u32 {
    assert!(from <= 31);
    assert!(to <= 31);
    assert!(from <= to);

    let mask = match to {
        31 => u32::max_value(),
        _ => (1 << (to + 1)) - 1,
    };

    (r.get() & mask) >> from
}
// #[macro_export]
// macro_rules! bit_set {
// ($r:expr, $to:expr, $from:expr, $bits:expr) => (
// assert!($from <= $to);
// let mask = match $to == 31 {
// true => u32::max_value() << $from,
// false => ((1 << ($to+1)) - 1) << $from
// };
// r = (*r & !mask) | ((bits << $from) & mask);
// )
// }
//

// where T:
// One +
// Add<Output = T> +
// Sub<Output = T> +
// Not<Output = T> +
// BitAnd<Output = T> +
// BitOr<Output = T> +
// Shl<T> +
// Shl<usize, Output = T> +
// Zero

// use std::ops::*;
// use std::num::*;
//
// pub fn bits_set<T: Integer>(r: &mut T, from: usize, to: usize, bits: T)
// where T: Add<usize> + Shl<usize, Output = T>, usize: Add<T, Output=T>
// {
// let mask: T = ((1 << (to+T::one())) - T::one()) << from;
// r = (*r & !mask) | ((bits << from) & mask);
// }


/// Set a range of bits in a 32-bit data-type.

pub fn bits_set(r: &mut Volatile<u32>, from: usize, to: usize, bits: u32) {
    assert!(from <= 31);
    assert!(to <= 31);
    assert!(from <= to);

    let mask = match to == 31 {
        true => u32::max_value() << from,
        false => ((1 << (to + 1)) - 1) << from,
    };

    let current = r.get();
    r.set((current & !mask) | ((bits << from) & mask));
}

pub fn bits_set_16(r: &mut Volatile<u16>, from: usize, to: usize, bits: u16) {
    assert!(from <= 15);
    assert!(to <= 15);
    assert!(from <= to);

    let mask = match to == 15 {
        true => u16::max_value() << from,
        false => ((1 << (to + 1)) - 1) << from,
    };

    let current = r.get();
    r.set((current & !mask) | ((bits << from) & mask));
}


#[cfg(test)]
mod tests {
    use bitops::*;

    #[test]
    fn bits_set_from_to() {
        for from in 0..32 {
            for to in from..32 {
                let mut r: Volatile<u32> = Volatile::with_value(0);
                let all_ones: usize = (1 << (to - from + 1)) - 1;


                bits_set(&mut r, from, to, all_ones as u32);

                for check in 0..32 {
                    if check >= from && check <= to {
                        assert!(is_bit_set!(r, check));
                    } else {
                        assert!(!is_bit_set!(r, check));
                    }
                }

                assert!(bits_get(r, from, to) == all_ones as u32);
            }
        }
    }
}
