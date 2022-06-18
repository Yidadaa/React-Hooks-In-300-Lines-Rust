use std::any::Any;
use std::fmt::Debug;
use std::rc::Rc;

use crate::HookState;
use crate::{Effect, GuardList};

pub trait StaticClone: Clone + 'static {}
impl<T> StaticClone for T where T: Clone + 'static {}
pub trait Guard: PartialEq + Clone + Debug + 'static {}
impl<T> Guard for T where T: PartialEq + Clone + Debug + 'static {}

pub struct Ref<T: StaticClone> {
    id: (u32, usize),
    initial_value: T,
}

impl<T: StaticClone> Ref<T> {
    pub fn get(&self) -> T {
        let (bucket_id, ref_id) = self.id;

        if let Some(bucket) = HookState::get_bucket(&bucket_id) {
            let val = bucket.state_slots[ref_id].as_ref();
            let val = val.downcast_ref::<T>().unwrap().clone();

            return val;
        }

        self.initial_value.clone()
    }

    pub fn set(&self, value: T) {
        let (bucket_id, ref_id) = self.id;
        let bucket = HookState::get_bucket(&bucket_id).unwrap();
        bucket.state_slots[ref_id as usize] = Box::new(value);
    }
}

pub fn use_state<T: StaticClone>(initial_state: T) -> (T, Rc<impl Fn(T) -> ()>) {
    if HookState::get_current_bucket().is_some() {
        return use_reducer(
            |_: T, current_val: T| {
                // TODO: support function as current_val
                return current_val;
            },
            initial_state,
        );
    }

    panic!()
}

pub fn use_ref<T: StaticClone>(initial_state: T) -> Ref<T> {
    if let Some(bucket) = HookState::get_current_bucket() {
        let bucket_id = HookState::get_stack().last().unwrap().clone();
        let ref_id = bucket.next_state_slot_idx;
        use_state(initial_state.clone());

        return Ref {
            id: (bucket_id, ref_id),
            initial_value: initial_state,
        };
    }

    panic!()
}

pub fn use_reducer<T: StaticClone>(
    reducer: impl Fn(T, T) -> T,
    initial_value: T,
) -> (T, Rc<impl Fn(T) -> ()>) {
    if let Some(bucket) = HookState::get_current_bucket() {
        let index = bucket.next_state_slot_idx.clone();
        let id = HookState::last().unwrap().clone();

        if bucket.state_slots.len() <= index {
            bucket.state_slots.push(Box::new(initial_value.clone()));
        }

        let slot_value = bucket.state_slots[index]
            .downcast_ref::<T>()
            .unwrap_or(&initial_value)
            .clone();

        let slot = (
            slot_value.clone(),
            Rc::new(move |update_value: T| {
                print!("[Hook Reducer] use_reducer map {} to", id);
                let id = HookState::map_id(id);
                print!(" {}\n", id);
                let bucket = HookState::get_bucket(&id).unwrap();
                bucket.state_slots[index] =
                    Box::new(reducer(slot_value.clone(), update_value.clone()));
            }),
        );

        bucket.next_state_slot_idx += 1;

        slot
    } else {
        panic!("use_reducer() only valid inside an FC or custom hook.");
    }
}

pub fn guards_changed<T: Guard>(guards: &T, last_guards: &GuardList) -> bool {
    return last_guards.is_none() || {
        let last_guards = last_guards.as_ref().unwrap().downcast_ref::<T>().unwrap();
        println!("[Hook Guards] {:?} and {:?}", guards, last_guards);
        last_guards
    } != guards;
}

pub fn use_effect<T: Guard>(func: impl Fn() -> fn() + 'static, guards: T) {
    if let Some(bucket) = HookState::get_current_bucket() {
        if bucket.effects.len() <= bucket.next_effect_idx {
            bucket.effects.push((None, None));
        }

        let id = HookState::last().unwrap().clone();
        let index = bucket.next_effect_idx;
        let (_, last_guards) = &bucket.effects[index];

        if guards_changed(&guards, last_guards) {
            println!("[Hook Effect] {:?} guards changed", id);
            let effect_fn = move || {
                let bucket = HookState::get_bucket(&id).unwrap();
                if let Some(Some(cleanup)) = bucket.cleanups.get(index) {
                    cleanup();
                    bucket.cleanups[index] = None;
                }

                let ret = func();

                if bucket.cleanups.len() <= index {
                    bucket.cleanups.push(Some(ret));
                }

                None
            };

            let guards = Some(Box::new(guards) as Box<dyn Any>);
            let effect_fn = Some(Box::new(effect_fn) as Effect);

            bucket.effects[index] = (effect_fn, guards);
        }

        bucket.next_effect_idx += 1;
    } else {
        panic!()
    }
}

pub fn use_memo<T: Guard, R: StaticClone>(func: impl Fn() -> R, guards: T) -> R {
    if let Some(bucket) = HookState::get_current_bucket() {
        if bucket.memoizations.len() <= bucket.next_memoization_idx {
            bucket.memoizations.push((Box::new(()), None));
        }

        let index = bucket.next_memoization_idx;
        let (_, last_guards) = &bucket.memoizations[index];

        if guards_changed(&guards, last_guards) {
            let memo_value = func();
            bucket.memoizations[index] =
                (Box::new(memo_value), Some(Box::new(guards) as Box<dyn Any>));
        }

        let (memo, _) = &bucket.memoizations[index];
        let memo = memo.downcast_ref::<R>().unwrap();
        bucket.next_memoization_idx += 1;

        return memo.clone();
    }

    panic!()
}

pub fn use_callback<T: Guard, F: StaticClone>(func: F, guards: T) -> F {
    if HookState::get_current_bucket().is_some() {
        return use_memo(|| func.clone(), guards);
    }

    panic!()
}
