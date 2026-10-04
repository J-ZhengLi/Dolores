#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[cfg(feature = "smoke")]
mod smoke;
mod turn;

use dolores_core::{
    ConnectionPreferences, Message as ChatMessage, ModelProvider, Role, Session, SessionStore,
};
use dolores_provider_openai::OpenAiProvider;
use dolores_store_sqlite::SqliteStore;
use iced::{
    widget::{
        button, column, container, pick_list, row, scrollable, space, text, text_editor, text_input,
    },
    Element, Length, Subscription, Task, Theme,
};
use std::{path::PathBuf, sync::Arc};
use tokio_util::sync::CancellationToken;

// Fixed worker count keeps the runtime small even on many-core machines.
struct ChatExecutor(tokio::runtime::Runtime);
impl iced::Executor for ChatExecutor {
    fn new() -> Result<Self, std::io::Error> {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .max_blocking_threads(2)
            .enable_all()
            .build()
            .map(Self)
    }
    fn spawn(&self, future: impl std::future::Future<Output = ()> + Send + 'static) {
        self.0.spawn(future);
    }
    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        let _guard = self.0.enter();
        f()
    }
    fn block_on<T>(&self, future: impl std::future::Future<Output = T>) -> T {
        self.0.block_on(future)
    }
}
fn main() -> iced::Result {
    iced::application(App::boot, App::update, App::view)
        .title("Dolores")
        .executor::<ChatExecutor>()
        .subscription(App::subscription)
        .theme(App::theme)
        .window(iced::window::Settings {
            size: iced::Size::new(1120.0, 780.0),
            min_size: Some(iced::Size::new(600.0, 500.0)),
            ..Default::default()
        })
        .run()
}
fn data_directory() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("DOLORES_DATA_DIR") {
        let path = PathBuf::from(path);
        if !path.is_absolute() {
            return Err("DOLORES_DATA_DIR must be an absolute directory.".into());
        }
        return Ok(path);
    }
    directories::BaseDirs::new()
        .map(|dirs| dirs.data_dir().join("dev.dolores.desktop"))
        .ok_or_else(|| "Could not locate the application data directory.".into())
}

