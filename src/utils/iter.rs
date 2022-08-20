pub struct TakeSome<'a, I>
where
    I: Iterator,
{
    n: usize,
    inner: &'a mut I,
}

impl<'a, I> Iterator for TakeSome<'a, I>
where
    I: Iterator,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n > 0 {
            self.n -= 1;
            self.inner.next()
        } else {
            None
        }
    }
}

type TryIntoResult<F, T> = Result<T, <T as TryFrom<F>>::Error>;

pub trait IteratorExt: Iterator {
    fn take_some(&mut self, n: usize) -> TakeSome<Self>
    where
        Self: Sized,
    {
        TakeSome { n, inner: self }
    }

    fn try_take_array<const N: usize>(&mut self) -> TryIntoResult<Vec<Self::Item>, [Self::Item; N]>
    where
        Self: Sized,
        Vec<Self::Item>: FromIterator<Self::Item>,
        [Self::Item; N]: TryFrom<Vec<Self::Item>>,
    {
        self.take_some(N).collect::<Vec<Self::Item>>().try_into()
    }

    fn take_array<const N: usize>(&mut self) -> [Self::Item; N]
    where
        Self: Sized,
        Self::Item: std::fmt::Debug,
    {
        self.try_take_array().unwrap()
    }
}

impl<I: Iterator> IteratorExt for I {}
