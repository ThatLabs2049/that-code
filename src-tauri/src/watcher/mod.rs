use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::{AppHandle, Manager};

use crate::db::DbState;
use crate::rag;
use crate::settings;

pub struct WatcherState {
    last_run: Mutex<Option<Instant>>,
}

impl WatcherState {
    pub fn new() -> Self {
        Self {
            last_run: Mutex::new(None),
        }
    }
}

pub fn start_workspace_watcher(app: AppHandle) {
    std::thread::spawn(move || {
        if let Err(err) = watch_loop(app) {
            eprintln!("workspace watcher stopped: {err}");
        }
    });
}

fn watch_loop(app: AppHandle) -> Result<(), String> {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

    let db_state = app.state::<DbState>();

    let (settings, workspace) = {
        let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
        let settings = settings::load(&conn).map_err(|e| e.to_string())?;
        let workspace = settings
            .workspace_path
            .clone()
            .ok_or_else(|| "no workspace".to_string())?;
        (settings, workspace)
    };

    if !settings.rag_enabled || !settings.rag_auto_index {
        return Ok(());
    }

    let workspace_path = Path::new(&workspace);
    if !workspace_path.exists() {
        return Err("workspace path does not exist".into());
    }

    let app_for_events = app.clone();
    let mut watcher = RecommendedWatcher::new(
        move |_| {
            if let Err(err) = debounced_index(&app_for_events) {
                eprintln!("auto index failed: {err}");
            }
        },
        Config::default(),
    )
    .map_err(|err| err.to_string())?;

    watcher
        .watch(workspace_path, RecursiveMode::Recursive)
        .map_err(|err| err.to_string())?;

    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}

fn debounced_index(app: &AppHandle) -> Result<(), String> {
    let watcher_state = app.state::<WatcherState>();
    let db_state = app.state::<DbState>();

    {
        let mut last = watcher_state.last_run.lock().map_err(|e| e.to_string())?;
        if last
            .map(|t| t.elapsed() < Duration::from_secs(3))
            .unwrap_or(false)
        {
            return Ok(());
        }
        *last = Some(Instant::now());
    }

    let settings = {
        let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
        settings::load(&conn).map_err(|e| e.to_string())?
    };

    if !settings.rag_enabled || !settings.rag_auto_index {
        return Ok(());
    }

    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    rt.block_on(async {
        let indexed_times = {
            let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
            rag::indexed_path_times(&conn).map_err(|e| e.to_string())
        }?;

        let (paths_to_replace, chunks, _summary) =
            rag::build_incremental_index(&settings, &indexed_times)
                .await
                .map_err(|e| e.to_string())?;

        let conn = db_state.conn.lock().map_err(|e| e.to_string())?;
        rag::persist_incremental_index(&conn, &paths_to_replace, &chunks).map_err(|e| e.to_string())
    })
}