#[derive(Debug, Clone)]
enum Event {
    Booted(Result<Snapshot, String>),
    Sessions(Result<Vec<Session>, String>),
    Loaded(Result<(String, Vec<ChatMessage>), String>),
    Deleted(Result<Vec<Session>, String>),
    Configured(Result<ConnectionPreferences, String>),
    Select(String),
    New,
    Delete,
    ConfirmDelete,
    Settings,
    CloseSettings,
    BaseUrl(String),
    Model(String),
    Key(String),
    Save,
    Edit(text_editor::Action),
    Send,
    Stop,
    Turn(turn::TurnEvent),
    Theme(iced::theme::Mode),
    Resized(f32),
    Copy(String),
    Escape,
    #[cfg(feature = "smoke")]
    Smoke(smoke::Event),
}
#[derive(Debug, Clone)]
struct Snapshot {
    sessions: Vec<Session>,
    preferences: ConnectionPreferences,
}
struct ActiveTurn {
    id: u64,
    user: String,
    answer: String,
    cancel: CancellationToken,
}
#[derive(Debug, Clone, PartialEq, Eq)]
struct SessionChoice {
    id: String,
    title: String,
}
impl std::fmt::Display for SessionChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.title)
    }
}
struct App {
    store: Option<Arc<dyn SessionStore>>,
    provider: Option<Arc<dyn ModelProvider>>,
    sessions: Vec<Session>,
    selected: Option<String>,
    messages: Vec<ChatMessage>,
    composer: text_editor::Content,
    preferences: ConnectionPreferences,
    base_url: String,
    model: String,
    key: String,
    settings: bool,
    deleting: bool,
    busy: bool,
    active: Option<ActiveTurn>,
    sequence: u64,
    error: Option<String>,
    mode: iced::theme::Mode,
    width: f32,
    #[cfg(feature = "smoke")]
    smoke: Option<smoke::Smoke>,
}
impl App {
    fn boot() -> (Self, Task<Event>) {
        #[cfg(feature = "smoke")]
        smoke::install_trace();
        let mut app = Self {
            store: None,
            provider: None,
            sessions: vec![],
            selected: None,
            messages: vec![],
            composer: text_editor::Content::new(),
            preferences: ConnectionPreferences::default(),
            base_url: String::new(),
            model: String::new(),
            key: String::new(),
            settings: false,
            deleting: false,
            busy: true,
            active: None,
            sequence: 0,
            error: None,
            mode: iced::theme::Mode::Light,
            width: 1120.0,
            #[cfg(feature = "smoke")]
            smoke: smoke::Smoke::from_environment(),
        };
        let bootstrap = data_directory().and_then(|directory| {
            std::fs::create_dir_all(&directory)
                .map_err(|_| "Could not create the local data directory.".to_string())?;
            SqliteStore::open(&directory.join("dolores.db"))
                .map(|store| Arc::new(store) as Arc<dyn SessionStore>)
        });
        let task = match bootstrap {
            Ok(store) => {
                app.store = Some(store.clone());
                Task::perform(
                    turn::blocking(move || {
                        Ok(Snapshot {
                            sessions: store.list()?,
                            preferences: store.preferences()?,
                        })
                    }),
                    Event::Booted,
                )
            }
            Err(error) => Task::done(Event::Booted(Err(error))),
        };
        (
            app,
            Task::batch([task, iced::system::theme().map(Event::Theme)]),
        )
    }
    fn subscription(&self) -> Subscription<Event> {
        let subscription = Subscription::batch([
            iced::system::theme_changes().map(Event::Theme),
            iced::window::resize_events().map(|(_, size)| Event::Resized(size.width)),
            iced::event::listen_with(|event, status, _| match (event, status) {
                (
                    iced::Event::Keyboard(iced::keyboard::Event::KeyPressed {
                        key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                        ..
                    }),
                    iced::event::Status::Ignored,
                ) => Some(Event::Escape),
                _ => None,
            }),
        ]);
        #[cfg(feature = "smoke")]
        {
            Subscription::batch([subscription, smoke::trace_subscription()])
        }
        #[cfg(not(feature = "smoke"))]
        subscription
    }
    fn theme(&self) -> Theme {
        let dark = self.mode == iced::theme::Mode::Dark;
        Theme::custom(
            "Dolores",
            iced::theme::Palette {
                background: if dark {
                    iced::Color::from_rgb8(24, 26, 29)
                } else {
                    iced::Color::from_rgb8(248, 247, 243)
                },
                text: if dark {
                    iced::Color::from_rgb8(234, 235, 232)
                } else {
                    iced::Color::from_rgb8(34, 39, 43)
                },
                primary: if dark {
                    iced::Color::from_rgb8(128, 172, 225)
                } else {
                    iced::Color::from_rgb8(45, 98, 164)
                },
                success: iced::Color::from_rgb8(56, 142, 111),
                warning: iced::Color::from_rgb8(184, 126, 34),
                danger: if dark {
                    iced::Color::from_rgb8(232, 136, 128)
                } else {
                    iced::Color::from_rgb8(166, 61, 51)
                },
            },
        )
    }
    fn locked(&self) -> bool {
        self.busy || self.active.is_some()
    }
    fn refresh(&self) -> Task<Event> {
        let Some(store) = self.store.clone() else {
            return Task::none();
        };
        Task::perform(turn::blocking(move || store.list()), Event::Sessions)
    }
    fn update(&mut self, event: Event) -> Task<Event> {
        #[cfg(feature = "smoke")]
        if let Event::Smoke(event) = event {
            return self.smoke_update(event);
        }
        #[cfg(feature = "smoke")]
        if matches!(&event, Event::Loaded(Ok(_))) {
            if let Some(smoke) = &mut self.smoke {
                smoke.loaded = true;
            }
        }
        let mut task = Task::none();
        match event {
            Event::Booted(result) => {
                self.busy = false;
                match result {
                    Ok(snapshot) => {
                        self.sessions = snapshot.sessions;
                        self.preferences = snapshot.preferences;
                        self.base_url = self.preferences.base_url.clone();
                        self.model = self.preferences.model.clone();
                        if let Some(session) = self.sessions.first() {
                            task = Task::done(Event::Select(session.id.clone()));
                        }
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            Event::Theme(mode) => self.mode = mode,
            Event::Resized(width) => self.width = width,
            Event::Copy(content) => task = iced::clipboard::write(content),
            Event::Escape if !self.locked() => {
                if self.settings {
                    self.settings = false;
                    self.key.clear();
                }
                self.deleting = false;
            }
            Event::Sessions(result) => match result {
                Ok(sessions) => self.sessions = sessions,
                Err(error) => self.error = Some(error),
            },
            Event::Select(id) if !self.locked() => {
                if let Some(store) = self.store.clone() {
                    self.busy = true;
                    self.error = None;
                    self.deleting = false;
                    task = Task::perform(
                        turn::blocking(move || Ok((id.clone(), store.messages(&id)?))),
                        Event::Loaded,
                    );
                }
            }
            Event::Loaded(result) => {
                self.busy = false;
                match result {
                    Ok((id, messages)) => {
                        self.selected = Some(id);
                        self.messages = messages;
                        self.composer = text_editor::Content::new();
                        task = iced::widget::operation::snap_to(
                            "conversation",
                            scrollable::RelativeOffset::END,
                        );
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            Event::New if !self.locked() => {
                self.selected = None;
                self.messages.clear();
                self.composer = text_editor::Content::new();
                self.error = None;
                self.deleting = false;
            }
            Event::Delete if !self.locked() => self.deleting = !self.deleting,
            Event::ConfirmDelete if !self.locked() => {
                if let (Some(store), Some(id)) = (self.store.clone(), self.selected.clone()) {
                    self.busy = true;
                    task = Task::perform(
                        turn::blocking(move || {
                            store.delete(&id)?;
                            store.list()
                        }),
                        Event::Deleted,
                    );
                }
            }
            Event::Deleted(result) => {
                self.busy = false;
                self.deleting = false;
                match result {
                    Ok(sessions) => {
                        self.sessions = sessions;
                        self.selected = None;
                        self.messages.clear();
                        self.composer = text_editor::Content::new();
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            Event::Settings if !self.locked() => {
                self.settings = true;
                self.base_url = self.preferences.base_url.clone();
                self.model = self.preferences.model.clone();
                self.error = None;
            }
            Event::CloseSettings if !self.busy => {
                self.settings = false;
                self.key.clear();
                self.error = None;
            }
            Event::BaseUrl(value) if !self.locked() => self.base_url = value,
            Event::Model(value) if !self.locked() => self.model = value,
            Event::Key(value) if !self.locked() => self.key = value,
            Event::Save if !self.locked() => {
                let preferences = ConnectionPreferences {
                    base_url: self.base_url.trim().into(),
                    model: self.model.trim().into(),
                };
                match OpenAiProvider::new(&preferences, self.key.clone()) {
                    Ok(provider) => {
                        if let Some(store) = self.store.clone() {
                            self.busy = true;
                            self.error = None;
                            self.provider = Some(Arc::new(provider));
                            task = Task::perform(
                                turn::blocking(move || {
                                    store.save_preferences(&preferences)?;
                                    Ok(preferences)
                                }),
                                Event::Configured,
                            );
                        }
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            Event::Configured(result) => {
                self.busy = false;
                match result {
                    Ok(preferences) => {
                        self.preferences = preferences;
                        self.settings = false;
                        self.key.clear();
                    }
                    Err(error) => {
                        self.provider = None;
                        self.error = Some(error);
                    }
                }
            }
            Event::Edit(action) if !self.locked() => self.composer.perform(action),
            Event::Send if !self.locked() && !self.settings => {
                let user = self.composer.text().trim().to_owned();
                if let Err(error) = dolores_core::prepare_context(vec![], &user) {
                    self.error = Some(error);
                } else if let (Some(store), Some(provider)) =
                    (self.store.clone(), self.provider.clone())
                {
                    self.sequence += 1;
                    let id = self.sequence;
                    let cancel = CancellationToken::new();
                    self.active = Some(ActiveTurn {
                        id,
                        user: user.clone(),
                        answer: String::new(),
                        cancel: cancel.clone(),
                    });
                    self.error = None;
                    self.deleting = false;
                    self.composer = text_editor::Content::new();
                    task = Task::run(
                        turn::stream(store, provider, self.selected.clone(), user, cancel, id),
                        Event::Turn,
                    );
                } else {
                    self.error =
                        Some("Open Connection settings and save a connection first.".into());
                }
            }
            Event::Stop => {
                if let Some(active) = &self.active {
                    active.cancel.cancel();
                }
            }
            Event::Turn(event) => {
                if self.active.as_ref().map(|active| active.id) != Some(event.id()) {
                    return Task::none();
                }
                match event {
                    turn::TurnEvent::Started { session, .. } => self.selected = Some(session),
                    turn::TurnEvent::Delta { text, .. } => {
                        self.active.as_mut().unwrap().answer.push_str(&text);
                        task = iced::widget::operation::snap_to(
                            "conversation",
                            scrollable::RelativeOffset::END,
                        );
                    }
                    turn::TurnEvent::Done { result, .. } => {
                        let active = self.active.take().unwrap();
                        match result {
                            Ok(answer) => {
                                self.messages.push(ChatMessage {
                                    parts: vec![],
                                    role: Role::User,
                                    content: active.user,
                                });
                                self.messages.push(ChatMessage {
                                    parts: vec![],
                                    role: Role::Assistant,
                                    content: answer,
                                });
                                if self.messages.len() > dolores_core::HISTORY_LIMIT {
                                    self.messages
                                        .drain(..self.messages.len() - dolores_core::HISTORY_LIMIT);
                                }
                            }
                            Err(error) => {
                                self.composer = text_editor::Content::with_text(&active.user);
                                self.error = Some(error);
                            }
                        }
                        task = Task::batch([
                            self.refresh(),
                            iced::widget::operation::snap_to(
                                "conversation",
                                scrollable::RelativeOffset::END,
                            ),
                        ]);
                    }
                }
            }
            _ => {}
        }
        #[cfg(feature = "smoke")]
        {
            task = Task::batch([task, self.smoke_advance()]);
        }
        task
    }
    fn view(&self) -> Element<'_, Event> {
        let locked = self.locked();
        let mut sidebar = column![
            text("Dolores").size(30),
            text("One conversation at a time").size(13),
            space().height(16),
            button("New conversation")
                .on_press_maybe((!locked).then_some(Event::New))
                .width(Length::Fill),
            space().height(12),
            text("CONVERSATIONS").size(12)
        ]
        .spacing(8);
        let mut sessions = column![].spacing(5);
        for session in &self.sessions {
            sessions = sessions.push(
                button(text(&session.title).size(14))
                    .width(Length::Fill)
                    .style(if self.selected.as_deref() == Some(&session.id) {
                        button::primary
                    } else {
                        button::text
                    })
                    .on_press_maybe((!locked).then(|| Event::Select(session.id.clone()))),
            );
        }
        sidebar = sidebar
            .push(scrollable(sessions).height(Length::Fill))
            .push(
                button("Connection settings")
                    .style(button::secondary)
                    .width(Length::Fill)
                    .on_press_maybe((!locked).then_some(Event::Settings)),
            )
            .push(
                text(if self.provider.is_some() {
                    "Connection saved for this launch"
                } else {
                    "Save a connection to begin"
                })
                .size(12),
            );
        let sidebar = container(sidebar)
            .padding(20)
            .width(238)
            .height(Length::Fill)
            .style(sidebar_style);
        let content: Element<'_, Event> = if self.settings {
            self.settings_view()
        } else {
            let title = self
                .sessions
                .iter()
                .find(|s| Some(&s.id) == self.selected.as_ref())
                .map(|s| s.title.as_str())
                .unwrap_or("New conversation");
            let mut header = row![text(title).size(19), space().width(Length::Fill)]
                .spacing(12)
                .align_y(iced::Alignment::Center);
            if self.selected.is_some() {
                header = header.push(
                    button("Delete")
                        .style(button::text)
                        .on_press_maybe((!locked).then_some(Event::Delete)),
                );
            }
            let mut transcript = column![].spacing(20).width(Length::Fill);
            if self.messages.is_empty() && self.active.is_none() {
                transcript = transcript.push(space().height(50)).push(text("A place to think together.").size(30))
                    .push(text("Connect a model, then begin a conversation. Your completed conversations stay on this device.").size(16));
            }
            for message in &self.messages {
                transcript = transcript.push(message_view(&message.role, &message.content));
            }
            if let Some(active) = &self.active {
                transcript = transcript
                    .push(message_view(&Role::User, &active.user))
                    .push(message_view(
                        &Role::Assistant,
                        if active.answer.is_empty() {
                            "Thinking…"
                        } else {
                            &active.answer
                        },
                    ));
            }
            let mut main = column![header].spacing(16);
            if self.deleting {
                main = main.push(
                    row![
                        text("Delete this conversation permanently?"),
                        button("Delete conversation")
                            .style(button::danger)
                            .on_press(Event::ConfirmDelete)
                    ]
                    .spacing(12),
                );
            }
            main = main.push(
                scrollable(container(transcript).padding(12).width(Length::Fill))
                    .id("conversation")
                    .height(Length::Fill),
            );
            if let Some(error) = &self.error {
                main = main.push(text(error).size(14).style(text::danger));
            }
            let mut editor = text_editor(&self.composer)
                .placeholder("Write a message…")
                .height(100)
                .padding(12)
                .key_binding(|key| {
                    if matches!(key.status, text_editor::Status::Focused { .. })
                        && key.key == iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter)
                        && key.modifiers.command()
                    {
                        Some(text_editor::Binding::Custom(Event::Send))
                    } else {
                        text_editor::Binding::from_key_press(key)
                    }
                });
            if !locked {
                editor = editor.on_action(Event::Edit);
            }
            let send = if self.active.is_some() {
                button("Stop").style(button::danger).on_press(Event::Stop)
            } else {
                button("Send").on_press_maybe(
                    (!locked && !self.composer.text().trim().is_empty()).then_some(Event::Send),
                )
            };
            main = main.push(editor).push(
                row![
                    text("Ctrl / ⌘ + Enter to send · Enter for a new line")
                        .size(12)
                        .width(Length::Fill),
                    send
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            );
            container(main)
                .padding(24)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };
        if self.width >= 800.0 {
            row![sidebar, content].height(Length::Fill).into()
        } else {
            let choices: Vec<_> = self
                .sessions
                .iter()
                .map(|s| SessionChoice {
                    id: s.id.clone(),
                    title: s.title.clone(),
                })
                .collect();
            let selected = choices
                .iter()
                .find(|s| Some(&s.id) == self.selected.as_ref())
                .cloned();
            let mut picker = pick_list(choices, selected, |choice| Event::Select(choice.id))
                .placeholder("Saved conversations")
                .width(Length::Fill);
            if locked {
                picker = pick_list(
                    Vec::<SessionChoice>::new(),
                    None::<SessionChoice>,
                    |choice| Event::Select(choice.id),
                )
                .placeholder("Conversation in progress")
                .width(Length::Fill);
            }
            let compact = column![
                row![
                    text("Dolores").size(23).width(Length::Fill),
                    button("New").on_press_maybe((!locked).then_some(Event::New)),
                    button("Connection")
                        .style(button::secondary)
                        .on_press_maybe((!locked).then_some(Event::Settings))
                ]
                .spacing(8),
                picker
            ]
            .spacing(8);
            column![
                container(compact)
                    .padding(12)
                    .width(Length::Fill)
                    .style(sidebar_style),
                content
            ]
            .height(Length::Fill)
            .into()
        }
    }
    fn settings_view(&self) -> Element<'_, Event> {
        let enabled = !self.busy;
        let mut form = column![text("Connection settings").size(25), text("Connect to an OpenAI-compatible text model.").size(15), space().height(12), text("API base URL"),
            text_input("https://provider.example/v1", &self.base_url).on_input_maybe(enabled.then_some(Event::BaseUrl as fn(_) -> _)),
            text("Model ID"), text_input("Exact model ID", &self.model).on_input_maybe(enabled.then_some(Event::Model as fn(_) -> _)),
            text("API key · optional for local servers"), text_input("Kept only for this launch", &self.key).secure(true).on_input_maybe(enabled.then_some(Event::Key as fn(_) -> _)),
            text("Keys stay in process memory. Save your connection again after restarting. Preferences and conversation text are stored locally without encryption.").size(13)].spacing(12).max_width(620);
        if let Some(error) = &self.error {
            form = form.push(text(error).style(text::danger));
        }
        form = form.push(
            row![
                button("Save connection").on_press_maybe(enabled.then_some(Event::Save)),
                button("Back")
                    .style(button::secondary)
                    .on_press_maybe(enabled.then_some(Event::CloseSettings))
            ]
            .spacing(12),
        );
        container(scrollable(form))
            .padding(32)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
fn message_view<'a>(role: &Role, content: &'a str) -> Element<'a, Event> {
    let name = match role {
        Role::User => "YOU",
        Role::Assistant => "DOLORES",
        Role::System => "SYSTEM",
    };
    column![
        row![
            text(name).size(12).width(Length::Fill),
            button(text("Copy").size(12))
                .style(button::text)
                .on_press(Event::Copy(content.to_owned()))
        ]
        .align_y(iced::Alignment::Center),
        text(content).size(16)
    ]
    .spacing(6)
    .into()
}
fn sidebar_style(theme: &Theme) -> container::Style {
    let mut color = theme.palette().background;
    let adjustment = if color.r < 0.5 { 0.035 } else { -0.025 };
    color.r += adjustment;
    color.g += adjustment;
    color.b += adjustment;
    container::Style {
        background: Some(color.into()),
        ..Default::default()
    }
}
