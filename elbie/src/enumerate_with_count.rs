use core::iter::Enumerate;

/* A simple iterator that lets me access the next index when enumerating. */

// FUTURE: The Enumerate struct does have an unstable next_index function, which if I had that would mean I wouldn't need this struct. When that becomes stable I can rewrite and delete this module.

#[derive(Clone)]
pub(crate) struct EnumerateCount<Inner> {
    inner: Enumerate<Inner>,
    next_index: usize
}

impl<Inner: Iterator> EnumerateCount<Inner> {
    pub(crate) fn new(inner: Inner) -> Self {
        Self { inner: inner.enumerate(),
               next_index: 0 }
    }

    pub(crate) const fn next_index(&self) -> usize {
        self.next_index
    }
}

impl<Iter> Iterator for EnumerateCount<Iter> where Iter: Iterator {
    type Item = (usize, <Iter as Iterator>::Item);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((index, item)) = self.inner.next() {
            self.next_index = index + 1;
            Some((index, item))
        } else {
            None
        }
    }
}
