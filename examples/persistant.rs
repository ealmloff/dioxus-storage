use dioxus::prelude::*;
use dioxus_storage::use_persistent;

fn main() {
    dioxus_web::launch(app)
}

fn app(cx: Scope) -> Element {
    let num = use_persistent(cx, "count", || 0);
    cx.render(rsx! {
        div {
            button {
                onclick: move |_| {
                    num.modify(|num| *num += 1);
                },
                "Increment"
            }
            div {
                "{*num.read()}"
            }
        }
    })
}
