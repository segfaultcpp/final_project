pub trait GetIndex<T>
where
    T: Eq + PartialEq,
{
    fn get_index(&self, value: &T) -> Option<usize>;
}

impl<T> GetIndex<T> for Vec<T>
where
    T: Eq + PartialEq,
{
    fn get_index(&self, value: &T) -> Option<usize> {
        self.iter()
            .enumerate()
            .find(|(_, elem)| **elem == *value)
            .map(|(idx, _)| idx)
    }
}
