use std::iter::FusedIterator;

use super::{Cigar, CigarElem};

pub struct CigarIter<'a> {
    inner: &'a [CigarElem],
}

impl <'a> Iterator for CigarIter<'a> {
    type Item = CigarElem;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.inner.is_empty() {
            let e = self.inner[0];
            self.inner = &self.inner[1..];
            Some(e)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let l = self.inner.len();
        (l, Some(l))
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let l = self.inner.len();
        if l > n {
            let e = self.inner[n];
            self.inner = &self.inner[n + 1..];
            Some(e)
        } else {
            self.inner = &[];
            None
        }
    }
}

impl <'a> DoubleEndedIterator for CigarIter<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let l = self.inner.len();
        if l > 0 {
            let e = self.inner[l - 1];
            self.inner = &self.inner[.. l - 1];
            Some(e)
        } else {
            None
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        let l = self.inner.len();
        if l > n {
            let e = self.inner[l - 1 - n];
            self.inner = &self.inner[n + 1..];
            Some(e)
        } else {
            self.inner = &[];
            None
        }
    }
}

impl <'a> FusedIterator for CigarIter<'a> {}

impl <'a> ExactSizeIterator for CigarIter<'a> {}

impl Cigar {
    pub fn elems<'a>(&'a self) -> CigarIter<'a> {
        CigarIter{inner: self.as_elems()}
    }
}
