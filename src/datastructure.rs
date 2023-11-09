type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    car: T,
    cdr: Link<T>,
}

pub struct List<T> {
    head: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self { head: None }
    }
    pub fn push(&mut self, t: T) {
        let newnd = Some(Box::new(Node {
            car: t,
            cdr: std::mem::replace(&mut self.head, None),
        }));
        self.head = newnd;
    }
    pub fn pop(&mut self) -> Option<T> {
        match std::mem::replace(&mut self.head, Link::None) {
            Link::None => None,
            Link::Some(nd) => {
                let Node { car, cdr } = *nd;
                self.head = cdr;
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
    for i in 0..100 {
        lst.push(i);
    }
    for (i, e) in lst.iter().enumerate() {
        assert_eq!(*e, 100 - i - 1)
    }
}
