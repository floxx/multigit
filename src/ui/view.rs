//! Drawing of the board: toolbar, cards, output panel, CLI, toasts, dialogs.

use crate::domain::naming::{subtitles, title};
use crate::domain::output::{Run, SectionState};
use crate::domain::status::{Head, RepoState, Upstream};
use crate::ui::app::{App, Card};
use crate::ui::cli::{CliAction, CliContext};
use crate::ui::settings;
use egui::{Color32, RichText, Ui};
use std::path::PathBuf;

/// Content width of one card.
const CARD_WIDTH: f32 = 290.0;
/// Inner margin of a card frame.
const CARD_MARGIN: f32 = 8.0;
/// Width a card takes on the board: content, margins and frame stroke.
const CARD_OUTER_WIDTH: f32 = CARD_WIDTH + 2.0 * CARD_MARGIN + 2.0;

/// Something the user clicked that changes the board; applied after drawing.
enum BoardAction {
    Remove(PathBuf),
    Button(usize, PathBuf),
    Toggle(PathBuf),
}

fn ok_color(ui: &Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(0x6c, 0xc6, 0x6c)
    } else {
        Color32::from_rgb(0x1a, 0x7f, 0x37)
    }
}

/// Folder picker for (+). Blocks until the dialog closes.
fn pick_folder() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .set_title("Add repository or folder with repositories")
        .pick_folder()
}

impl App {
    /// Draw the whole app into `ui` (the eframe root ui, or a test harness).
    pub fn show(&mut self, ui: &mut Ui) {
        self.handle_events();
        if let Some(folder) = self.take_picked_folder() {
            self.add_folder(&folder);
        }
        let mut next = self.tick(std::time::Instant::now());
        if !self.toasts.is_empty() {
            // Repaint soon enough to take expired toasts down.
            next = next.min(std::time::Duration::from_millis(500));
        }
        ui.ctx()
            .request_repaint_after(next.max(std::time::Duration::from_millis(200)));

        egui::Panel::top("toolbar").show(ui, |ui| self.toolbar(ui));
        egui::Panel::bottom("cli").show(ui, |ui| {
            ui.add_space(4.0);
            self.cli_line(ui);
            ui.add_space(4.0);
        });
        egui::Panel::bottom("output")
            .resizable(true)
            .default_size(220.0)
            .min_size(60.0)
            .show(ui, |ui| self.output_panel(ui));
        egui::CentralPanel::default().show(ui, |ui| self.board(ui));

        self.toasts_overlay(ui);
        self.confirm_dialog(ui);
        settings::show(self, ui.ctx());
    }

