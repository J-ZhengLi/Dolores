# Bounded experimental windows — milestone 19

## Frozen trial (2026-10-07)

Try only the maintained Flutter 3.47.5 stable SDK's experimental same-engine
`RegularWindowController` / `RegularWindow` backend on this Windows host.
The feature flag is enabled only in an explicit disposable diagnostic entry.
No SDK edit, channel migration, plugin comparison or production internal import.
The trial allows at most two repair iterations; unsupported engine symbols are
not repaired by rebuilding/replacing the Flutter engine.

One NativeBridge and AppHost own the profile and all work. The first window must
remain useful if window creation fails. Probe two native windows, theme/focus/
close and shared owner identity. An additional empty window must add no more than
128 MiB private memory and less than 1% idle CPU of one core. Baseline variability
and physical input results must be labeled rather than inferred from widgets.

Before adopting this backend, require native focus/input/close/theme evidence,
acknowledged transfers without duplicate document or PTY owners and reliable
failed-receive recovery. A failed gate keeps single-window pages and a truthful
Experimental availability reason. The saved default-On preference and explicit
Off values remain independent of capability.

## Dependent work

**Recorded trial disposition:** corrected creation/close works with one host, but
the measured empty secondary adds 1.222% idle CPU, above the frozen less-than-1%
gate. Backend held; use the [qualification](../qualification/experimental-windows.md).
19.2/19.3 remain dependent held work; 19.4 delivers the explicit fallback.

19.2 depends on a qualified backend: file/diff/terminal group receive ACK precedes
source disposal; canceled or failed receipt retains the source. Home chat tabs do
not detach. 19.3 then adds versioned accessible bounds/layout, missing-monitor
clamping, stopped cold shell recovery and acknowledged rejoin when switched Off.
Failed rejoin retains the original window and work. No separate profile host.

19.4 may exit with a supported capability or a held backend, as specified in
[ROADMAP](../ROADMAP.md). Held 19.2/19.3 are not reported as implemented. Preserve
the supported single-window editor, Source Control, terminals and Home; expose the
reason in Settings without repeatedly attempting unavailable initialization.

Upstream [windowing announcement](https://flutter.dev/blog/desktop-windowing-apis)
is a reference, not evidence that this maintained engine exports every required
symbol. Current SDK source labels these APIs internal and prohibits production
imports; that support constraint must also be recorded at adoption.
