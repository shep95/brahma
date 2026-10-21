//! Window › Performance: a local log of how the app has been performing in this session (one
//! sample a second: frame rate, UI time, the last canvas refresh's composite and upload times,
//! GPU path; plus every command that took longer than a frame), kept in memory and exportable
//! as JSON through the save dialog (a download on the web). Nothing leaves the device unless
//! the person exports and shares the file: it is the thing to attach to a bug report.

use std::collections::VecDeque;

use egui::{Color32, RichText, vec2};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::theme::Tokens;
use crate::{BrahmaApp, widgets};

/// One second of the session.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    /// Milliseconds since the app started.
    pub at_ms: f64,
    pub fps: f32,
    pub ui_ms: f64,
    pub ui_max_ms: f64,
    pub composite_ms: f64,
    pub upload_ms: f64,
    pub gpu: bool,
    /// Open documents and the active one's size, so a slow second can be read in context.
    pub documents: usize,
    pub megapixels: f64,
}

/// A command that took longer than a frame.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlowCommand {
    pub at_ms: f64,
    pub id: String,
    pub ms: f64,
}

/// Commands at or above this take a row in the log (a 60 Hz frame is 16.7 ms).
pub const SLOW_COMMAND_MS: f64 = 16.0;
/// How many seconds the log keeps (an hour).
pub const MAX_SAMPLES: usize = 3600;
pub const MAX_SLOW: usize = 500;

/// The log and the window's state.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PerfLogUi {
    pub open: bool,
    #[serde(skip)]
    pub samples: VecDeque<Sample>,
    #[serde(skip)]
    pub slow: VecDeque<SlowCommand>,
    #[serde(skip)]
    pub last_sample_ms: f64,
    #[serde(skip)]
    pub status: Option<String>,
}

impl Clone for PerfLogUi {
    fn clone(&self) -> Self {
        Self { open: self.open, samples: self.samples.clone(), slow: self.slow.clone(), last_sample_ms: self.last_sample_ms, status: self.status.clone() }
    }
}

impl std::fmt::Debug for PerfLogUi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerfLogUi").field("open", &self.open).field("samples", &self.samples.len()).field("slow", &self.slow.len()).finish()
    }
}

impl PartialEq for PerfLogUi {
    fn eq(&self, o: &Self) -> bool {
        self.open == o.open && self.samples.len() == o.samples.len() && self.slow.len() == o.slow.len()
    }
}

/// Once a second: record a sample. Called every frame.
pub fn tick(app: &mut BrahmaApp) {
    let now = crate::gpu_canvas::now_ms();
    if now - app.ui.perf_log.last_sample_ms < 1000.0 && !app.ui.perf_log.samples.is_empty() {
        return;
    }
    app.ui.perf_log.last_sample_ms = now;
    let (documents, megapixels) = (
        app.session.documents().len(),
        app.session.active().map(|d| f64::from(d.doc.size.width) * f64::from(d.doc.size.height) / 1e6).unwrap_or(0.0),
    );
    let p = &app.perf;
    let sample = Sample {
        at_ms: now,
        fps: app.fps,
        ui_ms: p.ui_ms,
        ui_max_ms: p.ui_max_ms,
        composite_ms: p.composite_ms,
        upload_ms: p.upload_ms,
        gpu: p.gpu,
        documents,
        megapixels: (megapixels * 100.0).round() / 100.0,
    };
    let log = &mut app.ui.perf_log;
    log.samples.push_back(sample);
    while log.samples.len() > MAX_SAMPLES {
        log.samples.pop_front();
    }
}

/// A command finished: log it when it was slow.
pub fn command(app: &mut BrahmaApp, id: &str, ms: f64) {
    if ms < SLOW_COMMAND_MS || !ms.is_finite() {
        return;
    }
    let log = &mut app.ui.perf_log;
    log.slow.push_back(SlowCommand { at_ms: crate::gpu_canvas::now_ms(), id: id.to_string(), ms: (ms * 10.0).round() / 10.0 });
    while log.slow.len() > MAX_SLOW {
        log.slow.pop_front();
    }
}

