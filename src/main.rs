#[macro_use]
extern crate lazy_mut;

mod functional;

use functional::*;

#[derive(Clone, Debug)]
struct ComplexState {
    pub value: i32,
    pub name: String,
}

#[derive(Clone)]
enum Action {
    Increment(i32),
    Decrement(i32),
    SetName(String),
}

fn comp_state_reducer(state: ComplexState, action: Action) -> ComplexState {
    let mut state = state;

    match action {
        Action::Increment(val) => {
            state.value += val;
        }
        Action::Decrement(val) => {
            state.value -= val;
        }
        Action::SetName(name) => {
            state.name = name;
        }
    }

    state
}

#[derive(Clone, Debug)]
struct ThemeContext {
    pub theme: String,
}

fn comp_a(id: u32) {
    HookState::before_run(id);

    let (count, set_count) = use_state(0);
    let (number, set_number) = use_state(0);

    use_effect(
        move || {
            println!(
                "[Comp Effect] comp_a() setting {} to {} after count changes.",
                number,
                number + 2
            );
            set_number(number + 2);

            || {}
        },
        count,
    );

    use_effect(
        move || {
            println!("[Comp Effect] comp_a() mounted.");

            || {
                println!("[Comp Effect] comp_a() unmounted.");
            }
        },
        (),
    );

    let sum = use_memo(
        move || {
            println!("[Comp Memo] comp_a() memoizing sum re-run.");
            count + number
        },
        (count, number),
    );
    let print_sum = use_callback(
        move || {
            println!("[Comp Callback] comp_a() sum: {}", sum);
        },
        (count, number),
    );

    println!("[Comp Comp] comp_a() sum: {}", sum);
    print_sum();

    set_count(count + 1);

    let ref_val = use_ref(0);
    ref_val.set(ref_val.get() + 2);
    println!("[Comp Ref] comp_a() ref_val: {}", ref_val.get());

    let (state, dispatch) = use_reducer(
        comp_state_reducer,
        ComplexState {
            value: 0,
            name: "name".to_string(),
        },
    );

    dispatch(Action::Increment(1));
    dispatch(Action::Decrement(2));
    dispatch(Action::SetName(format!("{}_a", state.value)));

    println!("[Comp] comp_a() state: {:?}", state);

    Context::new(ThemeContext {
        theme: "light".to_string(),
    })
    .provide(sub_comp);

    HookState::after_run(id);
}

fn sub_comp() {
    let ctx = use_context::<ThemeContext>();

    println!("[Sub Comp] sub_comp() ctx: {:?}", ctx.theme);
}

fn main() {
    HookState::init();
    let id = HookState::create_comp_id();
    comp_a(id);
    comp_a(id);
    comp_a(id);
    HookState::reset(id);
}
