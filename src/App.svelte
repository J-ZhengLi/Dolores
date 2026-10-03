<script lang="ts">
  import { onMount, tick } from 'svelte';
  import {
    bridge,
    desktop,
    type Session,
    type Message,
    type Preferences,
  } from './lib/bridge';

  let sessions = $state<Session[]>([]);
  let current = $state<string | null>(null);
  let messages = $state<Message[]>([]);
  let draft = $state('');
  let answer = $state('');
  let pendingInput = $state('');
  let runId = $state<string | null>(null);
  let busy = $state(false);
  let stopRequested = false;
  let loading = $state(true);
  let switching = $state(false);
  let connected = $state(false);
  let error = $state('');
  let preferences = $state<Preferences>({
    baseUrl: 'http://localhost:11434/v1',
    model: '',
  });
  let endpoint = $state('');
  let model = $state('');
  let apiKey = $state('');
  let settingsError = $state('');
  let saving = $state(false);
  let settingsDialog: HTMLDialogElement;
  let composer: HTMLTextAreaElement;
  let conversation: HTMLDivElement;
  let sessionVersion = 0;
  let scrollQueued = false;

  const title = $derived(
    sessions.find((session) => session.id === current)?.title ??
      'A new beginning',
  );

  function explain(cause: unknown): string {
    return cause instanceof Error ? cause.message : String(cause);
  }
  async function refresh() {
    const data = await bridge.bootstrap();
    sessions = data.sessions;
    preferences = data.preferences;
    connected = data.connected;
  }
  onMount(() => {
    void (async () => {
      try {
        await refresh();
        if (sessions.length) await selectSession(sessions[0].id);
      } catch (cause) {
        error = explain(cause);
      } finally {
        loading = false;
        await tick();
        performance.mark('dolores-interactive');
      }
    })();
  });

  async function selectSession(id: string) {
    if (busy || switching) return;
    const version = ++sessionVersion;
    switching = true;
    error = '';
    try {
      const history = await bridge.messages(id);
      if (version !== sessionVersion) return;
      current = id;
      messages = history;
      answer = '';
      pendingInput = '';
      draft = '';
      await tick();
      composer?.focus();
      await scrollDown();
    } catch (cause) {
      error = explain(cause);
    } finally {
      switching = false;
    }
  }
  async function newSession() {
    if (busy || switching) return;
    ++sessionVersion;
    current = null;
    messages = [];
    draft = '';
    answer = '';
    pendingInput = '';
    error = '';
    await tick();
    composer?.focus();
  }
  async function removeSession() {
    if (!current || busy || switching) return;
    switching = true;
    try {
      await bridge.deleteSession(current);
      current = null;
      messages = [];
      draft = '';
      answer = '';
      pendingInput = '';
      error = '';
      await refresh();
    } catch (cause) {
      error = explain(cause);
    } finally {
      switching = false;
    }
  }
  function openSettings() {
    endpoint = preferences.baseUrl;
    model = preferences.model;
    apiKey = '';
    settingsError = '';
    settingsDialog.showModal();
  }
  async function saveConnection(event: SubmitEvent) {
    event.preventDefault();
    saving = true;
    settingsError = '';
    try {
      await bridge.configure(
        { baseUrl: endpoint.trim(), model: model.trim() },
        apiKey.trim(),
      );
      apiKey = '';
      await refresh();
      settingsDialog.close();
      composer?.focus();
    } catch (cause) {
      settingsError = explain(cause);
    } finally {
      saving = false;
    }
  }
  async function scrollDown() {
    await tick();
    if (conversation) conversation.scrollTop = conversation.scrollHeight;
  }
  function scheduleScroll() {
    if (
      scrollQueued ||
      !conversation ||
      conversation.scrollHeight -
        conversation.scrollTop -
        conversation.clientHeight >
        80
    )
      return;
    scrollQueued = true;
    requestAnimationFrame(() => {
      scrollQueued = false;
      void scrollDown();
    });
  }
  async function send() {
    if (busy || switching || loading || !draft.trim()) return;
    if (!connected) {
      openSettings();
      return;
    }
    const input = draft.trim();
    if (new TextEncoder().encode(input).length > 16 * 1024) {
      error = 'Message exceeds the 16 KiB limit.';
      return;
    }
    busy = true;
    stopRequested = false;
    error = '';
    pendingInput = input;
    answer = '';
    const id = crypto.randomUUID();
    runId = id;
    let committed = false;
    let cancellationForwarded = false;
    try {
      if (!current) {
        const session = await bridge.createSession();
        current = session.id;
        sessions = [session, ...sessions];
      }
      const sessionId = current;
      draft = '';
      await scrollDown();
      if (stopRequested)
        throw new Error('Response stopped. Your message was not saved.');
      await bridge.generate(sessionId, id, input, (event) => {
        if (runId !== event.runId) return;
        if (stopRequested && !cancellationForwarded) {
          cancellationForwarded = true;
          void bridge.cancel(id).catch((cause) => {
            if (runId === id) error = explain(cause);
          });
        }
        answer += event.delta;
        scheduleScroll();
      });
      committed = true;
      runId = null;
      messages = [
        ...messages,
        { role: 'user' as const, content: input },
        { role: 'assistant' as const, content: answer },
      ].slice(-80);
      pendingInput = '';
      answer = '';
      messages = await bridge.messages(sessionId);
      await refresh();
    } catch (cause) {
      if (committed) {
        error =
          'Your response was saved, but the workspace could not refresh. ' +
          explain(cause);
      } else {
        error = explain(cause);
        draft = input;
      }
    } finally {
      busy = false;
      runId = null;
      await scrollDown();
      composer?.focus();
    }
  }
  async function stop() {
    if (runId) {
      stopRequested = true;
      try {
        await bridge.cancel(runId);
      } catch (cause) {
        error = explain(cause);
      }
    }
  }
  function composerKey(event: KeyboardEvent) {
    if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
      event.preventDefault();
      void send();
    }
  }
  function suggestion(text: string) {
    draft = text;
    composer?.focus();
  }
