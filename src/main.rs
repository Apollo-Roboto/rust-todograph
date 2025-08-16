#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint::SharedString;

mod models;

slint::include_modules!();

fn main() {
    let main_window = AppWindow::new().unwrap();

    let main_window_weak = main_window.as_weak();
    main_window.on_load(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let tasks: Vec<MindTask> = vec![
            MindTask {
                state: MindTaskState::Todo,
                title: SharedString::from("I have a thing to do"),
                x: 0.,
                y: -100.,
            },
            MindTask {
                state: MindTaskState::Doing,
                title: SharedString::from("I'm doing great"),
                x: 0.,
                y: -150.,
            },
            MindTask {
                state: MindTaskState::Done,
                title: SharedString::from("Hell yeah"),
                x: 0.,
                y: -200.,
            },
        ];

        let tasks_model = std::rc::Rc::new(slint::VecModel::from(tasks));

        main_window.set_tasks(tasks_model.into());
    });

    main_window.run().unwrap();
}
