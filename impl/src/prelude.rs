// --------------------------------------------------
// external
// --------------------------------------------------
use std::hash::Hash;
use std::collections::HashMap;

/// [Counts] trait
pub trait Counts {
    type Item;
    /// Counts the occurrences of each item in the iterator.
    fn counts(&mut self) -> HashMap<Self::Item, usize>;
}

/// [Counts] implementation for all iterators
impl<'a, T, I> Counts for I
where
    T: Eq + Hash,
    I: Iterator<Item = T> + 'a,
{
    type Item = T;
    fn counts(&mut self) -> HashMap<T, usize> {
        let mut counts = HashMap::new();
        self.for_each(|item| (*counts.entry(item).or_insert(0_usize) += 1_usize));
        counts
    }
}

/// [Positions] trait
pub trait Positions<T> {
    fn positions(&mut self) -> HashMap<T, Vec<usize>>;
}

/// [Positions] implementation for all iterators
impl<T, I> Positions<T> for I
where
    T: Eq + Hash,
    I: Iterator<Item = T>,
{
    fn positions(&mut self) -> HashMap<T, Vec<usize>> {
        let mut counts = HashMap::new();
        self.enumerate().for_each(|(index, item)| {
            counts.entry(item).or_insert_with(Vec::new).push(index);
        });
        counts
    }
}

/// [Repeated] trait 
pub trait Repeated<T> {
    fn repeated(&mut self) -> Vec<T>;
}

/// [Repeated] implementation for all iterators
impl<T, I> Repeated<T> for I
where
    T: Eq + Hash + Clone,
    I: Iterator<Item = T>,
{
    fn repeated(&mut self) -> Vec<T> {
        self.counts()
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(item, _)| item.clone())
            .collect()
    }
}

/// [RepeatedIndices] trait
pub trait RepeatedIndices<T> {
    fn repeated_idx(&mut self) -> Vec<usize>;
}

/// [RepeatedIndices] implementation for all iterators
impl<T, I> RepeatedIndices<T> for I
where
    T: Eq + Hash + Clone,
    I: Iterator<Item = T> + ExactSizeIterator,
{
    fn repeated_idx(&mut self) -> Vec<usize> {
        self.into_iter()
            .positions()
            .into_iter()
            .filter(|(_, indices)| indices.len() > 1)
            .flat_map(|(_, indices)| indices)
            .collect()
    }
}

/// [Unique] trait
pub trait Unique<T> {
    fn _unique(&mut self) -> Vec<T>;
}

/// [Unique] implementation for all iterators
impl<T, I> Unique<T> for I
where
    T: Eq + Hash + Clone,
    I: Iterator<Item = T>,
{
    fn _unique(&mut self) -> Vec<T> {
        self.counts()
            .into_iter()
            .filter(|(_, count)| *count == 1)
            .map(|(item, _)| item.clone())
            .collect()
    }
}

/// [UniqueIndices] trait
pub trait UniqueIndices<T> {
    fn _unique_idx(&mut self) -> Vec<usize>;
}

/// [UniqueIndices] implementation for all iterators
impl<T, I> UniqueIndices<T> for I
where
    T: Eq + Hash + Clone,
    I: Iterator<Item = T> + ExactSizeIterator,
{
    fn _unique_idx(&mut self) -> Vec<usize> {
        self.into_iter()
            .positions()
            .into_iter()
            .filter(|(_, indices)| indices.len() == 1)
            .flat_map(|(_, indices)| indices)
            .collect()
    }
}