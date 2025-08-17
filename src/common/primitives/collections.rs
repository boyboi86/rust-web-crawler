/// Collection primitive building blocks for optional vector handling
/// Level 3 implementation - complete struct and functionality for collection types
use serde::{Deserialize, Serialize};

/// Building block for optional vectors - ensures consistent optional collection handling
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct OptionalVec<T>(Option<Vec<T>>);

impl<T> OptionalVec<T> {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self(Some(Vec::with_capacity(capacity)))
    }

    pub fn from_vec(vec: Vec<T>) -> Self {
        if vec.is_empty() {
            Self(None)
        } else {
            Self(Some(vec))
        }
    }

    pub fn from_option(option: Option<Vec<T>>) -> Self {
        Self(option)
    }

    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub fn is_empty(&self) -> bool {
        match &self.0 {
            Some(vec) => vec.is_empty(),
            None => true,
        }
    }

    pub fn len(&self) -> usize {
        match &self.0 {
            Some(vec) => vec.len(),
            None => 0,
        }
    }

    pub fn capacity(&self) -> usize {
        match &self.0 {
            Some(vec) => vec.capacity(),
            None => 0,
        }
    }

    pub fn push(&mut self, item: T) {
        match &mut self.0 {
            Some(vec) => vec.push(item),
            None => self.0 = Some(vec![item]),
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        match &mut self.0 {
            Some(vec) => {
                let result = vec.pop();
                if vec.is_empty() {
                    self.0 = None;
                }
                result
            }
            None => None,
        }
    }

    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        let items: Vec<T> = iter.into_iter().collect();
        if items.is_empty() {
            return;
        }

        match &mut self.0 {
            Some(vec) => vec.extend(items),
            None => self.0 = Some(items),
        }
    }

    pub fn append(&mut self, other: &mut OptionalVec<T>) {
        if let Some(other_vec) = other.0.take() {
            self.extend(other_vec);
        }
    }

    pub fn clear(&mut self) {
        self.0 = None;
    }

    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        if let Some(vec) = &mut self.0 {
            vec.retain(|item| f(item));
            if vec.is_empty() {
                self.0 = None;
            }
        }
    }

    pub fn truncate(&mut self, len: usize) {
        if let Some(vec) = &mut self.0 {
            vec.truncate(len);
            if vec.is_empty() {
                self.0 = None;
            }
        }
    }

    pub fn drain<R>(&mut self, range: R) -> impl Iterator<Item = T> + '_
    where
        R: std::ops::RangeBounds<usize>,
    {
        match &mut self.0 {
            Some(vec) => {
                let drained: Vec<T> = vec.drain(range).collect();
                if vec.is_empty() {
                    self.0 = None;
                }
                drained.into_iter()
            }
            None => Vec::new().into_iter(),
        }
    }

    pub fn as_slice(&self) -> &[T] {
        match &self.0 {
            Some(vec) => vec.as_slice(),
            None => &[],
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match &mut self.0 {
            Some(vec) => vec.as_mut_slice(),
            None => &mut [],
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.0.as_ref()?.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        self.0.as_mut()?.get_mut(index)
    }

    pub fn first(&self) -> Option<&T> {
        self.0.as_ref()?.first()
    }

    pub fn last(&self) -> Option<&T> {
        self.0.as_ref()?.last()
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        match &self.0 {
            Some(vec) => vec.iter(),
            None => [].iter(),
        }
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        match &mut self.0 {
            Some(vec) => vec.iter_mut(),
            None => [].iter_mut(),
        }
    }

    pub fn into_iter(self) -> impl Iterator<Item = T> {
        match self.0 {
            Some(vec) => vec.into_iter(),
            None => Vec::new().into_iter(),
        }
    }

    pub fn into_option(self) -> Option<Vec<T>> {
        self.0
    }

    pub fn into_vec(self) -> Vec<T> {
        match self.0 {
            Some(vec) => vec,
            None => Vec::new(),
        }
    }

    pub fn as_option(&self) -> Option<&Vec<T>> {
        self.0.as_ref()
    }

    pub fn as_mut_option(&mut self) -> Option<&mut Vec<T>> {
        self.0.as_mut()
    }

    pub fn take(&mut self) -> Option<Vec<T>> {
        self.0.take()
    }

    pub fn replace(&mut self, vec: Vec<T>) -> Option<Vec<T>> {
        self.0.replace(vec)
    }

    pub fn contains(&self, item: &T) -> bool
    where
        T: PartialEq,
    {
        match &self.0 {
            Some(vec) => vec.contains(item),
            None => false,
        }
    }

    pub fn sort(&mut self)
    where
        T: Ord,
    {
        if let Some(vec) = &mut self.0 {
            vec.sort();
        }
    }

    pub fn sort_by<F>(&mut self, compare: F)
    where
        F: FnMut(&T, &T) -> std::cmp::Ordering,
    {
        if let Some(vec) = &mut self.0 {
            vec.sort_by(compare);
        }
    }

    pub fn dedup(&mut self)
    where
        T: PartialEq,
    {
        if let Some(vec) = &mut self.0 {
            vec.dedup();
            if vec.is_empty() {
                self.0 = None;
            }
        }
    }

    pub fn reverse(&mut self) {
        if let Some(vec) = &mut self.0 {
            vec.reverse();
        }
    }
}

impl<T> From<Vec<T>> for OptionalVec<T> {
    fn from(vec: Vec<T>) -> Self {
        Self::from_vec(vec)
    }
}

impl<T> From<Option<Vec<T>>> for OptionalVec<T> {
    fn from(option: Option<Vec<T>>) -> Self {
        Self::from_option(option)
    }
}

impl<T> From<OptionalVec<T>> for Option<Vec<T>> {
    fn from(optional_vec: OptionalVec<T>) -> Self {
        optional_vec.0
    }
}

impl<T> From<OptionalVec<T>> for Vec<T> {
    fn from(optional_vec: OptionalVec<T>) -> Self {
        optional_vec.into_vec()
    }
}

impl<T> IntoIterator for OptionalVec<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        match self.0 {
            Some(vec) => vec.into_iter(),
            None => Vec::new().into_iter(),
        }
    }
}

