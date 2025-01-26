use std::collections::BinaryHeap;

pub trait IntoOrderedIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;

    fn into_ordered_iter(self) -> Self::IntoIter;
}

pub struct BinaryHeapOrderedIter<T>(BinaryHeap<T>)
where
    T: Ord;

impl<T> Iterator for BinaryHeapOrderedIter<T>
where
    T: Ord,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<T> IntoOrderedIterator for BinaryHeap<T>
where
    T: Ord,
{
    type Item = T;

    type IntoIter = BinaryHeapOrderedIter<T>;

    fn into_ordered_iter(self) -> Self::IntoIter {
        BinaryHeapOrderedIter(self)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BinaryHeap;

    use pretty_assertions::assert_eq;

    use super::IntoOrderedIterator;

    #[test]
    fn iterates_binary_heap_in_sorted_order() {
        // given
        let binary_heap = BinaryHeap::from([1, 3, 2, 9, 5, 8, 4, 6, 7]);

        // when
        let ordered_vec = binary_heap.into_ordered_iter().collect::<Vec<_>>();

        // then
        assert_eq!(vec![9, 8, 7, 6, 5, 4, 3, 2, 1], ordered_vec);
    }
}
