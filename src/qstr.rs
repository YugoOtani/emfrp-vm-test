// this is based on micropython qstr implementation
// https://github.com/micropython/micropython/blob/master/py/qstr.c
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QstrIndex(usize);
const POOLSIZE_MIN: usize = 1;

#[derive(Debug)]
pub struct QstrPool {
    parent: Option<Box<QstrPool>>,
    qstrs: Vec<String>,
    total_prev_len: usize,
}
impl QstrPool {
    pub fn empty() -> Self {
        Self {
            parent: None,
            qstrs: Vec::with_capacity(POOLSIZE_MIN),
            total_prev_len: 0,
        }
    }
    fn _get(&self, n: usize) -> Option<&str> {
        self.qstrs.get(n).map(|x| x.as_str())
    }
    pub fn get(&self, QstrIndex(n): QstrIndex) -> Option<&str> {
        let mut pool = self;
        while n < pool.total_prev_len {
            pool = pool.parent.as_ref().unwrap()
        }
        pool._get(n - pool.total_prev_len)
    }
    pub fn find(&self, s: &str) -> Option<QstrIndex> {
        for (i, s2) in self.qstrs.iter().enumerate() {
            if s2 == s {
                return Some(QstrIndex(i + self.total_prev_len));
            }
        }
        if let Some(ref par) = self.parent {
            par.find(s)
        } else {
            None
        }
    }
    pub fn insert(mut self, s: String) -> (Self, QstrIndex) {
        match self.find(&s) {
            Some(ind) => (self, ind),
            None => {
                let current = self.qstrs.capacity();
                if current == self.qstrs.len() {
                    let total_prev_len = current + self.total_prev_len;
                    let mut qstrs = Vec::with_capacity(current * 2);
                    qstrs.push(s.to_string());
                    let new = Self {
                        parent: Some(Box::new(self)),
                        qstrs,
                        total_prev_len,
                    };
                    (new, QstrIndex(total_prev_len))
                } else {
                    self.qstrs.push(s);
                    let n = self.qstrs.len();
                    (self, QstrIndex(n))
                }
            }
        }
    }
}

#[test]
fn qstrpool_test() {
    let mut pool = QstrPool::empty();
    for s in ["a", "b", "c", "d", "a", "c", "e", "f", "a"] {
        (pool, _) = pool.insert(s.to_string())
    }
    for (i, s) in ["a", "b", "c", "d", "e", "f"].iter().enumerate() {
        println!("{}", s);
        assert_eq!(Some(QstrIndex(i)), pool.find(s));
        assert_eq!(Some(*s), pool.get(QstrIndex(i)));
    }
}
