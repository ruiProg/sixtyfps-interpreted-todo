use sixtyfps::{Model, VecModel};
use sixtyfps_interpreter::{print_diagnostics, ComponentCompiler, SharedString, Struct, Value};
use std::rc::Rc;

fn create_todo_item(title: &str, checked: bool) -> Value {
    [
        ("title".into(), SharedString::from(title).into()),
        ("checked".into(), checked.into()),
    ]
    .into_iter()
    .collect::<Struct>()
    .into()
}

struct TodoItem(String, bool);

impl TryFrom<Value> for TodoItem {
    type Error = &'static str;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Struct(todo_item) => {
                match (todo_item.get_field("title"), todo_item.get_field("checked")) {
                    (Some(Value::String(title)), Some(Value::Bool(checked))) => {
                        Ok(TodoItem(title.to_string(), *checked))
                    }
                    _ => Err("Struct does not have expected fields".into()),
                }
            }
            _ => Err("Not a struct".into()),
        }
    }
}

fn main() {
    let todo_model = Rc::new(VecModel::<Value>::from(vec![
        create_todo_item("Implement the .60 file", true),
        create_todo_item("Do the Rust part", true),
        create_todo_item("Make the C++ code", false),
        create_todo_item("Write some JavaScript code", false),
        create_todo_item("Test the application", false),
        create_todo_item("Ship to customer", false),
        create_todo_item("???", false),
        create_todo_item("Profit", false),
    ]));

    let mut compiler = ComponentCompiler::default();

    let definition = spin_on::spin_on(compiler.build_from_path("ui/todo.60"));
    print_diagnostics(&compiler.diagnostics());
    if let Some(definition) = definition {
        let instance = definition.create();

        instance
            .set_callback("todo-added", {
                let todo_model = todo_model.clone();
                move |args| {
                    if !args.is_empty() {
                        if let Value::String(text) = &args[0] {
                            todo_model.push(create_todo_item(text.as_str(), false))
                        };
                    }
                    Value::Void
                }
            })
            .unwrap();

        instance
            .set_callback("remove-done", {
                let todo_model = todo_model.clone();
                move |_| {
                    let mut offset = 0;
                    for i in 0..todo_model.row_count() {
                        let todo_item = TodoItem::try_from(todo_model.row_data(i - offset));
                        if let Ok(TodoItem(_, checked)) = todo_item {
                            if checked {
                                todo_model.remove(i - offset);
                                offset += 1;
                            }
                        }
                    }
                    Value::Void
                }
            })
            .unwrap();

        instance
            .set_property("todo-model", Value::Model(todo_model))
            .unwrap();

        instance.run();
    }
}
