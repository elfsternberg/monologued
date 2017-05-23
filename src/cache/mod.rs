use HashMap;


/* As I understand it, from the Java Version I looked at, This is
   basically two structures: A hash table for a fast lookup, and a
   doubly linked list.  Every time you "hit" something, you move it to
   the top of the list.

   I want to keep this generic, but the value must be sized.  The
   eviction routine is based on the maximum amount of memory the value
   can be.

   I really don't feel like I know what I'm doing here.  This doesn't
   seem to build on itself correctly, although it works on paper!
   Grr.  I want values to MOVE into the HashMap and LIVE there, and I
   want them to go away when the value is deleted, and I want
   REFERENCES to the value to be made available to consumers
   (i.e. consumers may borrow the value, but immutably and
   non-permanently.)
*/


struct LruKeyRef<K> {
    k: *const K
}

/* 
   Okay, this took me forever to work out.  We want to *move* the key
   and the value into the cache, but then we need a reference to the
   key so we can access it, and we need to able to get the hash of the
   key *from* the reference, as well as assert equality.

   Gad, the semicolon habit is hard to break! 

   Seriously, though, once you understand that this works so long as
   the key supports the hash() function, everything starts to fall
   into place.
*/

impl<K: Hash> Hash for LruKeyRef<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe { (*self.k).hash(state) }
    }
}


struct LruCacheItem<K: Hash + PartialEq, V: Sized> {
    key: K,
    value: V,
    prev: *mut LruCacheItem<K, V>,
    next: *mut LruCacheItem<K, V>
};

/* 
   AH!  &self is a *reference* to the reference *const K:k; it's a
   reference to a reference.  But other is also a reference, so why
   the &*other.key construct? -- Answer, because in this context, self
   automatically becomes a reference to self, and automatically
   appends the (unseen) & symbol.  A weird and inconvenient
   convenience.
*/

impl<K: PartialEq> PartialEq for LruKeyRef<K> {
    fn eq(&self, other: &LruKeyRef<K>) -> bool {
        unsafe { (*self.k).eq(&*other.k) }
    }
}

impl<K: Eq> Eq for KeyRef<K> {}

impl<K, V> LruCacheItem<K, V> {
    fn new(key: K, value: V) -> Self {
        LruCacheItem {
            key: key,
            value: value,
            size: usize,
            prev: ptr::null_mut(),
            next: ptr::null_mut()
        }
    }
}

pub struct LruCache<K, V> {
    cache: HashMap<LruKeyRef<K>, Box<LruCacheItem<V>>>,
    unitsize: usize,
    maxsize: usize,
    cursize: usize,
    head *mut LruCacheItem<K, V>,
    tail *mut LruCacheItem<K, V>
};

impl<K: Hash + Eq, V: Sized> LruCache<K, V> {
    pub fn new(usize maxcount, usize maxsize) -> LruCache<K, V> {
        let mut lrucache = LruCache {
            cache: Hashmap::new(),
            unitsize: maxsize,
            maxsize: maxsize * maxcount,
            cursize: 0,
.            /* I like how the documentation says "Really, reconsider
               before you do something like this." */
            head: unsafe { Box::into_raw(Box_new(mem::uninitialized::<LruCacheItem<K, V>>())) },
            tail: unsafe { Box::into_raw(Box_new(mem::uninitialized::<LruCacheItem<K, V>>())) }
        }
        
        /* Head now contains a pointer to an unboxed heap object that
           is an LruCacheItem, so that LruCacheItem's 'next' field can
           be addressed and set to the unboxed heap object Tail.  And
           vice versa.
         */
        unsafe {
            (*lrucache.head).next = lrucache.tail;
            (*lrucache.tail).prev = lrucache.head;
        }

        lrucache
    }

    /* I know LRU-RS used 'put', but 'insert' is what HashMap uses and
       this is mostly a hashmap implementation with some extra
       behavior.
     */
    pub fn insert(&mut self, key: K, value: V) -> bool {
        /* All right, let's walk through this.  First, get_mut tries
           to look up a key and return a value.  We've said that the
           map takes an LruKeyRef<K> as the key, and that K honors the
           traits Hash + PartialEq, so the "Hash for LruKeyRef"
           definition above is built to tell HashMap what bucket to
           use (delta whatever the Robin Hood Hashing algorithm does).
           We build a KeyRef out of a reference to the key (cheap),
           knowing that the type inside KeyRef is constant and
           therefore safe from tampering.

           The return value is a reference to a Boxed (heap pointer)
           LRUCache.  We create a new pointer to it.

           The '&mut' just says that we're returning a mutable
           reference (which we are).
         */

        if value.len() > self.unitsize {
            return False
        }
        
        let node_ptr = self.cache.get_mut(&KeyRef { k: &key }).map(|node| {
            /* I'm kinda surprised that assignments aren't expressions
               by themselves.
             */
            let node_ptr: *mut LruEntry<K, V> = &mut **node;
            node_ptr
        });
↓
        match node_ptr {
            Some(node_ptr) => {
                /* Previous entry exists.  If the previous value is
                   bigger than the request, we'll get a negative
                   number, which means no additional evictions are
                   necessary */
                self.check_eviction(value.len() - (*node_ptr).size);
                unsafe {
                    (*node_ptr).value = value;
                    (*node_ptr).size = value.len();
                }
                self.promote(node_ptr);
            }

            None => {
                /* check_eviction *MAY* return the last node it
                   removed.  Convenient for re-use.
                 */
                let mut old_node = self.check_eviction(value.len());
                let mut node = match old_node {
                    None => {
                        Box::New(LruCacheEntry(k, v))
                    }
                    Some(old_node) => {
                        old_node.key = key;
                        old_node.value = value;
                        old_node.size = value.len();
                        self.detach(old_node);
                        old_node
                    }
                }
                let node_ptr: *mut LruCacheEnty<K, V> = &mut *node;
                self.attach(node_ptr);
                let key = unsafe { &(*node_ptr).key };
                self.cache.insert(LruKeyRef { k: key }, node);
            }
        }
        True
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        let item = try!(match self.cache.remove(k));
        let ptr = item.next.prev;
        item.prev.next = item.next;
        item.next.prev = ptr;
        item.value
    }


    pub fn get<'a>(&a' mut self, key: &K) -> Option<&'a V> {
        let key = KeyRef { key: k }
        let (node_ptr, value) = match self.map.get_key(&key) {
            None => (None, None);
            Some(node) => {
                let node_ptr: &mut LruCacheEntry<K, V> = &mut **node;
                (Some(node_ptr), Some(unsafe { &(*node_ptr).value }))
            }
        };
        match node_ptr {
            None => (),
            Some(node_ptr) { self.promote(node_ptr); }
        }
        value
    }
                      

    pub fn has(&self, k: &K) -> bool {
        let key = KeyRef { k: k };
        self.map.contains_key(&key)
    }

    pub fn remove(&mut self, k: &K) -> Option<V> {
        let key = KeyRef { k: k };
        match self.map.remove(&key) {
            None => None,
            Some(entry) => Some(entry.value)
        }
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

        

    
                
