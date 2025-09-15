#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};

// use rust_firework_core::commands::{
//     ClearTaskActiveCommand, Command, CreateTaskCommand, DeleteTaskCommand, SetTaskActiveCommand,
//     SetTaskParentCommand, SetTaskPositionCommand, SetTaskStateCommand, SetTaskTitleCommand,
// };
use rust_firework_core::commands;
use rust_firework_core::commands::Command;
use rust_firework_core::{Editor, MindTask, MindTaskState, Point, TaskGraph};
use slint::ComponentHandle;

mod ui {
    slint::include_modules!();
}

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

fn main() {
    let main_window = ui::AppWindow::new().unwrap();

    let editor = Arc::new(Mutex::new(Editor::default()));

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_refresh_other(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let editor = editor_clone.lock().unwrap();

        // get the tasks
        let tasks: Vec<ui::MindTask> = editor
            .state
            .graph
            .tasks
            .iter()
            .map(|t| ui::MindTask {
                id: t.id as i32,
                state: t.state.into(),
                title: t.title.clone().into(),
                x: t.pos.x,
                y: t.pos.y,
                parent: t.parent.map_or(-1, |id| id as i32),
                completion: editor.state.graph.calc_progress(t.id).unwrap_or(0.0),
            })
            .collect();
        let tasks_model = std::rc::Rc::new(slint::VecModel::from(tasks));

        // refresh the copied active task
        if let Some(active_task_id) = editor.state.active_task
            && let Some(active_task) = editor
                .state
                .graph
                .tasks
                .iter()
                .find(|t| t.id == active_task_id)
        {
            let ui_task = ui::MindTask {
                id: active_task.id as i32,
                state: active_task.state.into(),
                title: active_task.title.clone().into(),
                x: active_task.pos.x,
                y: active_task.pos.y,
                parent: active_task.parent.map_or(-1, |id| id as i32),
                completion: editor
                    .state
                    .graph
                    .calc_progress(active_task.id)
                    .unwrap_or(0.0),
            };
            main_window.set_active_task(ui_task);
            main_window.set_has_active_task(true);
        } else {
            main_window.set_has_active_task(false);
        }

        main_window.set_tasks(tasks_model.into());
        main_window.set_overall_progress(editor.state.graph.calc_progress_all().unwrap_or(0.0));
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_refresh_edges(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let editor = editor_clone.lock().unwrap();

        // get the positions of the edges
        let edges: Vec<ui::MindEdge> = editor
            .state
            .graph
            .get_all_edges()
            .iter()
            .map(|(from, to)| ui::MindEdge {
                from_x: from.x,
                from_y: from.y,
                to_x: to.x,
                to_y: to.y,
            })
            .collect();
        let edges_model = std::rc::Rc::new(slint::VecModel::from(edges));

        main_window.set_edges(edges_model.into());
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

        main_window.set_task_loading_state(ui::TaskLoadingState::Loading);

        editor.state.graph = TaskGraph::load(&path).unwrap();

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

        match editor.state.graph.save(&path) {
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

        if let Some(task) = editor
            .state
            .graph
            .tasks
            .iter()
            .find(|t| t.id == task_id as u32)
        {
            let cmd = Box::new(commands::DeleteTaskCommand::new(task.clone()));
            editor.execute(cmd).unwrap();
        }

        // avoid deadlock from the next invoke
        std::mem::drop(editor);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_create_task(move |title, x, y| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut editor = editor_clone.lock().unwrap();

        let id = editor.state.graph.generate_id();

        let task = MindTask {
            id,
            pos: Point { x, y },
            title: title.into(),
            creation_date: chrono::Utc::now(),
            ..Default::default()
        };

        let cmd = Box::new(commands::CreateTaskCommand::new(task));
        editor.execute(cmd).unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(editor);

        main_window.invoke_refresh_all();
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

        // avoid deadlock from the next invoke
        std::mem::drop(editor);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_task_moved(move |task_id, x, y| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut editor = editor_clone.lock().unwrap();

        let mut cmd = Box::new(commands::SetTaskPositionCommand::new(
            task_id as u32,
            Point { x, y },
        ));
        // TODO: this is called too often, I need to save only when it was dropped into the command history
        // for now I'll just not have the movement in the history at all
        // command_history.execute(cmd, &mut task_manager).unwrap();
        cmd.execute(&mut editor.state).unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(editor);

        main_window.invoke_refresh_edges();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_set_parent_to_task(move |task_id, parent_id| {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut editor = editor_clone.lock().unwrap();

        let Ok(task_id) = task_id.try_into() else {
            return;
        };
        let Ok(parent_id) = parent_id.try_into() else {
            return;
        };

        let cmd = Box::new(commands::SetTaskParentCommand::new(
            task_id,
            Some(parent_id),
        ));
        editor.execute(cmd).unwrap();

        // avoid deadlock from the next invoke
        std::mem::drop(editor);

        main_window.invoke_refresh_all();
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

        // avoid deadlock from the next invoke
        std::mem::drop(editor);

        main_window.invoke_refresh_all();
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

        // avoid deadlock from the next invoke
        std::mem::drop(editor);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_set_active_task(move |task_id| {
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
            .cloned()
        else {
            return;
        };

        let cmd = Box::new(commands::SetTaskActiveCommand::new(task_id as u32));
        editor.execute(cmd).unwrap();

        let ui_task = ui::MindTask {
            id: task.id as i32,
            state: task.state.into(),
            title: task.title.into(),
            x: task.pos.x,
            y: task.pos.y,
            parent: task.parent.map_or(-1, |id| id as i32),
            completion: editor.state.graph.calc_progress(task.id).unwrap_or(0.0),
        };

        main_window.set_active_task(ui_task);
        main_window.set_has_active_task(true);

        // avoid deadlock from the next invoke
        std::mem::drop(editor);

        main_window.invoke_refresh_all();
    });

    let main_window_weak = main_window.as_weak();
    let editor_clone = editor.clone();
    main_window.on_clear_active_task(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };

        let mut editor = editor_clone.lock().unwrap();

        let cmd = Box::new(commands::ClearTaskActiveCommand::new());
        editor.execute(cmd).unwrap();

        main_window.set_has_active_task(false);

        // avoid deadlock from the next invoke
        std::mem::drop(editor);

        main_window.invoke_refresh_all();
    });

    main_window.run().unwrap();
}
