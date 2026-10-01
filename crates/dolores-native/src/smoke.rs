//! Explicit, feature-gated developer runner. Drives the real controller and
//! HTTP fixture, captures the real renderer; it does not prove pointer/IME UAT.
use super::{App, Event as AppEvent};
use iced::{widget::text_editor, Task};
use std::path::PathBuf;

pub fn install_trace() {
    if std::env::var_os("DOLORES_TRACE_FILE").is_some() {
        struct Logger;
        impl log::Log for Logger {
            fn enabled(&self, _: &log::Metadata) -> bool {
                true
            }
            fn log(&self, record: &log::Record) {
                if record.target().starts_with("iced") {
                    trace_line(format!("{} {}", record.level(), record.args()));
                }
            }
            fn flush(&self) {}
        }
        let _ = log::set_logger(Box::leak(Box::new(Logger)));
        log::set_max_level(log::LevelFilter::Warn);
    }
}
fn trace_line(line: String) {
    use std::io::Write;
    if let Some(path) = std::env::var_os("DOLORES_TRACE_FILE") {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(file, "{line}");
        }
    }
}
pub fn trace_subscription() -> iced::Subscription<AppEvent> {
    if std::env::var_os("DOLORES_TRACE_FILE").is_none() {
        return iced::Subscription::none();
    }
    iced::event::listen_raw(|event, _, _| {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static COUNT: AtomicUsize = AtomicUsize::new(0);
        if COUNT.fetch_add(1, Ordering::Relaxed) < 128 {
            match event {
                iced::Event::Window(event) => trace_line(format!("window {event:?}")),
                iced::Event::Mouse(_) => trace_line("mouse".into()),
                _ => trace_line("other".into()),
            }
        }
        None
    })
}

