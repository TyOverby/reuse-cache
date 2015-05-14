#![feature(std_misc)]

use std::rc::Rc;
use std::cell::{RefCell, BorrowState};
use std::ops::{Deref, DerefMut};

#[derive(Clone)]
pub struct ReuseCache<T> {
    all: Rc<Vec<RefCell<Option<T>>>>
}

pub struct Item<T> {
    parent_cache: ReuseCache<T>,
    idx: usize,
    item: Option<T>
}

impl <T> ReuseCache<T> {
    pub fn new<F: FnMut() -> T>(count: usize, mut init: F) -> ReuseCache<T> {
        let mut v = Vec::new();
        v.extend((0 .. count).map(|_| RefCell::new(Some(init()))));
        ReuseCache { all: Rc::new(v) }
    }

    pub fn get(&self) -> Option<Item<T>> {
        for (i, slot) in self.all.iter().enumerate() {
            if slot.borrow_state() == BorrowState::Unused {
                if slot.borrow().is_some() {
                    return Some(Item {
                        parent_cache: ReuseCache{ all: self.all.clone() },
                        idx: i,
                        item: slot.borrow_mut().take()
                    })
                }
            }
        }

        None
    }
}

impl <T> Item<T> {
    pub fn replace(&mut self, new: T) -> T {
        let old = self.item.take().unwrap();
        self.item = Some(new);
        old
    }
}

impl <T> Deref for Item<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.item.as_ref().unwrap()
    }
}

impl <T> DerefMut for Item<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.item.as_mut().unwrap()
    }
}

impl <T> Drop for Item<T> {
    fn drop(&mut self) {
        let it = self.item.take();
        *(self.parent_cache.all.get(self.idx).unwrap().borrow_mut()) = it;
    }
}

#[test]
fn test_empty() {
    let rc = ReuseCache::new(0, || 0u32);
    assert!(rc.get().is_none())
}

#[test]
fn test_single() {
    let rc = ReuseCache::new(1, || 5u32);
    assert!(&*rc.get().unwrap() == &5u32)
}

#[test]
fn test_reuse() {
    let rc = ReuseCache::new(1, || 5u32);

    {
        let mut it = rc.get().unwrap();
        *it = 10u32;
    }

    {
        let it = rc.get().unwrap();
        assert!(&*it == &10u32)
    }
}

#[test]
fn test_taken() {
    let rc = ReuseCache::new(1, || 5u32);
    let it1 = rc.get();
    assert!(it1.is_some());
    let it2 = rc.get();
    assert!(it2.is_none());
}

#[test]
fn test_replace() {
    let rc = ReuseCache::new(1, || 5u32);
    {
        let mut it = rc.get().unwrap();
        assert!(it.replace(4) == 5);
    }

    {
        let it = rc.get().unwrap();
        assert!(*it == 4)
    }
}
