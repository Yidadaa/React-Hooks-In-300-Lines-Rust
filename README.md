# 300-Lines-React-Hooks-In-Rust
A minimal implementation of react hooks in Rust.

# 300 行 Rust 代码实现 React Hooks

最近在用 Rust 写一个 Native GUI 框架的玩具，调研了一圈状态管理模式后，觉得 React Hooks 的设计蛮不错，在 Github 搜了一圈，想找找可以参考的实现，发现 Web 框架 [Yew](https://yew.rs/) 的实现还不错，此外 Rust Native GUI 框架中有个叫 [Dioxus](https://github.com/DioxusLabs/dioxus) 也是用的 React 这套范式，然而他们都有点怪怪的，比如 Dioxus 在实现组件的时候，官方的 Example：

```Rust
fn app(cx: Scope) -> Element {
    let mut count = use_state(&cx, || 0);

    cx.render(rsx! {
        h1 { "High-Five counter: {count}" }
        button { onclick: move |_| count += 1, "Up high!" }
        button { onclick: move |_| count -= 1, "Down low!" }
    })
}
```

有两个槽点：
1. 需要把一个 `cx` 传来传去；
2. 组件的结构用 `rsx!` 宏来定义。

再来看看 Yew 的示例：

```Rust
use yew::{Callback, function_component, html, use_state};

#[function_component(StateComp)]
fn state() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        Callback::from(move |_| counter.set(*counter + 1))
    };


    html! {
        <div>
            <button {onclick}>{ "Increment value" }</button>
            <p>
                <b>{ "Current value: " }</b>
                { *counter }
            </p>
        </div>
    }
}
```

很好，没有”多余“的 `cx` 变量了，转而用宏来声明组件，不过新的槽点出现了：我定义的函数式组件明明有个函数名了，为啥还要给宏传入参数来作为组件名呢？而且组件结构依然是用 `html!` 宏来声明。

不过问题不大，先抛开定义组件的范式不谈，其实很多 Rust 的 GUI 框架都对 html / jsx 的那套写法念念不忘，而搞出来 `rsx!` / `jsx!` 这些过程宏来模拟 html / jsx 的写法，虽然形似了，但写起来却无比难受，因为不是 Rust 原生语法，写所谓的 `rsx!` / `jsx!` 代码是没有任何补全可言的，而且 Rust 宏的调试也比较困难。

然后来说说组件的定义，Dioxus 的组件是一个真正的函数，而 Yew 的组件是个长得像函数的结构体，上面的例子中，`function_component!` 宏会把 `state()` 转换成 `StateComp` 结构体，然后 `html!` 宏将 html 代码转换成多个和 `StateComp` 差不多的结构体嵌套成的树状结构来描述 UI。 