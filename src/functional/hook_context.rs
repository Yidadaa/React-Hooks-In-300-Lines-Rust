use std::cell::RefCell;
use std::rc::Rc;
use std::{any::Any, collections::HashMap};

use lazy_mut::LazyMut;

pub type GuardList = Option<Rc<dyn Any>>;
pub type Effect = Rc<dyn Fn() -> Option<fn()>>;

#[derive(Default)]
pub struct Bucket {
    pub next_state_slot_idx: usize,
    pub state_slots: Vec<Rc<RefCell<dyn Any>>>,

    pub next_effect_idx: usize,
    pub effects: Vec<(Option<Effect>, GuardList)>,
    pub cleanups: Vec<Option<fn()>>,

    pub next_memoization_idx: usize,
    pub memoizations: Vec<(Rc<dyn Any>, GuardList)>,
}

lazy_mut! {
    static mut BUCKETS: HashMap<u32, Bucket> = HashMap::new();
    static mut STATEMAP: HashMap<u32, u32> = HashMap::new();
    static mut STACK: Vec<u32> = vec![];
    static mut FUNCID: u32 = 0;
}

pub struct HookState;

impl HookState {
    pub fn init() {
        unsafe {
            HookState::try_init_static(&mut BUCKETS);
            HookState::try_init_static(&mut STACK);
            HookState::try_init_static(&mut FUNCID);
            HookState::try_init_static(&mut STATEMAP);
        }
    }

    pub fn try_init_static<T>(item: &mut LazyMut<T>) {
        if !item.is_initialized() {
            item.init();
        }
    }

    pub fn get_buckets() -> &'static HashMap<u32, Bucket> {
        return unsafe { &BUCKETS };
    }

    pub fn get_stack() -> &'static Vec<u32> {
        return unsafe { &STACK };
    }

    pub fn get_current_bucket() -> Option<&'static mut Bucket> {
        unsafe {
            if let Some(id) = STACK.last() {
                if !BUCKETS.contains_key(id) {
                    println!("[Hook Create Bucket] create Bucket {:?}", id);
                }
                return Some(BUCKETS.entry(*id).or_insert(Bucket::default()));
            }
        }

        None
    }

    pub fn get_bucket(id: &u32) -> Option<&'static mut Bucket> {
        unsafe {
            return BUCKETS.get_mut(id);
        }
    }

    pub fn create_comp_id() -> u32 {
        unsafe {
            *FUNCID = (1 + *FUNCID) % u32::max_value();

            println!("[Hook Create Comp] create comp id: {}", *FUNCID);

            *FUNCID
        }
    }

    pub fn reset_comp_id() {
        unsafe {
            *FUNCID = 0;
        }
    }

    pub fn push(id: u32) {
        unsafe {
            STACK.push(id);
        }
    }

    pub fn pop() {
        unsafe {
            STACK.pop();
        }
    }

    pub fn last() -> Option<&'static u32> {
        unsafe {
            return STACK.last();
        }
    }

    pub fn reset(id: u32) {
        HookState::push(id);
        let bucket = HookState::get_current_bucket().unwrap();
        for cleanup in &bucket.cleanups {
            if let Some(cleanup) = cleanup {
                cleanup();
            }
        }

        HookState::reset_bucket(id);
        HookState::pop();
    }

    pub fn run_effects_of(id: u32) {
        let bucket = HookState::get_bucket(&id).unwrap();
        for (effect, _) in bucket.effects.iter_mut() {
            if let Some(effect) = effect {
                effect();
            }
            *effect = None;
        }
    }

    pub fn reset_bucket_idx(id: u32) {
        let bucket = HookState::get_bucket(&id).unwrap();
        bucket.next_state_slot_idx = 0;
        bucket.next_effect_idx = 0;
        bucket.next_memoization_idx = 0;
    }

    pub fn reset_bucket(id: u32) {
        unsafe {
            BUCKETS.remove(&id);
        }
    }

    pub fn before_run(id: u32) {
        HookState::push(id); // push current context
        HookState::get_current_bucket(); // try to init bucket
        HookState::reset_bucket_idx(id);
    }

    pub fn after_run(id: u32) {
        HookState::run_effects_of(id); // run effects of component
        HookState::pop(); // pop current context
    }

    pub fn map_state(old_id: u32, new_id: u32) {
        unsafe {
            STATEMAP.insert(new_id, old_id);
        }
    }

    pub fn commit_map_state() {
        unsafe {
            println!(
                "[Hook Commit] commit map state old buckets: {:?}",
                BUCKETS.keys()
            );
            let mut new_buckets = HashMap::<u32, Bucket>::new();
            for (new_id, old_id) in STATEMAP.iter() {
                if let Some(old_bucket) = BUCKETS.remove(old_id) {
                    new_buckets.insert(*new_id, old_bucket);
                    println!("[Hook Commit] old {:?} -> new {:?}", old_id, new_id);
                }
            }
            println!(
                "[Hook Commit] commit map state new buckets: {:?}",
                new_buckets.keys()
            );
            BUCKETS.extend(new_buckets);
        }
    }

    pub fn clear_map_state() {
        unsafe {
            STATEMAP.clear();
        }
    }

    pub fn map_id(id: u32) -> u32 {
        unsafe {
            if let Some(id) = STATEMAP.get(&id) {
                return *id;
            }
        }

        id
    }
}