/// The log as JSON: the build, the machine's GPU path, the samples and the slow commands.
pub fn report(app: &BrahmaApp) -> Value {
    let log = &app.ui.perf_log;
    json!({
        "brahma": brahma_engine::build_info::VERSION,
        "platform": std::env::consts::OS,
        "gpu": app.perf.gpu_info,
        "uptimeMs": crate::gpu_canvas::now_ms(),
        "samples": log.samples,
        "slowCommands": log.slow,
    })
}

/// Export the log through the save dialog.
pub fn export(app: &mut BrahmaApp) -> Result<Value, String> {
    app.pick_save("brahma-performance.json", |app, path| {
        let bytes = serde_json::to_vec_pretty(&report(app)).map_err(|e| e.to_string())?;
        let write = app.services.write.as_mut().ok_or("this build cannot write files")?;
        write(&path, &bytes)?;
        app.ui.perf_log.status = Some(format!("{}: {path}", tl!("Saved")));
        Ok(json!({"path": path, "bytes": bytes.len()}))
    })
}

fn sparkline(ui: &mut egui::Ui, values: &[f32], max: f32, color: Color32, label: &str) {
    let t = Tokens::get(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 36.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, t.radius_sm, t.field);
    if values.len() >= 2 && max > 0.0 {
        let n = values.len();
        let pts: Vec<egui::Pos2> = values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let x = rect.left() + rect.width() * i as f32 / (n - 1) as f32;
                let y = rect.bottom() - rect.height() * (v / max).clamp(0.0, 1.0);
                egui::pos2(x, y)
            })
            .collect();
        ui.painter().add(egui::Shape::line(pts, egui::Stroke::new(1.5, color)));
    }
    ui.painter().text(rect.left_top() + vec2(6.0, 4.0), egui::Align2::LEFT_TOP, label, egui::FontId::proportional(10.5), t.text_faint);
}