impl<'a, T> IntoIterator for &'a OptionalVec<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> IntoIterator for &'a mut OptionalVec<T> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T> std::ops::Index<usize> for OptionalVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match self.0.as_ref() {
            Some(vec) => &vec[index],
            None => panic!("OptionalVec is None"),
        }
    }
}

impl<T> std::ops::IndexMut<usize> for OptionalVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.0.as_mut() {
            Some(vec) => &mut vec[index],
            None => panic!("OptionalVec is None"),
        }
    }
}

impl<T> Extend<T> for OptionalVec<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        self.extend(iter);
    }
}

impl<T> FromIterator<T> for OptionalVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let vec: Vec<T> = iter.into_iter().collect();
        Self::from_vec(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optional_vec_creation() {
        let empty = OptionalVec::<i32>::new();
        assert!(empty.is_none());
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let with_capacity = OptionalVec::<i32>::with_capacity(10);
        assert!(with_capacity.is_some());
        assert!(with_capacity.is_empty());
        assert_eq!(with_capacity.len(), 0);
        assert!(with_capacity.capacity() >= 10);

        let from_vec = OptionalVec::from_vec(vec![1, 2, 3]);
        assert!(from_vec.is_some());
        assert!(!from_vec.is_empty());
        assert_eq!(from_vec.len(), 3);
    }

    #[test]
    fn test_optional_vec_empty_handling() {
        let empty_vec = OptionalVec::<i32>::from_vec(vec![]);
        assert!(empty_vec.is_none());
        assert!(empty_vec.is_empty());

        let from_option_none = OptionalVec::<i32>::from_option(None);
        assert!(from_option_none.is_none());

        let from_option_empty = OptionalVec::<i32>::from_option(Some(vec![]));
        assert!(from_option_empty.is_some());
        assert!(from_option_empty.is_empty());
    }

    #[test]
    fn test_optional_vec_push_pop() {
        let mut opt_vec = OptionalVec::new();

        opt_vec.push(1);
        assert!(opt_vec.is_some());
        assert_eq!(opt_vec.len(), 1);

        opt_vec.push(2);
        assert_eq!(opt_vec.len(), 2);

        let popped = opt_vec.pop();
        assert_eq!(popped, Some(2));
        assert_eq!(opt_vec.len(), 1);

        let last = opt_vec.pop();
        assert_eq!(last, Some(1));
        assert!(opt_vec.is_none());
        assert!(opt_vec.is_empty());

        let nothing = opt_vec.pop();
        assert_eq!(nothing, None);
    }

    #[test]
    fn test_optional_vec_extend() {
        let mut opt_vec = OptionalVec::new();

        opt_vec.extend(vec![1, 2, 3]);
        assert_eq!(opt_vec.len(), 3);

        opt_vec.extend(vec![4, 5]);
        assert_eq!(opt_vec.len(), 5);

        let mut other = OptionalVec::from_vec(vec![6, 7]);
        opt_vec.append(&mut other);
        assert_eq!(opt_vec.len(), 7);
        assert!(other.is_none());
    }

    #[test]
    fn test_optional_vec_access() {
        let opt_vec = OptionalVec::from_vec(vec![10, 20, 30]);

        assert_eq!(opt_vec.get(0), Some(&10));
        assert_eq!(opt_vec.get(1), Some(&20));
        assert_eq!(opt_vec.get(3), None);

        assert_eq!(opt_vec.first(), Some(&10));
        assert_eq!(opt_vec.last(), Some(&30));

        let empty = OptionalVec::<i32>::new();
        assert_eq!(empty.first(), None);
        assert_eq!(empty.last(), None);
    }

    #[test]
    fn test_optional_vec_mutation() {
        let mut opt_vec = OptionalVec::from_vec(vec![1, 2, 3]);

        if let Some(item) = opt_vec.get_mut(1) {
            *item = 20;
        }

        assert_eq!(opt_vec.get(1), Some(&20));
    }

    #[test]
    fn test_optional_vec_iteration() {
        let opt_vec = OptionalVec::from_vec(vec![1, 2, 3]);

        let collected: Vec<_> = opt_vec.iter().collect();
        assert_eq!(collected, vec![&1, &2, &3]);

        let sum: i32 = opt_vec.iter().sum();
        assert_eq!(sum, 6);

        let empty = OptionalVec::<i32>::new();
        let empty_collected: Vec<_> = empty.iter().collect();
        assert!(empty_collected.is_empty());
    }

    #[test]
    fn test_optional_vec_into_iter() {
        let opt_vec = OptionalVec::from_vec(vec![1, 2, 3]);
        let collected: Vec<_> = opt_vec.into_iter().collect();
        assert_eq!(collected, vec![1, 2, 3]);

        let empty = OptionalVec::<i32>::new();
        let empty_collected: Vec<_> = empty.into_iter().collect();
        assert!(empty_collected.is_empty());
    }

    #[test]
    fn test_optional_vec_retain() {
        let mut opt_vec = OptionalVec::from_vec(vec![1, 2, 3, 4, 5]);

        opt_vec.retain(|&x| x % 2 == 0);
        assert_eq!(opt_vec.len(), 2);
        assert_eq!(opt_vec.as_slice(), &[2, 4]);

        opt_vec.retain(|&x| x > 10);
        assert!(opt_vec.is_none());
    }

    #[test]
    fn test_optional_vec_clear() {
        let mut opt_vec = OptionalVec::from_vec(vec![1, 2, 3]);
        assert!(!opt_vec.is_empty());

        opt_vec.clear();
        assert!(opt_vec.is_none());
        assert!(opt_vec.is_empty());
    }

    #[test]
    fn test_optional_vec_truncate() {
        let mut opt_vec = OptionalVec::from_vec(vec![1, 2, 3, 4, 5]);

        opt_vec.truncate(3);
        assert_eq!(opt_vec.len(), 3);
        assert_eq!(opt_vec.as_slice(), &[1, 2, 3]);

        opt_vec.truncate(0);
        assert!(opt_vec.is_none());
    }

    #[test]
    fn test_optional_vec_contains() {
        let opt_vec = OptionalVec::from_vec(vec![1, 2, 3]);

        assert!(opt_vec.contains(&2));
        assert!(!opt_vec.contains(&5));

        let empty = OptionalVec::<i32>::new();
        assert!(!empty.contains(&1));
    }

    #[test]
    fn test_optional_vec_sorting() {
        let mut opt_vec = OptionalVec::from_vec(vec![3, 1, 4, 1, 5]);

        opt_vec.sort();
        assert_eq!(opt_vec.as_slice(), &[1, 1, 3, 4, 5]);

        opt_vec.sort_by(|a, b| b.cmp(a));
        assert_eq!(opt_vec.as_slice(), &[5, 4, 3, 1, 1]);

        opt_vec.dedup();
        assert_eq!(opt_vec.as_slice(), &[5, 4, 3, 1]);

        opt_vec.reverse();
        assert_eq!(opt_vec.as_slice(), &[1, 3, 4, 5]);
    }

    #[test]
    fn test_optional_vec_conversions() {
        let vec = vec![1, 2, 3];
        let opt_vec = OptionalVec::from(vec.clone());
        assert_eq!(opt_vec.as_slice(), &[1, 2, 3]);

        let back_to_vec: Vec<i32> = opt_vec.clone().into();
        assert_eq!(back_to_vec, vec);

        let option: Option<Vec<i32>> = opt_vec.into();
        assert_eq!(option, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_optional_vec_from_iterator() {
        let opt_vec: OptionalVec<i32> = (1..=5).collect();
        assert_eq!(opt_vec.as_slice(), &[1, 2, 3, 4, 5]);

        let empty: OptionalVec<i32> = std::iter::empty().collect();
        assert!(empty.is_none());
    }

    #[test]
    fn test_optional_vec_indexing() {
        let opt_vec = OptionalVec::from_vec(vec![10, 20, 30]);

        assert_eq!(opt_vec[0], 10);
        assert_eq!(opt_vec[1], 20);
        assert_eq!(opt_vec[2], 30);
    }

    #[test]
    #[should_panic(expected = "OptionalVec is None")]
    fn test_optional_vec_indexing_none_panics() {
        let opt_vec = OptionalVec::<i32>::new();
        let _ = opt_vec[0]; // Should panic
    }

    #[test]
    fn test_optional_vec_drain() {
        let mut opt_vec = OptionalVec::from_vec(vec![1, 2, 3, 4, 5]);

        let drained: Vec<_> = opt_vec.drain(1..3).collect();
        assert_eq!(drained, vec![2, 3]);
        assert_eq!(opt_vec.as_slice(), &[1, 4, 5]);

        let all_drained: Vec<_> = opt_vec.drain(..).collect();
        assert_eq!(all_drained, vec![1, 4, 5]);
        assert!(opt_vec.is_none());
    }

    #[test]
    fn test_optional_vec_take_replace() {
        let mut opt_vec = OptionalVec::from_vec(vec![1, 2, 3]);

        let taken = opt_vec.take();
        assert_eq!(taken, Some(vec![1, 2, 3]));
        assert!(opt_vec.is_none());

        let replaced = opt_vec.replace(vec![4, 5]);
        assert_eq!(replaced, None);
        assert_eq!(opt_vec.as_slice(), &[4, 5]);
    }
}
