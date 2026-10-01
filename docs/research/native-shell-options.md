# Native shell options for Dolores brick 1.1

Checked primary sources on 2026-10-01. Recommendation: build the first comparison with **Iced 0.14.0 using tiny-skia only**, preserving the existing Rust core/provider/SQLite implementations. This is a testable architecture preference, not a memory or performance result. Keep Tauri until a comparable release build passes functionality and measured resource gates.

Iced's latest published stable-version documentation is [0.14.0](https://docs.rs/iced/0.14.0/iced/); eframe's is [0.36.2](https://docs.rs/eframe/0.36.2/eframe/). Iced still describes the library as experimental. Its manifest declares Rust 1.88 minimum and edition 2024. [Pinned Iced manifest](https://raw.githubusercontent.com/iced-rs/iced/0.14.0/Cargo.toml)

## Comparison

| Question | Iced 0.14.0 + tiny-skia | egui/eframe 0.36.2 + glow |
| --- | --- | --- |
| Desktop platforms | Windows, macOS, Linux; enable X11 and Wayland features for Linux. [Iced repository](https://github.com/iced-rs/iced) | Native desktop via eframe/winit; Windows/macOS/Linux integration and optional X11/Wayland. [eframe](https://docs.rs/eframe/0.36.2/eframe/), [features](https://docs.rs/crate/eframe/0.36.2/features) |
| Rendering | tiny-skia is a software rasterizer. With wgpu disabled it is the selected renderer, not a failed-GPU fallback. Enabling both builds the GPU/software fallback path. [Renderer selection source](https://raw.githubusercontent.com/iced-rs/iced/0.14.0/renderer/src/lib.rs) | glow uses OpenGL through glutin. A graphics-context creation failure can fail startup; no guaranteed CPU rasterizer is supplied by choosing glow. [run_native](https://docs.rs/eframe/0.36.2/eframe/fn.run_native.html), [features](https://docs.rs/crate/eframe/0.36.2/features) |
| Idle behavior | Event loop waits or waits until a scheduled redraw; no need for a timer to consume async messages. Avoid `unconditional-rendering`, animations, or recurring subscriptions in the baseline. [Iced event loop](https://raw.githubusercontent.com/iced-rs/iced/0.14.0/winit/src/lib.rs) | Immediate-mode UI need not imply a continuous render loop. `App::ui` runs when repainting is needed; background tasks call `Context::request_repaint`. `App::logic` also runs while hidden on explicit repaint requests. [App API](https://docs.rs/eframe/0.36.2/eframe/trait.App.html) |
| Async chat | Tokio executor feature plus `Task::run` for a stream; messages update retained state. [Task API](https://docs.rs/iced/0.14.0/iced/struct.Task.html) | Own a Tokio runtime; send typed events through a bounded channel; request repaint after publishing events; drain on the UI thread. This is an integration proposal based on [App API](https://docs.rs/eframe/0.36.2/eframe/trait.App.html). |
| IME | 0.14 changelog includes input-method support and candidate/preedit fixes. Chinese composition must be exercised on each target OS. [Changelog](https://github.com/iced-rs/iced/blob/0.14.0/CHANGELOG.md) | winit integration maps IME input and composition and includes Windows filtering. This proves a code path exists, not every IME works. [egui-winit source](https://raw.githubusercontent.com/emilk/egui/0.36.2/crates/egui-winit/src/lib.rs) |
| Accessibility | Upstream accessibility implementation issue remains open; 0.14 has no documented AccessKit feature. Treat screen-reader semantics as an unresolved gap. [Issue 552](https://github.com/iced-rs/iced/issues/552), [Iced features](https://docs.rs/crate/iced/0.14.0/features) | Explicit `accesskit` feature connects egui-winit's adapter. Custom widgets still need meaningful semantics and live NVDA/VoiceOver/Orca verification. [eframe features](https://docs.rs/crate/eframe/0.36.2/features), [integration source](https://raw.githubusercontent.com/emilk/egui/0.36.2/crates/egui-winit/src/lib.rs) |
| System theme | Runtime receives native/system theme changes; Linux theme detection is a separate feature. Default theme logic can follow detected light/dark; explicit `.theme(...)` overrides it. [Runtime](https://raw.githubusercontent.com/iced-rs/iced/0.14.0/winit/src/lib.rs), [base theme](https://raw.githubusercontent.com/iced-rs/iced/0.14.0/core/src/theme.rs) | `ctx.set_theme(egui::ThemePreference::System)` selects the OS preference; use current Context APIs, not outdated NativeOptions theme fields. [Context API](https://docs.rs/egui/0.36.2/egui/struct.Context.html#method.set_theme) |
| Fonts / CJK | Uses cosmic-text font system. Default `Shaping::Auto` chooses Basic for ASCII and Advanced with fallback otherwise; Basic explicitly has no fallback. Available system fonts determine coverage. [Text system](https://raw.githubusercontent.com/iced-rs/iced/0.14.0/graphics/src/text.rs), [shaping](https://raw.githubusercontent.com/iced-rs/iced/0.14.0/core/src/text.rs) | Application supplies font bytes and ordered family fallback through `FontDefinitions` and `ctx.set_fonts`. Do not assume the bundled fonts cover CJK or automatically discover OS fonts. [FontDefinitions](https://docs.rs/egui/0.36.2/egui/struct.FontDefinitions.html) |

Iced is the closest match to the requested software-rendered comparison. Its accessibility gap is material; a measured resource win alone is insufficient to promote it to the universal UI. Eframe becomes the stronger alternative if assistive-technology support must be present immediately, provided target machines create a working OpenGL context.

## Exact dependencies and entry points

Iced baseline, with [feature metadata](https://docs.rs/crate/iced/0.14.0/features):

```toml
[dependencies]
iced = { version = "=0.14.0", default-features = false, features = [
  "tiny-skia", "tokio", "x11", "wayland", "linux-theme-detection", "crisp"
] }
tokio = { version = "1", features = ["sync", "time"] }
```

Do not add wgpu, images, SVG, Markdown, developer tooling, or an unconditional-rendering feature to the first resource comparison unless the comparable behavior needs them. No global `advanced-shaping` flag is necessary for the default Auto policy. Add a pinned font via `.font(bytes)` when predictable glyph coverage is required; validate licensing and measure startup/font memory rather than bundling all system fonts. [Application font API](https://docs.rs/iced/0.14.0/iced/application/struct.Application.html#method.font)

The Iced 0.14 entry is `iced::application(boot, update, view).title("Dolores").run()`. The first argument is the boot function, **not the title** used by some older examples. [application API](https://docs.rs/iced/0.14.0/iced/fn.application.html)

Eframe glow alternative, with defaults disabled so wgpu is not silently included:

```toml
[dependencies]
eframe = { version = "=0.36.2", default-features = false, features = [
  "glow", "default_fonts", "accesskit", "x11", "wayland"
] }
tokio = { version = "1", features = ["rt-multi-thread", "sync", "time"] }
```

```rust
use eframe::egui;

#[derive(Default)]
struct Shell;

impl eframe::App for Shell {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.heading("Dolores");
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native("Dolores", options, Box::new(|cc| {
        cc.egui_ctx.set_theme(egui::ThemePreference::System);
        Ok(Box::new(Shell))
    }))
}
```

Current eframe requires `App::ui(&mut self, &mut egui::Ui, &mut Frame)`; old `App::update(ctx, frame)` examples do not match 0.36.2. [App API](https://docs.rs/eframe/0.36.2/eframe/trait.App.html), [launch API](https://docs.rs/eframe/0.36.2/eframe/fn.run_native.html)

## Iced stream bridge example

This independently written sample demonstrates the current APIs, not a production provider. Replace the timed source with the existing provider stream and storage coordinator; retain typed request/session IDs and explicit completion/error/cancellation events. Documentation sample has not been separately compiled in this research pass.

```rust
use iced::futures::SinkExt;
use iced::widget::{button, column, row, scrollable, text, text_input};
use iced::{Element, Task};

#[derive(Default)]
struct Chat {
    draft: String,
    reply: String,
    running: bool,
    generation: u64,
    cancel: Option<tokio::sync::oneshot::Sender<()>>,
}

#[derive(Debug, Clone)]
enum Message {
    Edited(String),
    Send,
    Stop,
    Chunk(u64, String),
    Finished(u64),
}

impl Chat {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Edited(value) => self.draft = value,
            Message::Send if !self.running && !self.draft.trim().is_empty() => {
                self.generation += 1;
                let id = self.generation;
                self.running = true;
                self.reply.clear();
                let (cancel_tx, mut cancel_rx) = tokio::sync::oneshot::channel();
                self.cancel = Some(cancel_tx);
                let stream = iced::stream::channel(64, async move |mut output| {
                    // Replace this loop with the shared backend's event source.
                    for piece in ["Hello", ", ", "Dolores", "."] {
                        tokio::select! {
                            _ = &mut cancel_rx => break,
                            _ = tokio::time::sleep(std::time::Duration::from_millis(80)) => {}
                        }
                        if output.send(Message::Chunk(id, piece.into())).await.is_err() {
                            return;
                        }
                    }
                    let _ = output.send(Message::Finished(id)).await;
                });
                return Task::run(stream, |message| message);
            }
            Message::Stop => {
                if let Some(cancel) = self.cancel.take() {
                    let _ = cancel.send(());
                }
            }
            Message::Chunk(id, piece) if id == self.generation => {
                self.reply.push_str(&piece);
            }
            Message::Finished(id) if id == self.generation => {
                self.running = false;
                self.cancel = None;
            }
            _ => {}
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        column![
            text("Dolores").size(24),
            scrollable(text(&self.reply)).height(iced::Fill),
            text_input("Message Dolores", &self.draft).on_input(Message::Edited),
            row![
                button("Send").on_press_maybe((!self.running).then_some(Message::Send)),
                button("Stop").on_press_maybe(self.running.then_some(Message::Stop)),
            ].spacing(8),
        ].spacing(12).padding(20).into()
    }
}

fn main() -> iced::Result {
    iced::application(Chat::default, Chat::update, Chat::view)
        .title("Dolores")
        .window_size((960.0, 720.0))
        .run()
}
```

The bounded helper runs a producer future alongside its receiver; dropping its stream drops the producer future. [Stream helper source](https://raw.githubusercontent.com/iced-rs/iced/0.14.0/futures/src/stream.rs) `Task::run` maps each stream item; `Task::abortable` is available when a UI-owned task handle is useful. [Task API](https://docs.rs/iced/0.14.0/iced/struct.Task.html)

For Dolores integration, cancellation must flow through the existing core mechanism so the coordinator can report an unsaved turn and restore the draft. The current SQLite store saves only completed pairs; it does not record cancelled responses. Merely aborting a UI task can skip the terminal notification. Spawn blocking SQLite work off the UI thread, preserve the single-run guard and avoid holding state locks while awaiting network work. These are design recommendations, not new core API claims.

For eframe integration, own the Tokio runtime for the application's lifetime, publish into a bounded `tokio::sync::mpsc` queue, call cloned `egui::Context::request_repaint` after a successful send, and drain with a per-frame bound. Receive in `App::logic` if persistence must advance while hidden. This avoids a permanent polling timer and keeps all widget mutations on the UI thread. Keep server streaming and storage operations independent of the repaint loop.

## Acceptance before a shell decision

Use the same provider/model, data directory policy, prompt/context limits, message retention, and generation behavior as Tauri. The native shell must expose provider setup, new/open/history sessions, token streaming, errors, Stop, cancelled-history persistence, and restart recovery before treating resource figures as comparable.

Test English and Chinese text, IME preedit/candidate positioning, mixed CJK/Latin wrapping, copy/paste, selectable transcripts, multiline composition, keyboard navigation, scaling, system-theme changes, and font absence. Record the screen-reader limitation explicitly rather than declaring general accessibility.

Measure uninstrumented release startup, active idle, minimized idle, streaming CPU/memory, and a long transcript. Report renderer, machine, OS, window size/DPI, resident working set, private allocation, and cold/warm startup separately. A software renderer can move costs to CPU and frame buffers; no memory estimates or guaranteed budget compliance are asserted here.
