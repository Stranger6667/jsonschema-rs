use std::sync::Arc;

pub struct List<T> {
    head: Option<Arc<Node<T>>>,
}

impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> Self {
        List {
            head: self.head.clone(),
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut current = self.head.take();
        while let Some(node) = current {
            if let Ok(mut node) = Arc::try_unwrap(node) {
                current = node.next.take();
            } else {
                break;
            }
        }
    }
}

impl<T> List<T> {
    pub(crate) fn new() -> Self {
        Self { head: None }
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }
    #[must_use]
    pub fn push_front(&self, value: Arc<T>) -> Self {
        List {
            head: Some(Arc::new(Node {
                value,
                next: self.head.clone(),
            })),
        }
    }
    #[must_use]
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            current: self.head.as_ref(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Node<T> {
    value: Arc<T>,
    next: Option<Arc<Node<T>>>,
}

#[derive(Debug)]
pub struct Iter<'a, T> {
    current: Option<&'a Arc<Node<T>>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.map(|current| {
            let value = &current.value;
            self.current = current.next.as_ref();
            &**value
        })
    }
}

impl<'a, T> IntoIterator for &'a List<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
