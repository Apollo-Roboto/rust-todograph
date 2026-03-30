#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused)]

use log::{debug, info};
use rust_firework_core::LOGGER;
use rust_firework_core::commands;
use rust_firework_core::editor::EditorEvent;
use rust_firework_core::{Editor, MindTask, MindTaskState, Point, TaskGraph};
use slint::Model;
use std::collections::HashSet;
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

    let childrens = editor
        .state
        .graph
        .iter_children_of_task(task_id)
        .map(|t| t.id as i32);

    let childrens = std::rc::Rc::new(slint::VecModel::from_iter(childrens)).into();

    let parent_index = editor
        .state
        .graph
        .tasks
        .iter()
        .position(|t| Some(t.id) == task.parent)
        .map_or(-1, |id| id as i32);

    let depends_on_indexes = editor
        .state
        .graph
        .tasks
        .iter()
        .enumerate()
        .filter(|(i, t)| task.depends_on.contains(&t.id))
        .map(|(i, _t)| i as i32);

    let depends_on_indexes =
        std::rc::Rc::new(slint::VecModel::from_iter(depends_on_indexes)).into();

    Ok(ui::MindTask {
        childrens,
        completion: editor.state.graph.calc_progress(task.id).unwrap_or(0.0),
        id: task.id as i32,
        selected: task.selected,
        blocked: editor.state.graph.is_task_blocked(task.id).unwrap_or(false),
        parent_id: task.parent.map_or(-1, |id| id as i32),
        parent_index,
        depends_on_indexes,
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
            let childrens = editor
                .state
                .graph
                .iter_children_of_task(task.id)
                .map(|t| t.id as i32);

            let childrens = std::rc::Rc::new(slint::VecModel::from_iter(childrens)).into();

            let parent_index = editor
                .state
                .graph
                .tasks
                .iter()
                .position(|t| Some(t.id) == task.parent)
                .map_or(-1, |id| id as i32);

            let depends_on_indexes = editor
                .state
                .graph
                .tasks
                .iter()
                .enumerate()
                .filter(|(i, t)| task.depends_on.contains(&t.id))
                .map(|(i, _t)| i as i32);

            let depends_on_indexes =
                std::rc::Rc::new(slint::VecModel::from_iter(depends_on_indexes)).into();

            ui::MindTask {
                childrens,
                completion: editor.state.graph.calc_progress(task.id).unwrap_or(0.0),
                id: task.id as i32,
                selected: task.selected,
                blocked: editor.state.graph.is_task_blocked(task.id).unwrap_or(false),
                depends_on_indexes,
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

    main_window.global::<ui::Func>().on_num_in_array(|n, arr| {
        let dbg_arr: Vec<_> = arr.iter().collect();
        debug!("num_in_array called with {n} {dbg_arr:?}");
        arr.iter().any(|i| i == n)
    });

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
    main_window.on_quit(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        main_window.window().hide();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_refresh_other(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let editor = editor_clone.lock().unwrap();

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
    main_window.on_open_file_picked(move |path| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut editor = editor_clone.lock().unwrap();

        // if no path, load new project
        match path.is_empty() {
            true => editor.clear_all(),
            false => editor.load(&path).unwrap(),
        }

        println!("editor tasks: {}", editor.state.graph.tasks.len());

        main_window.set_task_loading_state(ui::TaskLoadingState::Loading);
        main_window.set_history_past_count(editor.history.past().count() as i32);
        main_window.set_history_future_count(editor.history.future().count() as i32);
        main_window.set_history_limit(editor.history.limit() as i32);
        main_window.set_last_command(slint::SharedString::new());

        std::mem::drop(editor);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_save_file_picked(move |path| {
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

        let cmd = Box::new(commands::DeleteTaskCommand::new(task_id as u32));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
        main_window.invoke_refresh_other();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_create_task(move |title, x, y| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
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
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_create_task_with_parent(move |parent_id, title, x, y| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
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
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_change_state(move |task_id, state| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetTaskStateCommand::new(
            task_id as u32,
            state.into(),
        ));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_duplicate_selected(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::DuplicateSelectedCommand::new());
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_selected_move_dropped(move |x, y| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::MoveSelectedPositionCommand::new(Point { x, y }));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_add_dependency_to_task(move |task_id, depends_on_id| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let (Ok(task_id), Ok(depends_on_id)) = (task_id.try_into(), depends_on_id.try_into())
        else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::AddTaskDependencyCommand::new(
            task_id,
            depends_on_id,
        ));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_remove_dependency_from_task(move |task_id, depends_on_id| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let (Ok(task_id), Ok(depends_on_id)) = (task_id.try_into(), depends_on_id.try_into())
        else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::RemoveTaskDependencyCommand::new(
            task_id,
            depends_on_id,
        ));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_remove_all_dependencies_from_task(move |task_id| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let Ok(task_id) = task_id.try_into() else {
            return;
        };

        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::RemoveAllTaskDependencyCommand::new(task_id));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_set_parent_to_task(move |task_id, parent_id| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let (Ok(task_id), Ok(parent_id)) = (task_id.try_into(), parent_id.try_into()) else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetTaskParentCommand::new(
            task_id,
            Some(parent_id),
        ));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_unset_parent_from_task(move |task_id| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetTaskParentCommand::new(task_id as u32, None));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_rename_task(move |task_id, title| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetTaskTitleCommand::new(
            task_id as u32,
            title.into(),
        ));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_set_task_notes(move |task_id, notes| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetTaskNotesCommand::new(
            task_id as u32,
            notes.into(),
        ));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
    });

    let editor_clone = editor.clone();
    let main_window_weak = main_window.as_weak();
    main_window.on_set_active_task(move |task_id, keep_selection| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::SetActiveCommand::new(
            task_id as u32,
            keep_selection,
        ));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
        main_window.invoke_refresh_other();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_clear_active_task(move |keep_selected| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::ClearActiveCommand::new(keep_selected));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
        main_window.invoke_refresh_other();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_clear_selection(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();
        let cmd = Box::new(commands::ClearSelectionCommand::new());
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
        main_window.invoke_refresh_other();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_set_selection_from_box(move |x1, y1, x2, y2| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();

        let selection_box = (Point::new(x1, y1), Point::new(x2, y2));

        let tasks: HashSet<_> = editor
            .state
            .graph
            .tasks
            .iter()
            .filter(|t| t.pos.is_within(selection_box.0, selection_box.1))
            .map(|t| t.id)
            .collect();

        let cmd = Box::new(commands::SetSelectionCommand::new(tasks));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
        main_window.invoke_refresh_other();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_add_selection_from_box(move |x1, y1, x2, y2| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();

        let selection_box = (Point::new(x1, y1), Point::new(x2, y2));

        let tasks: HashSet<_> = editor
            .state
            .graph
            .tasks
            .iter()
            .filter(|t| t.pos.is_within(selection_box.0, selection_box.1))
            .map(|t| t.id)
            .collect();

        let cmd = Box::new(commands::AddToSelectionCommand::new(tasks));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
        main_window.invoke_refresh_other();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_remove_selection_from_box(move |x1, y1, x2, y2| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();

        let selection_box = (Point::new(x1, y1), Point::new(x2, y2));

        let tasks: HashSet<_> = editor
            .state
            .graph
            .tasks
            .iter()
            .filter(|t| t.pos.is_within(selection_box.0, selection_box.1))
            .map(|t| t.id)
            .collect();

        let cmd = Box::new(commands::RemoveFromSelectionCommand::new(tasks));
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
        main_window.invoke_refresh_other();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_select_all(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();

        let cmd = Box::new(commands::AddAllToSelectionCommand::new());
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
        main_window.invoke_refresh_other();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_delete_selected(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let mut editor = editor_clone.lock().unwrap();

        let cmd = Box::new(commands::DeleteSelectedCommand::new());
        editor.execute(cmd).unwrap();
        std::mem::drop(editor);
        main_window.invoke_refresh_tasks();
        main_window.invoke_refresh_other();
    });

    info!("Starting application");

    main_window.run().unwrap();

    info!("Bye bye!");
}
