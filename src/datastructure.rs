use std::fmt::Debug;
type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    car: T,
    cdr: Link<T>,
}
pub struct List<T> {
    head: Link<T>,
    len: usize,
}
impl<T> Default for List<T> {
    fn default() -> Self {
        Self { head: None, len: 0 }
    }
}
impl<T: Debug> Debug for List<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_list();
        for e in self.iter() {
            f.entry(e);
        }
        f.finish()
    }
}
impl<'a, T: 'a> List<T>
where
    &'a T: PartialEq,
{
    pub fn contains(&'a self, t: &'a T) -> bool {
        fn helper<'a, T: 'a>(lst: &'a Link<T>, t: &'a T) -> bool
        where
            &'a T: PartialEq,
        {
            match lst {
                None => false,
                Some(nd) => {
                    if t == &nd.car {
                        true
                    } else {
                        helper(&nd.cdr, t)
                    }
                }
            }
        }
        helper(&self.head, t)
    }
}
impl<T> List<T> {
    pub fn new() -> Self {
        Self { head: None, len: 0 }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        matches!(&self.head, None)
    }
    pub fn push(&mut self, t: T) {
        let newnd = Some(Box::new(Node {
            car: t,
            cdr: std::mem::take(&mut self.head),
        }));
        self.head = newnd;
        self.len += 1;
    }
    pub fn pop(&mut self) -> Option<T> {
        match std::mem::replace(&mut self.head, Link::None) {
            Link::None => None,
            Link::Some(nd) => {
                let Node { car, cdr } = *nd;
                self.head = cdr;
                self.len -= 1;
                Some(car)
            }
        }
    }
    pub fn iter(&self) -> ListIter<'_, T> {
        ListIter { cur: &self.head }
    }
}
pub struct ListIter<'a, T> {
    cur: &'a Link<T>,
}

impl<'a, T> Iterator for ListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cur {
            None => None,
            Some(nd) => {
                self.cur = &nd.cdr;
                Some(&nd.car)
            }
        }
    }
}
#[test]
fn linkedlist_test() {
    let mut lst = List::new();
    for i in 0..10 {
        lst.push(i);
    }
    println!("{:?}", lst);
}
