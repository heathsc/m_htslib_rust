use std::iter::FusedIterator;

use crate::sam::SeqComplement;

pub struct Compl<T> {
    iter: T,
}

impl<I, T> Iterator for Compl<I>
where
    I: Iterator<Item = T>,
    T: SeqComplement,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|b| b.get_complement())
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I, T> DoubleEndedIterator for Compl<I>
where
    I: DoubleEndedIterator<Item = T>,
    T: SeqComplement,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(|b| b.get_complement())
    }
}

impl<I, T> ExactSizeIterator for Compl<I>
where
    I: ExactSizeIterator + Iterator<Item = T>,
    T: SeqComplement,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl<I, T> FusedIterator for Compl<I>
where
    I: FusedIterator + Iterator<Item = T>,
    T: SeqComplement,
{
}

pub trait IterCompl<T> {
    fn compl(self) -> Compl<Self>
    where
        Self: Iterator<Item = T> + Sized,
        T: SeqComplement,
    {
        Compl { iter: self }
    }
}

impl<I, T> IterCompl<T> for I
where
    I: Iterator<Item = T>,
    T: SeqComplement,
{
}
