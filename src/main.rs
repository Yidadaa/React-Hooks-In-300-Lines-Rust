#[macro_use]
extern crate lazy_mut;

mod functional;

use functional::*;

fn comp_a(id: u32) {
    HookState::before_run(id);

    let (count, set_count) = use_state(0);
    let (number, set_number) = use_state(0);

    use_effect(
        move || {
            println!(
                "[Hook Effect] comp_a() setting {} to {} after count changes.",
                number,
                number + 2
            );
            set_number(number + 2);

            || {}
        },
        count,
    );

    let sum = use_memo(move || count + number, (count, number));
    let print_sum = use_callback(
        move || {
            println!("[Hook Callback] comp_a() sum: {}", sum);
        },
        (count, number),
    );

    println!("[Hook Comp] comp_a() sum: {}", sum);
    print_sum();

    set_count(count + 1);

    let ref_val = use_ref(0);
    ref_val.set(ref_val.get() + 2);

    println!("[Hook Comp] comp_a() ref_val: {}", ref_val.get());

    HookState::after_run(id);
}

fn main() {
    HookState::init();
    comp_a(1);
    comp_a(1);
    comp_a(1);
}
