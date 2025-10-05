#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused)]

use log::info;
use rust_firework_core::LOGGER;
use rust_firework_core::commands;
use rust_firework_core::commands::Command;
use rust_firework_core::editor::EditorEvent;
use rust_firework_core::{Editor, MindTask, MindTaskState, Point, TaskGraph};
use std::sync::{Arc, Mutex, MutexGuard};

use slint::ComponentHandle;

mod ui {
    slint::include_modules!();
}
mod widgets;

const APPLICATION_VERSION: &str = env!("CARGO_PKG_VERSION");
const APPLICATION_IS_RELEASE: bool = !cfg!(debug_assertions);

impl From<MindTaskState> for ui::MindTaskState {
    fn from(value: MindTaskState) -> Self {
        match value {
            MindTaskState::Todo => ui::MindTaskState::Todo,
            MindTaskState::Doing => ui::MindTaskState::Doing,
            MindTaskState::Done => ui::MindTaskState::Done,
        }
    }
}

impl From<ui::MindTaskState> for MindTaskState {
    fn from(value: ui::MindTaskState) -> Self {
        match value {
            ui::MindTaskState::Todo => MindTaskState::Todo,
            ui::MindTaskState::Doing => MindTaskState::Doing,
            ui::MindTaskState::Done => MindTaskState::Done,
        }
    }
}
fn editor_task_to_ui_task(task_id: u32, editor: &Editor) -> Result<ui::MindTask, ()> {
    let Some(task) = editor.state.graph.tasks.iter().find(|t| t.id == task_id) else {
        return Err(());
    };

    let childrens = std::rc::Rc::new(slint::VecModel::from_iter(
        task.childrens.iter().map(|id| *id as i32),
    ))
    .into();

    let parent_index = editor
        .state
        .graph
        .tasks
        .iter()
        .position(|t| Some(t.id) == task.parent)
        .map_or(-1, |id| id as i32);

    Ok(ui::MindTask {
        childrens,
        completion: editor.state.graph.calc_progress(task.id).unwrap_or(0.0),
        id: task.id as i32,
        parent_id: task.parent.map_or(-1, |id| id as i32),
        parent_index,
        state: task.state.into(),
        title: task.title.clone().into(),
        notes: task.notes.clone().into(),
        x: task.pos.x,
        y: task.pos.y,
    })
}

fn all_editor_task_to_ui_task(editor: &Editor) -> Vec<ui::MindTask> {
    editor
        .state
        .graph
        .tasks
        .iter()
        .map(|task| {
            let childrens = std::rc::Rc::new(slint::VecModel::from_iter(
                task.childrens.iter().map(|id| *id as i32),
            ))
            .into();

            let parent_index = editor
                .state
                .graph
                .tasks
                .iter()
                .position(|t| Some(t.id) == task.parent)
                .map_or(-1, |id| id as i32);

            ui::MindTask {
                childrens,
                completion: editor.state.graph.calc_progress(task.id).unwrap_or(0.0),
                id: task.id as i32,
                parent_id: task.parent.map_or(-1, |id| id as i32),
                parent_index,
                state: task.state.into(),
                title: task.title.clone().into(),
                notes: task.notes.clone().into(),
                x: task.pos.x,
                y: task.pos.y,
            }
        })
        .collect()
}

