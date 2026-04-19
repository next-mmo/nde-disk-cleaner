<script lang="ts">
  import { checkForUpdates, type UpdateInfo } from "./ipc";
  import { openUrl } from "@tauri-apps/plugin-opener";

  interface Props {
    open: boolean;
    allowPermanentDelete: boolean;
    onclose: () => void;
    ontogglePermanent: (v: boolean) => void;
  }

  let { open, allowPermanentDelete, onclose, ontogglePermanent }: Props = $props();

  let updateStatus = $state<"idle" | "checking" | "done" | "error">("idle");
  let updateInfo = $state<UpdateInfo | null>(null);
  let updateError = $state<string | null>(null);

  async function doCheckUpdate() {
    updateStatus = "checking";
    updateError = null;
    try {
      updateInfo = await checkForUpdates();
      updateStatus = "done";
    } catch (e: any) {
      updateError = String(e);
      updateStatus = "error";
    }
  }

  function openRelease() {
    if (updateInfo?.release_url) {
      openUrl(updateInfo.release_url);
    }
  }

  function handleOverlayClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains("overlay")) {
      onclose();
    }
  }

  function handleToggle(e: Event) {
    const target = e.target as HTMLInputElement;
    if (target.checked) {
      const ok = confirm(
        "⚠️ Permanent deletion CANNOT be undone.\n\nFiles will be removed from disk immediately — they will NOT go to Trash.\n\nAre you sure you want to enable this?",
      );
      if (!ok) {
        target.checked = false;
        return;
      }
    }
    ontogglePermanent(target.checked);
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="overlay" onclick={handleOverlayClick}>
    <div class="panel">
      <header class="panel-header">
        <h2>Settings</h2>
        <button class="close-btn" onclick={onclose} aria-label="Close settings">✕</button>
      </header>

      <div class="sections scroll">
        <!-- ── Deletion ─────────────────────────── -->
        <section class="section">
          <div class="section-icon">🗑️</div>
          <div class="section-body">
            <h3>Deletion Mode</h3>
            <p class="desc">
              When enabled, the "Delete" action will <strong>permanently remove</strong> files
              from disk instead of moving them to the Trash. This cannot be undone.
            </p>
            <label class="switch-row">
              <span class="switch-label">Allow permanent deletion</span>
              <span class="switch">
                <input
                  type="checkbox"
                  checked={allowPermanentDelete}
                  onchange={handleToggle}
                />
                <span class="slider"></span>
              </span>
            </label>
            {#if allowPermanentDelete}
              <div class="warning-banner">
                <span class="warning-icon">⚠️</span>
                <span>Permanent deletion is <strong>active</strong>. Deleted files cannot be recovered.</span>
              </div>
            {/if}
          </div>
        </section>

        <!-- ── Updates ─────────────────────────── -->
        <section class="section">
          <div class="section-icon">🔄</div>
          <div class="section-body">
            <h3>Updates</h3>
            <p class="desc">
              Check if a newer release is available on GitHub.
            </p>

            <button
              class="update-btn"
              onclick={doCheckUpdate}
              disabled={updateStatus === "checking"}
            >
              {#if updateStatus === "checking"}
                <span class="spinner"></span> Checking…
              {:else}
                Check for Updates
              {/if}
            </button>

            {#if updateStatus === "done" && updateInfo}
              <div class="update-result" class:has-update={updateInfo.has_update}>
                <div class="version-row">
                  <span class="ver-label">Current</span>
                  <span class="ver-value">v{updateInfo.current_version}</span>
                </div>
                <div class="version-row">
                  <span class="ver-label">Latest</span>
                  <span class="ver-value">v{updateInfo.latest_version}</span>
                </div>
                {#if updateInfo.has_update}
                  <div class="update-cta">
                    <span class="update-badge">New version available!</span>
                    <button class="primary" onclick={openRelease}>
                      Download v{updateInfo.latest_version}
                    </button>
                  </div>
                  {#if updateInfo.release_notes}
                    <details class="release-notes">
                      <summary>Release notes</summary>
                      <pre>{updateInfo.release_notes}</pre>
                    </details>
                  {/if}
                {:else}
                  <div class="up-to-date">✅ You're up to date!</div>
                {/if}
              </div>
            {/if}

            {#if updateStatus === "error"}
              <div class="update-error">
                ❌ {updateError}
              </div>
            {/if}
          </div>
        </section>
      </div>

      <footer class="panel-footer">
        <span class="footer-text">NDE Disk Cleaner</span>
      </footer>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 999;
    background: rgba(0, 0, 0, 0.55);
    backdrop-filter: blur(6px);
    display: flex;
    justify-content: flex-end;
    animation: fadeIn 150ms ease;
  }
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  @keyframes slideIn {
    from { transform: translateX(100%); }
    to { transform: translateX(0); }
  }

  .panel {
    width: 420px;
    max-width: 90vw;
    height: 100%;
    background: var(--bg-elev);
    border-left: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    animation: slideIn 200ms cubic-bezier(0.22, 1, 0.36, 1);
    box-shadow: -12px 0 40px rgba(0, 0, 0, 0.4);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 18px 20px 14px;
    border-bottom: 1px solid var(--border);
  }
  .panel-header h2 {
    margin: 0;
    font-size: 16px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .close-btn {
    border: none;
    background: transparent;
    color: var(--fg-dim);
    font-size: 16px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 6px;
    transition: background 80ms, color 80ms;
  }
  .close-btn:hover {
    background: var(--bg-panel);
    color: var(--fg);
  }

  .sections {
    flex: 1;
    overflow-y: auto;
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .section {
    display: flex;
    gap: 14px;
    padding: 16px;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: 10px;
  }
  .section-icon {
    font-size: 22px;
    flex-shrink: 0;
    margin-top: 2px;
  }
  .section-body {
    flex: 1;
    min-width: 0;
  }
  .section-body h3 {
    margin: 0 0 6px;
    font-size: 14px;
    font-weight: 600;
    color: var(--fg);
  }
  .desc {
    margin: 0 0 12px;
    font-size: 12px;
    color: var(--fg-dim);
    line-height: 1.5;
  }

  /* Toggle switch */
  .switch-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    cursor: pointer;
  }
  .switch-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--fg);
  }
  .switch {
    position: relative;
    width: 42px;
    height: 24px;
    flex-shrink: 0;
  }
  .switch input {
    opacity: 0;
    width: 0;
    height: 0;
    position: absolute;
  }
  .slider {
    position: absolute;
    cursor: pointer;
    inset: 0;
    background: #2a2e3e;
    border: 1px solid var(--border);
    border-radius: 999px;
    transition: background 200ms, border-color 200ms;
  }
  .slider::before {
    content: "";
    position: absolute;
    height: 18px;
    width: 18px;
    left: 2px;
    bottom: 2px;
    background: var(--fg-dim);
    border-radius: 50%;
    transition: transform 200ms, background 200ms;
  }
  .switch input:checked + .slider {
    background: var(--accent-hot);
    border-color: var(--accent-hot);
  }
  .switch input:checked + .slider::before {
    background: #fff;
    transform: translateX(18px);
  }

  .warning-banner {
    margin-top: 10px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 6px;
    background: rgba(255, 122, 122, 0.1);
    border: 1px solid rgba(255, 122, 122, 0.25);
    font-size: 11px;
    color: var(--accent-hot);
    animation: fadeIn 200ms ease;
  }
  .warning-icon {
    font-size: 14px;
  }

  /* Update */
  .update-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 8px 16px;
    font-weight: 600;
    font-size: 12px;
    border-radius: 8px;
    background: var(--bg-elev);
    border: 1px solid var(--border);
    color: var(--fg);
    transition: background 80ms, border-color 80ms;
  }
  .update-btn:hover:not(:disabled) {
    background: var(--bg);
    border-color: #3a4055;
  }
  .update-btn:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  .spinner {
    display: inline-block;
    width: 14px;
    height: 14px;
    border: 2px solid var(--fg-muted);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 600ms linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .update-result {
    margin-top: 12px;
    padding: 12px 14px;
    border-radius: 8px;
    background: var(--bg-elev);
    border: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    gap: 6px;
    animation: fadeIn 200ms ease;
  }
  .update-result.has-update {
    border-color: rgba(107, 212, 154, 0.3);
  }
  .version-row {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
  }
  .ver-label {
    color: var(--fg-dim);
    font-weight: 500;
  }
  .ver-value {
    font-family: ui-monospace, Menlo, monospace;
    color: var(--fg);
    font-weight: 600;
  }
  .update-cta {
    margin-top: 8px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }
  .update-badge {
    font-size: 12px;
    font-weight: 600;
    color: var(--ok);
  }
  .up-to-date {
    margin-top: 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--ok);
  }

  .release-notes {
    margin-top: 8px;
    font-size: 11px;
  }
  .release-notes summary {
    cursor: pointer;
    color: var(--fg-dim);
    font-weight: 600;
  }
  .release-notes pre {
    margin-top: 6px;
    padding: 10px;
    background: var(--bg);
    border-radius: 6px;
    font-size: 11px;
    white-space: pre-wrap;
    word-break: break-word;
    color: var(--fg-dim);
    max-height: 200px;
    overflow-y: auto;
  }

  .update-error {
    margin-top: 10px;
    padding: 8px 12px;
    border-radius: 6px;
    background: rgba(255, 122, 122, 0.08);
    border: 1px solid rgba(255, 122, 122, 0.2);
    font-size: 12px;
    color: var(--accent-hot);
  }

  .panel-footer {
    padding: 12px 20px;
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .footer-text {
    font-size: 10px;
    color: var(--fg-muted);
    letter-spacing: 0.04em;
  }
</style>