pub fn window(app: &mut BrahmaApp, ctx: &egui::Context) {
    if !app.ui.perf_log.open {
        return;
    }
    let t = Tokens::get(ctx);
    let mut close = false;
    let mut export_now = false;
    let mut clear = false;
    let samples: Vec<Sample> = app.ui.perf_log.samples.iter().cloned().collect();
    let slow: Vec<SlowCommand> = app.ui.perf_log.slow.iter().rev().take(12).cloned().collect();
    let status = app.ui.perf_log.status.clone();
    let now = samples.last().cloned();
    crate::analysis_ui::panel_window(app, ctx, "perf-log", tl!("Performance"), vec2(-20.0, 60.0), 380.0, |ui| {
        close = crate::analysis_ui::title_row(ui, tl!("Performance"));
        if let Some(s) = &now {
            ui.label(
                RichText::new(format!("{:.0} fps  ·  ui {:.1} ms (peak {:.1})  ·  composite {:.1} ms  ·  {}", s.fps, s.ui_ms, s.ui_max_ms, s.composite_ms, if s.gpu { "gpu" } else { "cpu" }))
                    .color(t.text)
                    .size(11.5),
            );
        }
        let fps: Vec<f32> = samples.iter().map(|s| s.fps).collect();
        let ui_ms: Vec<f32> = samples.iter().map(|s| s.ui_max_ms as f32).collect();
        let comp: Vec<f32> = samples.iter().map(|s| s.composite_ms as f32).collect();
        sparkline(ui, &fps, 120.0, t.accent, &format!("{} ({} s)", tl!("frames per second"), samples.len()));
        sparkline(ui, &ui_ms, 50.0, Color32::from_rgb(230, 160, 90), tl!("worst UI frame, ms (50 = full)"));
        sparkline(ui, &comp, 200.0, Color32::from_rgb(120, 170, 230), tl!("canvas composite, ms (200 = full)"));
        widgets::hairline(ui);
        widgets::section_label(ui, "Slow commands");
        if slow.is_empty() {
            ui.label(RichText::new(tl!("None yet: every command finished within a frame.")).color(t.text_dim).size(11.0));
        }
        for c in &slow {
            ui.label(RichText::new(format!("{:>7.1} ms  {}", c.ms, c.id)).color(t.text_dim).monospace().size(11.0));
        }
        widgets::hairline(ui);
        ui.horizontal(|ui| {
            if widgets::primary_button(ui, tl!("Export…"), 0.0).on_hover_text(tl!("Save the log as JSON to attach to a bug report; it stays on this device until you share it.")).clicked() {
                export_now = true;
            }
            if widgets::secondary_button(ui, tl!("Clear"), 0.0).clicked() {
                clear = true;
            }
        });
        if let Some(s) = &status {
            ui.label(RichText::new(s).color(t.text_dim).size(11.0));
        }
    });
    if close {
        app.ui.perf_log.open = false;
    }
    if clear {
        app.ui.perf_log.samples.clear();
        app.ui.perf_log.slow.clear();
        app.ui.perf_log.status = None;
    }
    if export_now
        && let Err(e) = export(app)
    {
        app.ui.perf_log.status = Some(e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn samples_and_slow_commands_are_logged_and_bounded() {
        let mut app = BrahmaApp::new(brahma_engine::Session::new(), crate::Services::default());
        app.run("file.new", json!({"width": 100, "height": 50})).unwrap();
        tick(&mut app);
        tick(&mut app);
        assert_eq!(app.ui.perf_log.samples.len(), 1, "one sample a second");
        assert_eq!(app.ui.perf_log.samples[0].megapixels, 0.01);
        app.ui.perf_log.last_sample_ms = -1e9;
        tick(&mut app);
        assert_eq!(app.ui.perf_log.samples.len(), 2);
        command(&mut app, "fast.one", 2.0);
        command(&mut app, "slow.one", 42.26);
        command(&mut app, "nan.one", f64::NAN);
        assert_eq!(app.ui.perf_log.slow.len(), 1);
        assert_eq!((app.ui.perf_log.slow[0].id.as_str(), app.ui.perf_log.slow[0].ms), ("slow.one", 42.3));
        for i in 0..(MAX_SLOW + 10) {
            command(&mut app, &format!("c{i}"), 20.0);
        }
        assert_eq!(app.ui.perf_log.slow.len(), MAX_SLOW);
        for _ in 0..(MAX_SAMPLES + 5) {
            app.ui.perf_log.last_sample_ms = -1e9;
            tick(&mut app);
        }
        assert_eq!(app.ui.perf_log.samples.len(), MAX_SAMPLES);
        let r = report(&app);
        assert_eq!(r["samples"].as_array().map(Vec::len), Some(MAX_SAMPLES));
        assert!(r["brahma"].is_string() && r["gpu"].is_object());
    }

    /// Export writes the JSON report to the path the dialog answers (a download on the web).
    #[test]
    fn export_writes_the_report_through_the_save_dialog() {
        let written: Arc<Mutex<Vec<(String, usize)>>> = Arc::default();
        let w = written.clone();
        let services = crate::Services {
            write: Some(Box::new(move |path: &str, bytes: &[u8]| {
                w.lock().unwrap().push((path.to_string(), bytes.len()));
                Ok(())
            })),
            file_dialog: Some(Box::new(|req, _parent, reply| match req {
                crate::FileDialogRequest::Save { suggested } => reply.send(Some(crate::FileDialogAnswer::SaveTo(suggested))),
                _ => reply.send(None),
            })),
            ..Default::default()
        };
        let mut app = BrahmaApp::new(brahma_engine::Session::new(), services);
        tick(&mut app);
        command(&mut app, "slow.one", 30.0);
        export(&mut app).unwrap();
        let ctx = egui::Context::default();
        for _ in 0..3 {
            app.poll_file_dialog(&ctx, None);
        }
        let w = written.lock().unwrap();
        assert_eq!(w.len(), 1, "{:?}", app.ui.perf_log.status);
        assert!(w[0].0.ends_with("brahma-performance.json") && w[0].1 > 100);
        assert!(app.ui.perf_log.status.as_deref().is_some_and(|s| s.starts_with("Saved")));
    }
}
