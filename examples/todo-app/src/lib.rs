use react_rs_core::{create_signal, ReadSignal, WriteSignal};
use react_rs_elements::html::*;
use react_rs_elements::node::IntoNode;

use react_rs_elements::Element;

#[derive(Clone)]
pub struct Todo {
    id: u32,
    text: String,
    completed: bool,
}

pub fn todo_app() -> Element {
    let (todos, set_todos) = create_signal(vec![
        Todo {
            id: 1,
            text: "Learn react.rs".to_string(),
            completed: false,
        },
        Todo {
            id: 2,
            text: "Build something awesome".to_string(),
            completed: false,
        },
        Todo {
            id: 3,
            text: "Ship it!".to_string(),
            completed: false,
        },
    ]);

    let (next_id, set_next_id) = create_signal(4u32);

    div()
        .class("todo-app")
        .child(h1().text("Todo App"))
        .child(todo_input(set_todos.clone(), next_id.clone(), set_next_id))
        .child(todo_list(todos.clone(), set_todos.clone()))
        .child(todo_stats(todos))
}

fn todo_input(
    set_todos: WriteSignal<Vec<Todo>>,
    next_id: ReadSignal<u32>,
    set_next_id: WriteSignal<u32>,
) -> Element {
    div()
        .class("todo-input")
        .child(input().type_("text").placeholder("What needs to be done?"))
        .child(button().text("Add").on_click(move |_| {
            let id = next_id.get();
            set_next_id.update(|n| *n += 1);
            set_todos.update(|todos| {
                todos.push(Todo {
                    id,
                    text: format!("New todo #{}", id),
                    completed: false,
                });
            });
        }))
}

fn todo_list(todos: ReadSignal<Vec<Todo>>, set_todos: WriteSignal<Vec<Todo>>) -> Element {
    let items: Vec<Element> = todos
        .get()
        .iter()
        .map(|todo| todo_item(todo.clone(), set_todos.clone()))
        .collect();

    ul().class("todo-list").children(items)
}

fn todo_item(todo: Todo, set_todos: WriteSignal<Vec<Todo>>) -> Element {
    let id = todo.id;
    let toggle_todos = set_todos.clone();

    li().class(if todo.completed {
        "completed"
    } else {
        "pending"
    })
    .child(
        input()
            .type_("checkbox")
            .disabled(todo.completed)
            .on_change(move |_| {
                toggle_todos.update(|todos| {
                    if let Some(t) = todos.iter_mut().find(|t| t.id == id) {
                        t.completed = !t.completed;
                    }
                });
            }),
    )
    .child(span().text(&todo.text))
    .child(button().text("×").on_click(move |_| {
        set_todos.update(|todos| {
            todos.retain(|t| t.id != id);
        });
    }))
}

fn todo_stats(todos: ReadSignal<Vec<Todo>>) -> Element {
    let total = todos.get().len();
    let completed = todos.get().iter().filter(|t| t.completed).count();
    let remaining = total - completed;

    div()
        .class("todo-stats")
        .child(span().text(format!("{} items left", remaining)))
        .child(span().text(format!(" | {} completed", completed)))
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main() {
    react_rs_dom::mount_to_body(todo_app().into_node());
}

#[cfg(not(target_arch = "wasm32"))]
pub fn main() {
    let output = react_rs_dom::render_to_string(&todo_app().into_node());
    println!("{}", output.html);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_app_renders() {
        let output = react_rs_dom::render_to_string(&todo_app().into_node());
        assert!(output.html.contains("Todo App"));
        assert!(output.html.contains("Learn react.rs"));
        assert!(output.html.contains("items left"));
    }
}