    fn toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal_wrapped(|ui| {
            if ui
                .button(RichText::new(" + ").strong())
                .on_hover_text("Add a repository, or scan a folder for repositories")
                .clicked()
            {
                self.open_folder_picker(pick_folder);
            }
            ui.separator();
            ui.label("Mark:");
            if ui.button("all").clicked() {
                self.mark_all();
            }
            if ui.button("none").clicked() {
                self.mark_none();
            }
            if ui.button("dirty").clicked() {
                self.mark_dirty();
            }
            if ui.button("behind").clicked() {
                self.mark_behind();
            }
            ui.separator();
            let any_marked = !self.marked.is_empty();
            ui.label(format!("{} marked:", self.marked.len()));
            let labels: Vec<String> = self
                .config
                .buttons
                .iter()
                .map(|b| b.label.clone())
                .collect();
            for (i, label) in labels.iter().enumerate() {
                if ui
                    .add_enabled(any_marked, egui::Button::new(label))
                    .on_hover_text("Run on all marked repositories")
                    .clicked()
                {
                    self.run_button_on_marked(i);
                }
            }
            if ui
                .add_enabled(any_marked, egui::Button::new("Remove marked"))
                .on_hover_text("Take the marked repositories off the board (files stay)")
                .clicked()
            {
                self.remove_marked();
            }
            ui.separator();
            if ui.button("Settings").clicked() {
                self.settings_open = true;
            }
        });
        if let Some(error) = &self.config_error {
            ui.colored_label(
                ui.visuals().error_fg_color,
                format!("Config not loaded, changes are not saved: {error}"),
            );
        }
    }

    fn board(&mut self, ui: &mut Ui) {
        if self.cards.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.weak("No repositories yet. Click + to add a repository or a folder.");
            });
            return;
        }
        let paths: Vec<PathBuf> = self.cards.iter().map(|c| c.path.clone()).collect();
        let subs = subtitles(&paths);
        let mut actions = Vec::new();
        let gap = 10.0;
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Containers do not wrap inside `horizontal_wrapped`, so lay out
                // fixed rows: as many columns as fit the width.
                let columns = (((ui.available_width() + gap) / (CARD_OUTER_WIDTH + gap)).floor()
                    as usize)
                    .max(1);
                let rows = self.cards.iter().zip(&subs).collect::<Vec<_>>();
                for row in rows.chunks(columns) {
                    ui.horizontal_top(|ui| {
                        for (i, (card, sub)) in row.iter().enumerate() {
                            if i > 0 {
                                ui.add_space(gap);
                            }
                            let marked = self.marked.contains(&card.path);
                            let busy = self.is_busy(&card.path);
                            card_ui(
                                ui,
                                card,
                                sub.as_deref(),
                                marked,
                                busy,
                                &self.config.buttons,
                                &mut actions,
                            );
                        }
                    });
                    ui.add_space(gap);
                }
            });
        for action in actions {
            match action {
                BoardAction::Remove(p) => self.remove(&p),
                BoardAction::Button(i, p) => self.run_button(i, &p),
                BoardAction::Toggle(p) => {
                    if !self.marked.remove(&p) {
                        self.marked.insert(p);
                    }
                }
            }
        }
    }

    fn cli_line(&mut self, ui: &mut Ui) {
        let values = self.repo_values();
        let cx = CliContext {
            enabled: !self.marked.is_empty(),
            history: &self.history,
            vocab: &self.vocab,
            values: &values,
            auto_insert_single: self.config.auto_insert_single_match,
        };
        let mut cli = std::mem::take(&mut self.cli);
        let actions = cli.show(ui, &cx);
        self.cli = cli;
        for action in actions {
            match action {
                CliAction::Run(line) => self.run_cli_line(&line),
                CliAction::NeedOptions(cmd) => self.request_options(&cmd),
            }
        }
    }

    fn output_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.strong("Output");
            if ui
                .add_enabled(!self.log.is_empty(), egui::Button::new("Clear"))
                .clicked()
            {
                self.log.clear();
            }
        });
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for run in self.log.runs() {
                    run_ui(ui, run);
                }
            });
    }

    fn toasts_overlay(&self, ui: &Ui) {
        if self.toasts.is_empty() {
            return;
        }
        egui::Area::new(egui::Id::new("toasts"))
            .order(egui::Order::Tooltip)
            .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 48.0))
            .show(ui.ctx(), |ui| {
                for toast in &self.toasts {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.label(&toast.text);
                    });
                }
            });
    }

    fn confirm_dialog(&mut self, ui: &Ui) {
        let Some(pending) = &self.confirm else {
            return;
        };
        let names: Vec<String> = pending.repos.iter().map(|p| title(p)).collect();
        let command = pending.command.clone();
        let mut decision = None;
        egui::Modal::new(egui::Id::new("confirm-destructive")).show(ui.ctx(), |ui| {
            ui.heading("Destructive command");
            ui.label(format!(
                "Run `git {command}` in {} repositories? This can discard work.",
                names.len()
            ));
            ui.label(names.join(", "));
            ui.horizontal(|ui| {
                if ui.button("Run").clicked() {
                    decision = Some(true);
                }
                if ui.button("Cancel").clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    decision = Some(false);
                }
            });
        });
        match decision {
            Some(true) => self.confirm_run(),
            Some(false) => self.cancel_run(),
            None => {}
        }
    }
}

fn card_ui(
    ui: &mut Ui,
    card: &Card,
    subtitle: Option<&str>,
    marked: bool,
    busy: bool,
    buttons: &[crate::domain::config::ButtonDef],
    actions: &mut Vec<BoardAction>,
) {
    let path_text = card.path.display().to_string();
    egui::Frame::group(ui.style())
        .inner_margin(CARD_MARGIN)
        .show(ui, |ui| {
            ui.set_width(CARD_WIDTH);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let mut checked = marked;
                    if ui
                        .checkbox(&mut checked, "")
                        .on_hover_text("Mark")
                        .changed()
                    {
                        actions.push(BoardAction::Toggle(card.path.clone()));
                    }
                    ui.label(RichText::new(title(&card.path)).strong().size(15.0))
                        .on_hover_text(&path_text);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .small_button("×")
                            .on_hover_text("Remove from board (files stay)")
                            .clicked()
                        {
                            actions.push(BoardAction::Remove(card.path.clone()));
                        }
                    });
                });
                if let Some(sub) = subtitle {
                    ui.weak(sub).on_hover_text(&path_text);
                }
                let greyed = card.snapshot().is_none();
                if greyed {
                    ui.multiply_opacity(0.55);
                }
                status_lines(ui, card);
                if let Some(error) = &card.fetch_error {
                    ui.colored_label(ui.visuals().error_fg_color, format!("fetch: {error}"));
                }
                if busy {
                    ui.colored_label(ui.visuals().warn_fg_color, "running");
                } else {
                    match card.last_command {
                        Some(true) => {
                            ui.colored_label(ok_color(ui), "last command: ok");
                        }
                        Some(false) => {
                            ui.colored_label(ui.visuals().error_fg_color, "last command: failed");
                        }
                        None => {}
                    }
                }
                ui.horizontal_wrapped(|ui| {
                    for (i, b) in buttons.iter().enumerate() {
                        if ui
                            .add_enabled(!greyed, egui::Button::new(&b.label).small())
                            .on_hover_text(format!("git {}", b.args))
                            .clicked()
                        {
                            actions.push(BoardAction::Button(i, card.path.clone()));
                        }
                    }
                });
            });
        });
}