#[derive(Debug, Clone)]
pub enum Event {
    Send(String),
    Save,
    Select(String),
    Capture(String),
    Captured(String, iced::window::Screenshot),
}
pub struct Smoke {
    directory: PathBuf,
    phase: u8,
    passed: Vec<&'static str>,
    pub loaded: bool,
}
impl Smoke {
    pub fn from_environment() -> Option<Self> {
        let directory = PathBuf::from(std::env::var_os("DOLORES_SMOKE_DIR")?);
        assert!(
            directory.is_absolute(),
            "Smoke output directory must be absolute"
        );
        assert!(
            std::env::var_os("DOLORES_DATA_DIR").is_some(),
            "Smoke requires an isolated DOLORES_DATA_DIR"
        );
        std::fs::create_dir_all(&directory).expect("Create smoke output directory");
        Some(Self {
            directory,
            phase: 0,
            passed: vec![],
            loaded: false,
        })
    }
}
fn later(event: Event) -> Task<AppEvent> {
    Task::perform(
        async {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            event
        },
        AppEvent::Smoke,
    )
}
impl App {
    pub fn smoke_advance(&mut self) -> Task<AppEvent> {
        let Some(smoke) = self.smoke.as_mut() else {
            return Task::none();
        };
        if self.busy {
            return Task::none();
        }
        match smoke.phase {
            0 => {
                assert!(self.sessions.is_empty(), "Use a fresh smoke data directory");
                smoke.phase = 1;
                self.settings = true;
                self.base_url = "http://127.0.0.1:19421/v1".into();
                self.model = "dolores-mock".into();
                later(Event::Save)
            }
            1 if self.provider.is_some() && !self.settings => {
                smoke.passed.push("configure connection");
                smoke.phase = 2;
                later(Event::Send("hello".into()))
            }
            2 if self.active.is_none() && self.messages.len() == 2 => {
                assert!(self.messages[1].content.contains("你好"));
                smoke.passed.push("Unicode HTTP stream and atomic history");
                smoke.phase = 3;
                later(Event::Send("slow".into()))
            }
            3 if self
                .active
                .as_ref()
                .is_some_and(|active| !active.answer.is_empty()) =>
            {
                smoke.phase = 4;
                Task::done(AppEvent::Stop)
            }
            4 if self.active.is_none() => {
                assert!(self.error.as_deref().is_some_and(|e| e.contains("stopped")));
                assert_eq!(self.messages.len(), 2);
                assert_eq!(self.composer.text().trim(), "slow");
                smoke
                    .passed
                    .push("Stop restores draft without partial save");
                smoke.phase = 5;
                later(Event::Send("fail".into()))
            }
            5 if self.active.is_none()
                && self.error.is_some()
                && self.composer.text().trim() == "fail" =>
            {
                assert!(!self.error.as_ref().unwrap().contains("fixture-private"));
                assert_eq!(self.messages.len(), 2);
                assert_eq!(self.composer.text().trim(), "fail");
                smoke
                    .passed
                    .push("HTTP failure restores draft and hides raw body");
                smoke.phase = 6;
                later(Event::Send("truncated".into()))
            }
            6 if self.active.is_none()
                && self.error.is_some()
                && self.composer.text().trim() == "truncated" =>
            {
                assert_eq!(self.messages.len(), 2);
                assert_eq!(self.composer.text().trim(), "truncated");
                smoke
                    .passed
                    .push("Truncated stream never saves partial turn");
                smoke.phase = 7;
                later(Event::Send("slow".into()))
            }
            7 if self.active.is_none() && self.messages.len() == 4 => {
                smoke.phase = 8;
                smoke.loaded = false;
                later(Event::Select(self.selected.clone().unwrap()))
            }
            8 if smoke.loaded && self.active.is_none() && self.messages.len() == 4 => {
                smoke.passed.push("Reload two completed turns from SQLite");
                smoke.phase = 9;
                self.mode = iced::theme::Mode::Light;
                later(Event::Capture("conversation-light".into()))
            }
            _ => Task::none(),
        }
    }
    pub fn smoke_update(&mut self, event: Event) -> Task<AppEvent> {
        match event {
            Event::Save => self.update(AppEvent::Save),
            Event::Send(input) => {
                self.composer = text_editor::Content::with_text(&input);
                self.update(AppEvent::Send)
            }
            Event::Select(id) => self.update(AppEvent::Select(id)),
            Event::Capture(name) => iced::window::oldest().then(move |id| match id {
                Some(id) => {
                    let name = name.clone();
                    iced::window::screenshot(id)
                        .map(move |shot| AppEvent::Smoke(Event::Captured(name.clone(), shot)))
                }
                None => Task::none(),
            }),
            Event::Captured(name, shot) => {
                let smoke = self.smoke.as_mut().unwrap();
                let file =
                    std::fs::File::create(smoke.directory.join(format!("{name}.png"))).unwrap();
                let mut encoder = png::Encoder::new(file, shot.size.width, shot.size.height);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder
                    .write_header()
                    .unwrap()
                    .write_image_data(&shot.rgba)
                    .unwrap();
                match name.as_str() {
                    "conversation-light" => {
                        self.mode = iced::theme::Mode::Dark;
                        later(Event::Capture("conversation-dark".into()))
                    }
                    "conversation-dark" => {
                        self.settings = true;
                        later(Event::Capture("settings-dark".into()))
                    }
                    "settings-dark" => {
                        self.settings = false;
                        iced::window::oldest().then(|id| match id {
                            Some(id) => Task::batch([
                                iced::window::resize(id, iced::Size::new(620.0, 700.0)),
                                later(Event::Capture("conversation-narrow-dark".into())),
                            ]),
                            None => Task::none(),
                        })
                    }
                    "conversation-narrow-dark" => {
                        self.settings = false;
                        self.mode = iced::theme::Mode::Light;
                        let report = serde_json::json!({ "passed": smoke.passed, "messageCount": self.messages.len(), "screenshots": 4,
                            "note": "Controller events and real HTTP fixture; pointer, keyboard and IME interaction not certified." });
                        std::fs::write(
                            smoke.directory.join("report.json"),
                            serde_json::to_vec_pretty(&report).unwrap(),
                        )
                        .unwrap();
                        smoke.phase = 10;
                        Task::none()
                    }
                    _ => Task::none(),
                }
            }
        }
    }
}
