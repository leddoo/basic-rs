
#[inline(always)]
pub const fn splat_4(b: u8) -> u32 {
    u32::from_ne_bytes([b; 4])
}

#[inline(always)]
pub const fn splat_8(b: u8) -> u64 {
    u64::from_ne_bytes([b; 8])
}


macro_rules! bitmask_impl {
    ($name:ident, $ty:ident, $splat:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub struct $name($ty);

        impl $name {
            pub const NONE: Self = Self(0);
            pub const ALL: Self = Self($splat(0x80));

            #[inline(always)]
            pub const fn from_bits(bits: $ty) -> Self { Self(bits) }


            #[inline(always)]
            pub const fn none(self) -> bool { self.0 == 0 }

            #[inline(always)]
            pub const fn any(self) -> bool { self.0 != 0 }

            #[inline(always)]
            pub const fn all(self) -> bool { self.0 == $splat(0x80) }


            #[inline(always)]
            pub fn not(self) -> Self {
                Self(self.0 ^ $splat(0x80))
            }


            #[inline(always)]
            pub fn find_zero_bytes(value: $ty) -> Self {
                // https://graphics.stanford.edu/~seander/bithacks.html#ZeroInWord
                let zero_or_high = value.wrapping_sub($splat(1));
                let not_high = !value & $splat(0x80);
                let mask = zero_or_high & not_high;
                Self(mask)
            }

            #[inline(always)]
            pub fn find_equal_bytes(value: $ty, byte: u8) -> Self {
                Self::find_zero_bytes(value ^ $splat(byte))
            }

            #[inline(always)]
            pub fn find_high_bit_bytes(value: $ty) -> Self {
                Self(value & $splat(0x80))
            }
        }

        impl Iterator for $name {
            type Item = usize;

            #[inline(always)]
            fn next(&mut self) -> Option<Self::Item> {
                if self.0 != 0 {
                    let i = self.0.trailing_zeros() / 8;
                    self.0 &= self.0 - 1;
                    return Some(i as usize);
                }
                return None;
            }
        }
    }
}

bitmask_impl!(Bitmask4, u32, splat_4);
bitmask_impl!(Bitmask8, u64, splat_8);

