use std::iter::FusedIterator;

use crate::{
    base::Base,
    sam::{SeqComplement, cigar::CigarOp},
};

use super::cigar::CigarElem;

pub struct AlignIterElem<T> {
    ref_seq: Option<Base>,
    seq: Option<T>,
}

impl<T> AlignIterElem<T> {
    fn make(seq: Option<T>, ref_seq: Option<Base>) -> Self {
        Self { ref_seq, seq }
    }

    pub fn seq(&self) -> Option<&T> {
        self.seq.as_ref()
    }

    pub fn ref_seq(&self) -> Option<Base> {
        self.ref_seq
    }
}

impl<T> SeqComplement for AlignIterElem<T>
where
    T: SeqComplement,
{
    fn get_complement(&self) -> Self {
        Self {
            ref_seq: self.ref_seq.as_ref().map(|b| b.get_complement()),
            seq: self.seq.as_ref().map(|x| x.get_complement()),
        }
    }
}

pub struct AlignIter<S, C, R, T>
where
    S: Iterator<Item = T> + FusedIterator,
    R: Iterator<Item = Base> + FusedIterator,
    C: Iterator<Item = CigarElem> + FusedIterator,
    T: Sized,
{
    seq: S,
    ref_seq: R,
    cigar_elems: C,
    current_elem: Option<CigarElem>,
    current_elem_rev: Option<CigarElem>,
}

impl<S, C, R, T> AlignIter<S, C, R, T>
where
    S: Iterator<Item = T> + FusedIterator,
    R: Iterator<Item = Base> + FusedIterator,
    C: Iterator<Item = CigarElem> + FusedIterator,
    T: Sized,
{
    pub fn new(seq: S, ref_seq: R, cigar_elems: C) -> Self {
        Self {
            seq,
            ref_seq,
            cigar_elems,
            current_elem: None,
            current_elem_rev: None,
        }
    }

    fn get_next_cigar_elem(&mut self) -> bool {
        self.current_elem = self
            .cigar_elems
            .next()
            .or_else(|| self.current_elem_rev.take());
        self.current_elem.is_none()
    }
}

impl<S, C, R, T> AlignIter<S, C, R, T>
where
    S: Iterator<Item = T> + FusedIterator + DoubleEndedIterator,
    R: Iterator<Item = Base> + FusedIterator + DoubleEndedIterator,
    C: Iterator<Item = CigarElem> + FusedIterator + DoubleEndedIterator,
    T: Sized,
{
    fn get_next_cigar_elem_back(&mut self) -> bool {
        self.current_elem_rev = self
            .cigar_elems
            .next_back()
            .or_else(|| self.current_elem.take());
        self.current_elem_rev.is_none()
    }
}

