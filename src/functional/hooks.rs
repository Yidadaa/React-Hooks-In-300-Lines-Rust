use std::any::Any;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

use crate::HookState;
use crate::{Effect, GuardList};

pub trait StaticClone: Clone + 'static {}
impl<T> StaticClone for T where T: Clone + 'static {}

pub trait Guard: PartialEq + Clone + Debug + 'static {}
impl<T> Guard for T where T: PartialEq + Clone + Debug + 'static {}

pub struct HookRef<T: StaticClone> {
    initial_value: T,
    ref_cell: Rc<RefCell<dyn Any>>,
}

impl<T: StaticClone> HookRef<T> {
    pub fn get(&self) -> T {
        return self
            .ref_cell
            .borrow()
            .downcast_ref::<T>()
            .unwrap_or(&self.initial_value.clone())
            .clone();
    }

    pub fn set(&self, value: T) {
        self.ref_cell
            .borrow_mut()
            .downcast_mut::<T>()
            .unwrap()
            .clone_from(&value);
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

pub fn use_ref<T: StaticClone>(initial_state: T) -> HookRef<T> {
    if let Some(bucket) = HookState::get_current_bucket() {
        use_state(initial_state.clone());
        let ref_ptr = Rc::clone(bucket.state_slots.last().unwrap());

        return HookRef {
            initial_value: initial_state,
            ref_cell: ref_ptr,
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
        let value = Rc::new(RefCell::new(initial_value));

        if bucket.state_slots.len() <= index {
            bucket.state_slots.push(value);
        }

        let slot_value = bucket.state_slots[index].clone();

        let slot = (
            slot_value
                .clone()
                .borrow()
                .downcast_ref::<T>()
                .unwrap()
                .clone(),
            Rc::new(move |update_value: T| {
                let slot_ptr = Rc::clone(&slot_value);
                let old_value = slot_ptr.borrow().downcast_ref::<T>().unwrap().clone();

                let new_value = reducer(old_value.clone(), update_value);

                *slot_ptr.borrow_mut().downcast_mut::<T>().unwrap() = new_value;
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

            let guards = Some(Rc::new(guards) as Rc<dyn Any>);
            let effect_fn = Some(Rc::new(effect_fn) as Effect);

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
            bucket.memoizations.push((Rc::new(()), None));
        }

        let index = bucket.next_memoization_idx;
        let (_, last_guards) = &bucket.memoizations[index];

        if guards_changed(&guards, last_guards) {
            let memo_value = func();
            bucket.memoizations[index] =
                (Rc::new(memo_value), Some(Rc::new(guards) as Rc<dyn Any>));
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
