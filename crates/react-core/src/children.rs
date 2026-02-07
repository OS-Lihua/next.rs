use std::ops::Deref;

pub struct Children<T>(Vec<T>);

impl<T> Children<T> {
    pub fn new(children: Vec<T>) -> Self {
        Self(children)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }

    pub fn into_vec(self) -> Vec<T> {
        self.0
    }
}

impl<T> Deref for Children<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> Default for Children<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> FromIterator<T> for Children<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl<T> IntoIterator for Children<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_children_basic() {
        let children: Children<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()]
            .into_iter()
            .collect();

        assert_eq!(children.len(), 3);
        assert!(!children.is_empty());
    }

    #[test]
    fn test_children_iteration() {
        let children: Children<i32> = vec![1, 2, 3].into_iter().collect();
        let sum: i32 = children.iter().sum();
        assert_eq!(sum, 6);
    }

    #[test]
    fn test_children_default() {
        let children: Children<String> = Children::default();
        assert!(children.is_empty());
        assert_eq!(children.len(), 0);
    }
}