</script>

<div class="workspace">
  <aside class="sidebar" aria-label="Conversations">
    <div class="brand">
      <span class="brand-mark" aria-hidden="true"><img src="/dolores.svg" alt="" width="28" height="28" /></span><span
        >Dolores<small>A little better, each time.</small></span
      >
    </div>
    <button
      class="new-button"
      onclick={newSession}
      disabled={busy || loading || switching}
      ><span aria-hidden="true">＋</span> New conversation <kbd>↗</kbd></button
    >
    <div class="section-label">
      YOUR CONVERSATIONS <span>{sessions.length}</span>
    </div>
    <nav class="sessions" aria-label="Saved conversations">
      {#each sessions as session (session.id)}
        <button
          class:active={current === session.id}
          onclick={() => selectSession(session.id)}
          disabled={busy || switching || loading}
          ><span class="session-dot" aria-hidden="true"></span><span
            >{session.title}</span
          ></button
        >
      {:else}
        <p class="empty-sidebar">
          Space for your next idea.<br />Your conversations will appear here.
        </p>
      {/each}
    </nav>
    <div class="sidebar-footer">
      <div class="local-note">
        <span class="status-dot" class:ready={connected}></span>{desktop
          ? 'Conversations saved locally'
          : 'Browser preview · no model connection'}
      </div>
      <button
        class="settings-button"
        onclick={openSettings}
        disabled={busy || loading || switching}
        ><span aria-hidden="true">⚙</span><span
          >Connection settings<small
            >{connected ? preferences.model : 'Choose your model'}</small
          ></span
        ><span aria-hidden="true">›</span></button
      >
    </div>
  </aside>

  <main>
    <header class="topbar">
      <div>
        <span class="eyebrow">CONVERSATION</span>
        <h1>{title}</h1>
        {#if sessions.length}
          <select
            class="mobile-sessions"
            aria-label="Saved conversation"
            value={current ?? ''}
            disabled={busy || switching || loading}
            onchange={(event) =>
              event.currentTarget.value
                ? selectSession(event.currentTarget.value)
                : newSession()}
          >
            <option value="">New conversation</option>
            {#each sessions as session (session.id)}<option value={session.id}
                >{session.title}</option
              >{/each}
          </select>
        {/if}
      </div>
      <div class="header-actions">
        <span class="model-badge"
          ><span class="status-dot" class:ready={connected}></span>{connected
            ? preferences.model
            : 'No model connected'}</span
        >
        {#if current}<button
            class="text-button"
            onclick={removeSession}
            disabled={busy || switching}
            aria-label="Delete conversation">Delete</button
          >{/if}
      </div>
    </header>

    <div
      class="conversation"
      bind:this={conversation}
      role="log"
      aria-label="Chat messages"
      aria-busy={busy}
      aria-live="off"
    >
      {#if loading}
        <p class="loading">Opening your workspace…</p>
      {:else if !messages.length && !pendingInput}
        <section class="welcome">
          <div class="welcome-symbol" aria-hidden="true">✳</div>
          <p class="eyebrow">A NEW BEGINNING</p>
          <h2>What’s on your mind?</h2>
          <p class="welcome-description">
            A quiet space to think, explore, and make things. <br />Start a
            conversation. We’ll build from there.
          </p>
          <div class="suggestions">
            <button
              onclick={() =>
                suggestion('Help me turn a rough idea into a clear plan.')}
              ><span aria-hidden="true">↗</span><strong
                >Find a starting point</strong
              ><small>Turn an idea into a plan</small></button
            >
            <button
              onclick={() =>
                suggestion(
                  'Help me understand a topic. Ask me what I want to explore.',
                )}
              ><span aria-hidden="true">◎</span><strong
                >Explore something</strong
              ><small>Make room for curiosity</small></button
            >
            <button
              onclick={() =>
                suggestion(
                  'Help me think through a decision. Ask me about the options.',
                )}
              ><span aria-hidden="true">◇</span><strong>Think it through</strong
              ><small>See a decision more clearly</small></button
            >
          </div>
        </section>
      {/if}
      <div class="message-list">
        {#each messages as message, index (index)}
          <article class="message" class:user={message.role === 'user'}>
            <div class="avatar" aria-hidden="true">
              {#if message.role === 'user'}Y{:else}<img src="/dolores.svg" alt="" width="28" height="28" />{/if}
            </div>
            <div class="message-body">
              <span class="message-author"
                >{message.role === 'user' ? 'You' : 'Dolores'}</span
              >
              <div class="message-content">{message.content}</div>
            </div>
          </article>
        {/each}
        {#if pendingInput}
          <article class="message user">
            <div class="avatar" aria-hidden="true">Y</div>
            <div class="message-body">
              <span class="message-author"
                >You <small>{busy ? '' : 'Unsaved'}</small></span
              >
              <div class="message-content">{pendingInput}</div>
            </div>
          </article>
          <article class="message">
            <div class="avatar" aria-hidden="true"><img src="/dolores.svg" alt="" width="28" height="28" /></div>
            <div class="message-body">
              <span class="message-author"
                >Dolores <small
                  >{busy ? 'Responding…' : 'Incomplete · unsaved'}</small
                ></span
              >
              <div class="message-content">
                {answer || (busy ? 'Thinking…' : 'No complete response.')}
              </div>
            </div>
          </article>
        {/if}
      </div>
    </div>

    <div class="composer-area">
      {#if error}<div class="error-banner" role="alert">{error}</div>{/if}
      {#if !desktop}<p class="preview-note">
          UI preview. Run the desktop app to chat and save conversations.
        </p>{/if}
      <form
        class="composer"
        onsubmit={(event) => {
          event.preventDefault();
          void send();
        }}
      >
        <label class="sr-only" for="message">Message Dolores</label>
        <textarea
          id="message"
          bind:this={composer}
          bind:value={draft}
          onkeydown={composerKey}
          disabled={busy || loading || switching}
          placeholder="Message Dolores…"
          rows="3"></textarea>
        <div class="composer-bottom">
          <span
            >{connected
              ? 'OpenAI-compatible'
              : 'Connect a model to begin'}</span
          >{#if busy}<button
              type="button"
              class="send-button stop-button"
              onclick={stop}>■ Stop</button
            >{:else}<button
              class="send-button"
              type="submit"
              disabled={!draft.trim() || loading || switching}
              aria-label="Send message">↑</button
            >{/if}
        </div>
      </form>
      <div class="composer-hint">
        <span
          >Enter to send <span aria-hidden="true">·</span> Shift + Enter for a new
          line</span
        ><span role="status" aria-live="polite"
          >{busy ? 'Dolores is responding' : 'Take your time.'}</span
        >
      </div>
    </div>
  </main>
</div>

<dialog
  bind:this={settingsDialog}
  onclose={() => {
    apiKey = '';
  }}
>
  <form class="connection-form" onsubmit={saveConnection}>
    <div class="dialog-header">
      <div>
        <p class="eyebrow">MAKE A CONNECTION</p>
        <h2>Your model, your choice.</h2>
      </div>
      <button
        type="button"
        class="close-button"
        onclick={() => settingsDialog.close()}
        disabled={saving}
        aria-label="Close settings">×</button
      >
    </div>
    <p>Use a local model server or a hosted OpenAI-compatible API.</p>
    <label for="endpoint">Base URL</label><input
      id="endpoint"
      type="url"
      required
      bind:value={endpoint}
      placeholder="http://localhost:11434/v1"
    />
    <small
      >Include the API prefix, such as /v1. HTTPS, or HTTP on this computer.</small
    >
    <label for="model">Model ID</label><input
      id="model"
      required
      maxlength="200"
      bind:value={model}
      placeholder="e.g. llama3.2"
    />
    <label for="api-key"
      >API key <span class="optional">optional for local servers</span></label
    ><input
      id="api-key"
      type="password"
      autocomplete="off"
      bind:value={apiKey}
      placeholder="Used for this app session only"
      disabled={!desktop}
    />
    <small
      >The key is kept in memory. Re-enter it after restarting. Saving settings
      does not test the connection.</small
    >
    {#if !desktop}<p class="preview-note">
        Preview settings are temporary. Model connections work in the desktop
        app.
      </p>{/if}
    {#if settingsError}<p class="error-banner" role="alert">
        {settingsError}
      </p>{/if}
    <button class="primary-button" disabled={saving || busy}
      >{saving
        ? 'Saving…'
        : desktop
          ? 'Save connection'
          : 'Save preview settings'}</button
    >
  </form>
</dialog>
