use std::iter::FusedIterator;

use crate::{base::Base, sam::{cigar::CigarOp, SeqComplement}};

use super::cigar::CigarElem;

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
    type Item = (Option<T>, Option<Base>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ret) = self.current_elem.take().and_then(|e| {
                if e.op_len() > 0 {
                    self.current_elem = e.decr_len();
                    match e.op() {
                        CigarOp::Match | CigarOp::Diff | CigarOp::Equal => {
                            Some((self.seq.next(), self.ref_seq.next()))
                        }
                        CigarOp::Ins => Some((self.seq.next(), None)),
                        CigarOp::Del => Some((None, self.ref_seq.next())),
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
            if self.get_next_cigar_elem() {
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
    T: Sized + SeqComplement,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ret) = self.current_elem_rev.take().and_then(|e| {
                if e.op_len() > 0 {
                    self.current_elem_rev = e.decr_len();
                    match e.op() {
                        CigarOp::Match | CigarOp::Diff | CigarOp::Equal => {
                            Some((self.seq.next_back().map(|b| b.get_complement()), self.ref_seq.next().map(|b| b.get_complement())))
                        }
                        CigarOp::Ins => Some((self.seq.next_back().map(|b| b.get_complement()), None)),
                        CigarOp::Del => Some((None, self.ref_seq.next().map(|b| b.get_complement()))),
                        CigarOp::SoftClip => {
                            self.seq.next_back().map(|b| b.get_complement());
                            None
                        }
                        CigarOp::RefSkip => {
                            self.ref_seq.next().map(|b| b.get_complement());
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
            if self.get_next_cigar_elem_back() {
                break;
            }
        }
        None
    }
}
