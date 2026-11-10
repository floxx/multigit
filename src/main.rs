//! multigit binary: loads config and history, starts the window.

use multigit::infrastructure::backend::GitBackend;
use multigit::infrastructure::store::{self, Paths};
use multigit::ui::{MultigitApp, app::App};
use std::sync::Arc;

fn main() -> eframe::Result {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let paths = Paths::from_env();
    let config = match &paths {
        Some(p) => store::load_config(&p.config_file).map_err(|e| e.to_string()),
        None => Err("HOME is not set".to_owned()),
    };
    let history = paths
        .as_ref()
        .map(|p| store::load_history(&p.history_file))
        .unwrap_or_default();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("multigit")
            .with_app_id("multigit")
            .with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "multigit",
        options,
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();
            let notify = Arc::new(move || ctx.request_repaint());
            let backend = Arc::new(GitBackend::default());
            Ok(Box::new(MultigitApp {
                app: App::new(config, paths, history, backend, notify),
            }))
        }),
    )
}