impl<S, C, R, T> Iterator for AlignIter<S, C, R, T>
where
    S: Iterator<Item = T> + FusedIterator,
    R: Iterator<Item = Base> + FusedIterator,
    C: Iterator<Item = CigarElem> + FusedIterator,
    T: Sized,
{
    type Item = AlignIterElem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mk = |s, r| AlignIterElem::make(s, r);

        loop {
            if let Some(ret) = self.current_elem.take().and_then(|e| {
                if e.op_len() > 0 {
                    print!("OOOK! CurrentElem {}", e);
                    self.current_elem = e.decr_len();
                    if let Some(e1) = self.current_elem {
                        println!(" -> {e1}");
                    } else {
                        println!(" -> None");
                    }
                    match e.op() {
                        CigarOp::Match | CigarOp::Diff | CigarOp::Equal => {
                            Some(mk(self.seq.next(), self.ref_seq.next()))
                        }
                        CigarOp::Ins => Some(mk(self.seq.next(), None)),
                        CigarOp::Del => Some(mk(None, self.ref_seq.next())),
                        CigarOp::SoftClip => {
                            self.seq.next();
                            None
                        }
                        CigarOp::RefSkip => {
                            self.ref_seq.next();
                            None
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }) {
                return Some(ret);
            }
            if self.current_elem.is_none() && self.get_next_cigar_elem() {
                break;
            }
        }
        None
    }
}

impl<S, C, R, T> DoubleEndedIterator for AlignIter<S, C, R, T>
where
    S: Iterator<Item = T> + DoubleEndedIterator + FusedIterator,
    R: Iterator<Item = Base> + DoubleEndedIterator + FusedIterator,
    C: Iterator<Item = CigarElem> + DoubleEndedIterator + FusedIterator,
    T: Sized,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let mk = |s, r| AlignIterElem::make(s, r);

        loop {
            if let Some(ret) = self.current_elem_rev.take().and_then(|e| {
                if e.op_len() > 0 {
                    self.current_elem_rev = e.decr_len();
                    match e.op() {
                        CigarOp::Match | CigarOp::Diff | CigarOp::Equal => {
                            Some(mk(self.seq.next_back(), self.ref_seq.next_back()))
                        }
                        CigarOp::Ins => Some(mk(self.seq.next_back(), None)),
                        CigarOp::Del => Some(mk(None, self.ref_seq.next_back())),
                        CigarOp::SoftClip => {
                            self.seq.next_back();
                            None
                        }
                        CigarOp::RefSkip => {
                            self.ref_seq.next_back();
                            None
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }) {
                return Some(ret);
            }
            if self.current_elem_rev.is_none() && self.get_next_cigar_elem_back() {
                break;
            }
        }
        None
    }
}

impl<S, C, R, T> FusedIterator for AlignIter<S, C, R, T>
where
    S: Iterator<Item = T> + FusedIterator,
    R: Iterator<Item = Base> + FusedIterator,
    C: Iterator<Item = CigarElem> + FusedIterator,
    T: Sized,
{
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{base::BaseIter, iterators::IterCompl, sam::cigar::CigarBuf};

    #[test]
    fn test_align_iter1() {
        let seq = BaseIter::from("ATTCAGGTC");
        let rf = BaseIter::from("ATTCCAGATC");
        let cb = "4M1D5M".parse::<CigarBuf>().expect("Error parsing Cigar");

        let itr = AlignIter::new(seq, rf, cb.elems());
        let mut v = Vec::new();
        for e in itr {
            let a = e.seq().copied().unwrap_or_default();
            let b = e.ref_seq().unwrap_or_default();
            println!("{a}\t{b}");
            v.push((a, b))
        }
        assert_eq!(v[7], (Base::from_u8(b'G'), Base::from_u8(b'A')));
        assert_eq!(v.len(), 10);
    }

    #[test]
    fn test_align_iter2() {
        let seq = BaseIter::from("ATTCAGGTC");
        let rf = BaseIter::from("ATTCCAGATC");
        let cb = "4M1D5M".parse::<CigarBuf>().expect("Error parsing Cigar");

        let itr = AlignIter::new(seq, rf, cb.elems()).rev().compl();
        let mut v = Vec::new();
        for e in itr {
            let a = e.seq().copied().unwrap_or_default();
            let b = e.ref_seq().unwrap_or_default();
            println!("{a}\t{b}");
            v.push((a, b))
        }
        assert_eq!(v[7], (Base::from_u8(b'A'), Base::from_u8(b'A')));
        assert_eq!(v.len(), 10);
    }

    #[test]
    fn test_align_iter3() {
        let seq = BaseIter::from("ATTCAGGTC");
        let rf = BaseIter::from("ATTCCAGATC");
        let cb = "4M1D5M".parse::<CigarBuf>().expect("Error parsing Cigar");

        let mut itr = AlignIter::new(seq, rf, cb.elems());
        let e = itr.nth(3).expect("Missing base");
        assert_eq!(e.ref_seq(), Some(Base::from_u8(b'C')));
        let e = itr.nth_back(2).expect("Missing base");
        assert_eq!(e.ref_seq(), Some(Base::from_u8(b'A')));
        assert_eq!(e.seq().copied(), Some(Base::from_u8(b'G')));
    }
}
