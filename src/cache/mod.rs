use HashMap;


/// As I understand it, from the Java Version I looked at, This is
/// basically two structures: A hash table for a fast lookup, and a
/// doubly linked list.  Every time you "hit" something, you move it
/// to the top of the list.

/// I want to keep this generic, but the value must be sized.  The
/// eviction routine is based on the maximum amount of memory the
/// value can be.

/// I really don't feel like I know what I'm doing here.  This doesn't
/// seem to build on itself correctly, although it works on paper!
/// Grr.  I want values to MOVE into the HashMap and LIVE there, and I
/// want them to go away when the value is deleted, and I want
/// REFERENCES to the value to be made available to consumers
/// (i.e. consumers may borrow the value, but immutably and
/// non-permanently.)


pub struct LruCacheItem<V: Sized> {
    value: V,
    prev: *mut LruCacheItem<T>,  // I have no idea how to make this part work in Rust yet.
    next: *mut LruCacheItem<T>
};

pub struct LruCache<K: Eq + Hash, T: PartialEq + Sized> {
    cache: HashMap<K, LruCacheItem<T>>,
    size: usize,
    head *mut LruCacheItem<T>,
    tail *mut LruCacheItem<T>
};

impl LruCacheItem<V> {
    fn new(value: V) -> Self {
        LruCacheItem {
            value: value,
            prev: ptr::null_mut(),
            next: ptr::null_mut()
        }
    }
}

impl LruCache<K: Hash + Eq, V: Sized> {

    pub fn new() -> LruCache<K, V> {
        let mut lrucache = LruCache {
            cache: Hashmap::new(),
            head: unsafe { ptr::null_mut() },
            tail: unsafe { ptr::null_mut() }
        }
        
        unsafe {
            (*lrucache.head) = &lrucache.tail;
            (*lrucache.tail) = &lrucache.head;
        }
        lrucache
    }

    pub fn insert(&mut self, k: K, v: V) {
        let item = match self.cache.get_mut(k) {
            Some(val) => {
                self.check_eviction(v.len() - val.len());
                LruCacheItem::new(v);
            }

            None => {
                self.check_eviction(v.len());
                LruCacheItem::new(v);
            }
        }
        unsafe {
            item.next = (*self.head).next;
            item.prev = self.head;
            *(self.head).next = &item;
            *(item.next).prev = &item;
        }
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        let item = try!(match self.cache.remove(k));
        let ptr = item.next.prev;
        item.prev.next = item.next;
        item.next.prev = ptr;
        item.value
    }


    pub fn get(&self, key: K) -> Option<V> {
        let item = try!(self.cache.get(key));
        item.value
    }

    fn check_eviction(&mut self, request: usize) {
        let mut done = false;
        loop {
            if self.head == self.tail {
                break;
            }
            if self.size + request < MAX_CACHESIZE {
                break;
            }
            self.remove(*self.tail);
        }
    }
};

        

    
                
