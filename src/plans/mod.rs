use ::cache::LRUCache as LRUCache;
use std::try;

pub struct Plans {
    cache: LRUCache<String, String>;
};

impl Plans {
    pub fn new() -> Plans {
        Plans { cache: LRUCache::new() }
    }
    
    pub fn fetch(&self, &username: str) -> Result<&str, &str> {
        let item = self.cache.get(username);
        match item {
            Some(plan) => plan,
            None => self.new_fetch(username)
        }
    }
    
    fn new_fetch(&self, &username: str) -> Result<&str, &str> {
        let userpath = try!(match plan::get_userpath(&username));
        let plan = try!(plan::get_userplan(&userpath))
            if plan.len() < ::CACHE_ITEM_SIZE {
                self.cache.insert(plan);
            }
        plan
    }
    
    pub fn evict(&self, &username: str) {
        self.cache.remove(username);
    }
    
    pub fn expire(&self) {
        let when = Tm::now_utc() - Duration::seconds(::CACHE_EXPIRATION_TIMEOUT);
        self.cache.evict_older_than(when);
    }
};
        
        