fn main() {
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    info!(
        "Application version: {} ({})",
        crate::APPLICATION_VERSION,
        if crate::APPLICATION_IS_RELEASE {
            "Release"
        } else {
            "Development"
        }
    );

    let main_window = ui::AppWindow::new().unwrap();

    main_window.set_app_version(APPLICATION_VERSION.into());
    main_window.set_is_release(APPLICATION_IS_RELEASE);

    widgets::setup_file_dialog_button(main_window.global::<ui::FileDialogButtonGlobal>());
    widgets::setup_hyperlink(main_window.global::<ui::HyperlinkGlobal>());

    let mut editor = Editor::default();
    main_window.set_history_limit(editor.history.limit() as i32);

    let main_window_weak = main_window.as_weak();
    editor.on_event(move |editor, event| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        match event {
            EditorEvent::CommandSuccess => {
                main_window.set_history_past_count(editor.history.past().count() as i32);
                main_window.set_history_future_count(editor.history.future().count() as i32);
                main_window.set_history_limit(editor.history.limit() as i32);
                main_window.set_last_command(
                    editor
                        .history
                        .last()
                        .map_or(slint::SharedString::new(), |cmd| {
                            slint::SharedString::from(cmd.to_string())
                        }),
                );
            }
            EditorEvent::CommandFailed(_e) => {
                main_window.set_history_past_count(editor.history.past().count() as i32);
                main_window.set_history_future_count(editor.history.future().count() as i32);
                main_window.set_history_limit(editor.history.limit() as i32);
            }
            _ => todo!(),
        }
    });

    let editor = Arc::new(Mutex::new(editor));

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_refresh_other(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let editor = editor_clone.lock().unwrap();

        // refresh the copied active task
        if let Some(task_id) = editor.state.active_task
            && let Some(task_index) = editor
                .state
                .graph
                .tasks
                .iter()
                .position(|t| t.id == task_id)
        {
            main_window.set_active_task_index(task_index as i32);
        } else {
            main_window.set_active_task_index(-1);
        }
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_refresh_tasks(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let editor = editor_clone.lock().unwrap();

        // get the tasks
        let tasks = all_editor_task_to_ui_task(&editor);
        let tasks_model = std::rc::Rc::new(slint::VecModel::from(tasks));

        main_window.set_tasks(tasks_model.into());
        main_window.set_overall_progress(editor.state.graph.calc_progress_all().unwrap_or(0.0));
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_load(move |path| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut editor = editor_clone.lock().unwrap();

        // no commands in the history is relevent after loading a project
        editor.history.clear();
        editor.state.active_task = None;

        main_window.set_task_loading_state(ui::TaskLoadingState::Loading);
        main_window.set_history_past_count(editor.history.past().count() as i32);
        main_window.set_history_future_count(editor.history.future().count() as i32);
        main_window.set_history_limit(editor.history.limit() as i32);
        main_window.set_last_command(slint::SharedString::new());

        editor.load(&path).unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(editor);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_save(move |path| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let editor = editor_clone.lock().unwrap();

        match editor.save(&path) {
            Ok(_) => {
                main_window.set_task_saving_state(ui::TaskSavingState::None);
            }
            Err(e) => {
                main_window.set_task_saving_state(ui::TaskSavingState::Error);
                main_window.set_task_saving_error_message(e.into());
            }
        };
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_undo(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut editor = editor_clone.lock().unwrap();

        editor.undo().unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(editor);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_redo(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();

        editor.redo().unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(editor);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_delete_task(move |task_id| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let Some(task) = editor
            .state
            .graph
            .tasks
            .iter()
            .find(|t| t.id == task_id as u32)
        else {
            return;
        };
        let cmd = Box::new(commands::DeleteTaskCommand::new(task.clone()));
        editor.execute(cmd).unwrap();

        if let Some(active_task_index) = editor
            .state
            .graph
            .tasks
            .iter()
            .position(|t| Some(t.id) == editor.state.active_task)
        {
            main_window.set_active_task_index(active_task_index as i32);
        } else {
            main_window.set_active_task_index(-1);
        }

        handle_task_change(&main_window_weak, editor);
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_create_task(move |title, x, y| {
        let mut editor = editor_clone.lock().unwrap();
        let task = MindTask {
            id: editor.state.graph.generate_id(),
            pos: Point { x, y },
            title: title.into(),
            creation_date: chrono::Utc::now(),
            ..Default::default()
        };
        let cmd = Box::new(commands::CreateTaskCommand::new(task));
        editor.execute(cmd).unwrap();
        handle_task_change(&main_window_weak, editor);
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_create_task_with_parent(move |parent_id, title, x, y| {
        let mut editor = editor_clone.lock().unwrap();
        let task = MindTask {
            id: editor.state.graph.generate_id(),
            pos: Point { x, y },
            title: title.into(),
            parent: Some(parent_id as u32),
            creation_date: chrono::Utc::now(),
            ..Default::default()
        };
        let cmd = Box::new(commands::CreateTaskCommand::new(task));
        editor.execute(cmd).unwrap();
        handle_task_change(&main_window_weak, editor);
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_change_state(move |task_id, state| {
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetTaskStateCommand::new(
            task_id as u32,
            state.into(),
        ));
        editor.execute(cmd).unwrap();
        handle_task_change(&main_window_weak, editor);
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_duplicate_task(move |task_id, _x, _y| {
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::DuplicateTaskCommand::new(task_id as u32));
        editor.execute(cmd).unwrap();
        handle_task_change(&main_window_weak, editor);
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_task_moved_dropped(move |task_id, x, y| {
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetTaskPositionCommand::new(
            task_id as u32,
            Point { x, y },
        ));
        editor.execute(cmd).unwrap();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_set_parent_to_task(move |task_id, parent_id| {
        let (Ok(task_id), Ok(parent_id)) = (task_id.try_into(), parent_id.try_into()) else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetTaskParentCommand::new(
            task_id,
            Some(parent_id),
        ));
        editor.execute(cmd).unwrap();
        handle_task_change(&main_window_weak, editor);
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_unset_parent_from_task(move |task_id| {
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetTaskParentCommand::new(task_id as u32, None));
        editor.execute(cmd).unwrap();
        handle_task_change(&main_window_weak, editor);
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_rename_task(move |task_id, title| {
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetTaskTitleCommand::new(
            task_id as u32,
            title.into(),
        ));
        editor.execute(cmd).unwrap();
        handle_task_change(&main_window_weak, editor);
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_set_task_notes(move |task_id, notes| {
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetTaskNotesCommand::new(
            task_id as u32,
            notes.into(),
        ));
        editor.execute(cmd).unwrap();
        handle_task_change(&main_window_weak, editor);
    });

    let editor_clone = editor.clone();
    let main_window_weak = main_window.as_weak();
    main_window.on_set_active_task(move |task_id| {
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetTaskActiveCommand::new(task_id as u32));
        editor.execute(cmd).unwrap();
        handle_active_task_change(&main_window_weak, editor);
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_clear_active_task(move || {
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::ClearTaskActiveCommand::new());
        editor.execute(cmd).unwrap();
        handle_active_task_change(&main_window_weak, editor);
    });

    info!("Starting application");

    main_window.run().unwrap();

    info!("Bye bye!");
}

fn handle_active_task_change(
    main_window_weak: &slint::Weak<ui::AppWindow>,
    editor: MutexGuard<Editor>,
) {
    let Some(main_window) = main_window_weak.upgrade() else {
        return;
    };
    let Some(task_id) = editor.state.active_task else {
        main_window.set_active_task_index(-1);
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
        return;
    };

    let Some(task_index) = editor
        .state
        .graph
        .tasks
        .iter()
        .position(|t| t.id == task_id)
    else {
        main_window.set_active_task_index(-1);
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
        return;
    };

    std::mem::drop(editor);

    main_window.set_active_task_index(task_index as i32);
    main_window.invoke_refresh_tasks();
}

fn handle_task_change(main_window_weak: &slint::Weak<ui::AppWindow>, editor: MutexGuard<Editor>) {
    let Some(main_window) = main_window_weak.upgrade() else {
        return;
    };
    std::mem::drop(editor);
    main_window.invoke_refresh_tasks();
}
