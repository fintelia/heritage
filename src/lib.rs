
#![allow(dead_code)]

use std::boxed::Box;
use std::sync::Arc;

#[derive(Debug)]
enum Child<T> {
    Exclusive(Box<T>),
    Shared(Arc<Box<T>>),
    None,
}

impl<T> Child<T>
    where T: Clone
{
    fn make_exclusive(&mut self) {
        if let Child::Shared(_) = *self {
            use std::mem;
            use std::ops::Deref;

            if let Child::Shared(arc) = mem::replace(self, Child::None) {
                *self = Child::Exclusive(arc.deref().clone());
            }
        }
    }
    fn make_shared(&mut self) {
        if let Child::Exclusive(_) = *self {
            use std::mem;

            if let Child::Exclusive(b) = mem::replace(self, Child::None) {
                *self = Child::Shared(Arc::new(b));
            }
        }
    }
    fn is_none(&self) -> bool {
        match self {
            &Child::None => true,
            &_ => false,
        }
    }
}

#[derive(Debug)]
struct Node<T>
    where T: Clone
{
    children: [Child<Node<T>>; 2],
    element: T,
}

impl<T> Node<T>
    where T: Clone,
          T: Ord
{
    fn new(element: T) -> Self {
        Node {
            children: [Child::None, Child::None],
            element: element,
        }
    }
    fn make_shared(&mut self) {
        for child in self.children.iter_mut() {
            if let &mut Child::Exclusive(ref mut b) = child {
                b.make_shared();
            }
            child.make_shared();
        }
    }
    fn insert(&mut self, v: T) {
        if self.element == v {
            return;
        }

        let index = if v < self.element { 0 } else { 1 };
        if let Child::None = self.children[index] {
            self.children[index] = Child::Exclusive(Box::new(Node::new(v)));
        } else {
            self.children[index].make_exclusive();
            if let Child::Exclusive(ref mut child) = self.children[index] {
                child.insert(v);
            }
        }
    }
    fn contains(&self, v: T) -> bool {
        if self.element == v {
            return true;
        }

        let index = if v < self.element { 0 } else { 1 };
        match self.children[index] {
            Child::Exclusive(ref b) => b.contains(v),
            Child::Shared(ref arc) => arc.contains(v),
            Child::None => false,
        }
    }
    fn snapshot(&mut self) -> Node<T> {
        self.make_shared();
        self.clone()
    }
}

impl<T> Clone for Node<T>
    where T: Clone
{
    fn clone(&self) -> Node<T> {
        let duplicate_child = |child: &Child<Node<T>>| {
            match child {
                &Child::Shared(ref c) => Child::Shared(c.clone()),
                &Child::None => Child::None,
                &Child::Exclusive(_) => unreachable!(),
            }
        };
        Node {
            children: [duplicate_child(&self.children[0]), duplicate_child(&self.children[1])],
            element: self.element.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use Node;

    #[test]
    fn clone_works() {
        let mut tree = Node::new(12);
        tree.insert(15);
        tree.insert(5);
        tree.insert(8);
        tree.insert(22);
        assert!(tree.contains(5));
        assert!(tree.contains(15));
        assert!(tree.contains(8));
        assert!(tree.contains(22));
        assert!(!tree.contains(13));
        tree.make_shared();

        assert!(tree.contains(5));
        assert!(tree.contains(15));
        assert!(tree.contains(8));
        assert!(tree.contains(22));
        assert!(!tree.contains(13));

        let mut tree2 = tree.clone();

        assert!(tree.contains(5));
        assert!(tree.contains(15));
        assert!(tree.contains(8));
        assert!(tree.contains(22));
        assert!(!tree.contains(13));
        assert!(tree2.contains(5));
        assert!(tree2.contains(15));
        assert!(tree2.contains(8));
        assert!(tree2.contains(22));
        assert!(!tree2.contains(13));

        tree2.insert(1);
        assert!(!tree.contains(1));
        assert!(tree2.contains(1));
    }

    #[test]
    fn snapshot_works() {
        let mut tree = Node::new(12);
        tree.insert(15);
        tree.insert(5);
        tree.insert(8);
        tree.insert(22);
        assert!(tree.contains(5));
        assert!(tree.contains(15));
        assert!(tree.contains(8));
        assert!(tree.contains(22));
        assert!(!tree.contains(13));

        let mut tree2 = tree.snapshot();

        assert!(tree.contains(5));
        assert!(tree.contains(15));
        assert!(tree.contains(8));
        assert!(tree.contains(22));
        assert!(!tree.contains(13));
        assert!(tree2.contains(5));
        assert!(tree2.contains(15));
        assert!(tree2.contains(8));
        assert!(tree2.contains(22));
        assert!(!tree2.contains(13));

        tree2.insert(1);
        assert!(!tree.contains(1));
        assert!(tree2.contains(1));
    }
}
