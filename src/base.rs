use std::{fmt, iter::FusedIterator};

use crate::sam::SeqComplement;

/// A base represents the IUPAC ambiguity codes
/// There are 16 possible codes, so Base can not be more than 15
///
/// 0 -> No base
/// 1 -> A
/// 2 -> C
/// 3 -> M (A | C)
/// 4 -> G
/// 5 -> R (A | G)
/// 6 -> S (C | G)
/// 7 -> V (A | C | G)
/// 8 -> T
/// 9 -> W (A | T)
/// 10 -> Y (C | T)
/// 11 -> H (A | C | T)
/// 12 -> K (G | T)
/// 13 -> D (A | G | T)
/// 14 -> B (C | G | T)
/// 15 -> N (A | C | G | T)
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Base(u8);

impl Base {
    #[inline]
    pub const fn new(x: u8) -> Self {
        Self(x & 0xf)
    }

    #[inline]
    pub const fn from_u8(c: u8) -> Self {
        Self(SEQ_NT16_TABLE[c as usize])
    }

    #[inline]
    pub const fn combine(&self, other: &Self) -> u8 {
        (self.0 << 4) | other.0
    }

    #[inline]
    pub const fn as_u8(&self) -> u8 {
        self.0
    }

    #[inline]
    pub const fn as_char(&self) -> char {
        BASE_TABLE[self.0 as usize] as char
    }
    
    #[inline]
    pub const fn single_base(&self) -> Option<u8> {
        SINGLE_BASE[self.0 as usize]
    }

    #[inline]
    pub const fn is_no_base(&self) -> bool {
        self.0 as u8 == 0
    }

    #[inline]
    pub const fn complement(&self) -> Self {
        Self(self.0.reverse_bits() >> 4)
    }
}

impl SeqComplement for Base {
    fn get_complement(&self) -> Self {
        self.complement()
    }
}

impl fmt::Display for Base {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_char())
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BaseQual {
    base: Base,
    qual: u8,
}

impl BaseQual {
    #[inline]
    pub fn new(base: Base, qual: u8) -> Self {
        Self { base, qual }
    }
    
    #[inline]
    pub fn base(&self) -> Base {
        self.base
    }
    
    #[inline]
    pub fn qual(&self) -> u8 {
        self.qual
    }
    
    #[inline]
    pub fn base_qual(&self) -> (Base, u8) {
        (self.base, self.qual)
    }
}

impl SeqComplement for BaseQual {
    fn get_complement(&self) -> Self {
        Self { base: self.base.complement(), qual: self.qual }
    }
}

const BASE_TABLE: &[u8; 16] = b"-ACMGRSVTWYHKDBN";

const SEQ_NT16_TABLE: [u8; 256] = [
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    1, 2, 4, 8, 15, 15, 15, 15, 15, 15, 15, 15, 15, 0, 15, 15, 15, 1, 14, 2, 13, 15, 15, 4, 11, 15,
    15, 12, 15, 3, 15, 15, 15, 15, 5, 6, 8, 15, 7, 9, 15, 10, 15, 15, 15, 15, 15, 15, 15, 1, 14, 2,
    13, 15, 15, 4, 11, 15, 15, 12, 15, 3, 15, 15, 15, 15, 5, 6, 8, 15, 7, 9, 15, 10, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15, 15, 15, 15,
];

const SINGLE_BASE: [Option<u8>; 16] = [
    None, Some(0), Some(1), None, Some(2), None, None, None, Some(3), None, None, None, None, None, None, None
];

pub struct BaseIter<'a> {
    inner: &'a [u8],
}

impl<'a> BaseIter<'a> {
    pub fn new(v: &'a [u8]) -> Self {
        Self { inner: v }
    }
}

impl<'a> Iterator for BaseIter<'a> {
    type Item = Base;

    fn next(&mut self) -> Option<Self::Item> {
        if self.inner.is_empty() {
            None
        } else {
            let b = Base::from_u8(self.inner[0]);
            self.inner = &self.inner[1..];
            Some(b)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.inner.len();
        (n, Some(n))
    }

    fn count(self) -> usize {
        self.inner.len()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.inner
            .get(n)
            .map(|b| {
                let b = Base::from_u8(*b);
                self.inner = &self.inner[n + 1..];
                b
            })
            .or_else(|| {
                self.inner = &[];
                None
            })
    }

    fn last(self) -> Option<Self::Item> {
        self.inner.last().map(|b| Base::from_u8(*b))
    }
}

impl<'a> ExactSizeIterator for BaseIter<'a> {}
impl<'a> FusedIterator for BaseIter<'a> {}

impl<'a> DoubleEndedIterator for BaseIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let l = self.inner.len();
        if l > 0 {
            let b = Base::from_u8(self.inner[l - 1]);
            self.inner = &self.inner[..l - 1];
            Some(b)
        } else {
            None
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let l = self.inner.len();
        if l > n {
            let b = Base::from_u8(self.inner[l - 1 - n]);
            self.inner = &self.inner[..l - 1 - n];
            Some(b)
        } else {
            self.inner = &[];
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn base_iter() {
        let mut itr = BaseIter::new(b"ACCG");
        assert_eq!(itr.next(), Some(Base::from_u8(b'A')));
        assert_eq!(itr.nth(2), Some(Base::from_u8(b'G')));
        assert_eq!(itr.next(), None);
    }

    #[test]
    fn base_iter_dbl() {
        let mut itr = BaseIter::new(b"ACCGTG");
        assert_eq!(itr.next(), Some(Base::from_u8(b'A')));
        assert_eq!(itr.next_back(), Some(Base::from_u8(b'G')));
        assert_eq!(itr.nth_back(1), Some(Base::from_u8(b'G')));
        assert_eq!(itr.next(), Some(Base::from_u8(b'C')));
        assert_eq!(itr.nth(5), None);
    }
}