fn status_lines(ui: &mut Ui, card: &Card) {
    let snap = match &card.probe {
        None => {
            ui.weak("loading");
            return;
        }
        Some(Err(e)) => {
            ui.colored_label(ui.visuals().error_fg_color, e.to_string());
            return;
        }
        Some(Ok(snap)) => snap,
    };
    let branch = match &snap.status.head {
        Head::Branch(b) => b.clone(),
        Head::Unborn(b) => format!("{b} (no commits)"),
        Head::Detached(sha) => format!("detached @ {sha}"),
    };
    ui.horizontal(|ui| {
        ui.weak("branch");
        ui.label(RichText::new(branch).monospace());
        match snap.state {
            RepoState::Rebasing => {
                ui.colored_label(ui.visuals().warn_fg_color, "rebasing");
            }
            RepoState::Merging => {
                ui.colored_label(ui.visuals().warn_fg_color, "merging");
            }
            RepoState::Normal => {}
        }
    });
    let c = snap.status.counts;
    ui.horizontal(|ui| {
        ui.weak("status");
        if c.is_clean() {
            ui.colored_label(ok_color(ui), "clean");
        } else {
            ui.label(
                RichText::new(format!(
                    "+{} ~{} ?{} !{}",
                    c.staged, c.modified, c.untracked, c.conflicts
                ))
                .monospace(),
            )
            .on_hover_text("+staged ~modified ?untracked !conflicts");
        }
    });
    ui.horizontal(|ui| match snap.status.upstream {
        Upstream::None => {
            ui.weak("no upstream");
        }
        Upstream::Gone => {
            ui.colored_label(ui.visuals().warn_fg_color, "upstream gone");
        }
        Upstream::Tracking { ahead, behind } => {
            ui.weak("unpushed");
            ui.label(RichText::new(ahead.to_string()).strong());
            ui.weak("unpulled");
            ui.label(RichText::new(behind.to_string()).strong());
        }
    });
}

fn run_ui(ui: &mut Ui, run: &Run) {
    ui.separator();
    ui.label(
        RichText::new(format!("git {}", run.command))
            .monospace()
            .strong(),
    );
    for section in &run.sections {
        let (status, color) = match &section.state {
            SectionState::Queued => ("queued".to_owned(), ui.visuals().weak_text_color()),
            SectionState::Running => ("running".to_owned(), ui.visuals().warn_fg_color),
            SectionState::Done(out) if out.success() => ("exit 0".to_owned(), ok_color(ui)),
            SectionState::Done(out) => (
                out.exit_code
                    .map_or("killed".to_owned(), |c| format!("exit {c}")),
                ui.visuals().error_fg_color,
            ),
            SectionState::SpawnFailed(_) => {
                ("failed to start".to_owned(), ui.visuals().error_fg_color)
            }
        };
        let header = RichText::new(format!("{}  {status}", title(&section.repo))).color(color);
        egui::CollapsingHeader::new(header)
            .id_salt((run.id, &section.repo))
            .default_open(true)
            .show(ui, |ui| match &section.state {
                SectionState::Done(out) => {
                    if !out.stdout.is_empty() {
                        ui.label(RichText::new(out.stdout.trim_end()).monospace());
                    }
                    if !out.stderr.is_empty() {
                        ui.label(RichText::new(out.stderr.trim_end()).monospace().weak());
                    }
                }
                SectionState::SpawnFailed(e) => {
                    ui.label(RichText::new(e).monospace());
                }
                _ => {}
            });
    }
}
