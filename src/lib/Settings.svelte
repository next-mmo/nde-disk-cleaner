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
        <div class="header-titles">
          <h2>Settings</h2>
          <p class="sub">NDE Disk Cleaner</p>
        </div>
        <button class="close-btn ghost" onclick={onclose} aria-label="Close settings">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
            <path d="M3 3l8 8M11 3l-8 8" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>
          </svg>
        </button>
      </header>

      <div class="sections scroll">
        <!-- ── Deletion ─────────────────────────── -->
        <section class="section">
          <div class="section-icon icon-trash">
            <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              <path d="M5 6l1 14a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2l1-14"/>
            </svg>
          </div>
          <div class="section-body">
            <h3>Deletion Mode</h3>
            <p class="desc">
              When enabled, the "Delete" action will <strong>permanently remove</strong> files
              from disk instead of moving them to the Trash. This cannot be undone.
            </p>
            <label class="switch-row">
              <span class="switch-label">Allow permanent deletion</span>
              <span class="switch-tahoe">
                <input
                  type="checkbox"
                  checked={allowPermanentDelete}
                  onchange={handleToggle}
                />
                <span class="track"><span class="thumb"></span></span>
              </span>
            </label>
            {#if allowPermanentDelete}
              <div class="warning-banner">
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                  <path d="M12 9v4M12 17h.01"/>
                  <path d="M10.3 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/>
                </svg>
                <span>Permanent deletion is <strong>active</strong>. Deleted files cannot be recovered.</span>
              </div>
            {/if}
          </div>
        </section>

        <!-- ── Updates ─────────────────────────── -->
        <section class="section">
          <div class="section-icon icon-update">
            <svg viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 12a9 9 0 0 1-15.5 6.36M3 12a9 9 0 0 1 15.5-6.36"/>
              <path d="M21 4v5h-5M3 20v-5h5"/>
            </svg>
          </div>
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
        <span class="footer-text">NDE Disk Cleaner · v0.1.3</span>
      </footer>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 999;
    background: rgba(0, 0, 0, 0.45);
    -webkit-backdrop-filter: blur(20px) saturate(150%);
    backdrop-filter: blur(20px) saturate(150%);
    display: flex;
    justify-content: flex-end;
    animation: fadeIn 180ms ease;
  }
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  @keyframes slideIn {
    from { transform: translateX(100%) scale(0.98); opacity: 0; }
    to   { transform: translateX(0) scale(1);       opacity: 1; }
  }

  .panel {
    width: 420px;
    max-width: 92vw;
    height: 100%;
    background: var(--glass-sidebar);
    -webkit-backdrop-filter: blur(50px) saturate(200%);
    backdrop-filter: blur(50px) saturate(200%);
    border-left: 1px solid var(--border-soft);
    display: flex;
    flex-direction: column;
    animation: slideIn 280ms cubic-bezier(0.22, 1, 0.36, 1);
    box-shadow:
      -24px 0 60px rgba(0, 0, 0, 0.5),
      -1px 0 0 rgba(255, 255, 255, 0.04) inset;
  }

  .panel-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    padding: 22px 22px 18px;
    border-bottom: 1px solid var(--border-soft);
  }
  .header-titles h2 {
    margin: 0;
    font-size: 22px;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--fg);
  }
  .sub {
    margin: 4px 0 0;
    font-size: 12px;
    color: var(--fg-muted);
  }
  .close-btn {
    width: 28px;
    height: 28px;
    padding: 0;
    border-radius: 8px;
    color: var(--fg-dim);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .sections {
    flex: 1;
    overflow-y: auto;
    padding: 18px 22px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .section {
    display: flex;
    gap: 14px;
    padding: 16px;
    background: var(--glass-card);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
    backdrop-filter: blur(20px) saturate(180%);
    border: 1px solid var(--border-soft);
    border-radius: 12px;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.04) inset;
  }
  .section-icon {
    width: 36px;
    height: 36px;
    border-radius: 9px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
  .icon-trash {
    background: linear-gradient(180deg, rgba(255, 69, 58, 0.25), rgba(255, 69, 58, 0.12));
    color: var(--accent-hot);
    border: 1px solid rgba(255, 69, 58, 0.3);
  }
  .icon-update {
    background: linear-gradient(180deg, rgba(10, 132, 255, 0.25), rgba(10, 132, 255, 0.12));
    color: var(--accent);
    border: 1px solid rgba(10, 132, 255, 0.3);
  }
  .section-body {
    flex: 1;
    min-width: 0;
  }
  .section-body h3 {
    margin: 0 0 4px;
    font-size: 14px;
    font-weight: 600;
    color: var(--fg);
    letter-spacing: -0.01em;
  }
  .desc {
    margin: 0 0 12px;
    font-size: 12px;
    color: var(--fg-dim);
    line-height: 1.5;
  }

  /* Toggle switch (Tahoe) */
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
  .switch-tahoe {
    position: relative;
    display: inline-block;
    width: 38px;
    height: 22px;
    flex-shrink: 0;
  }
  .switch-tahoe input {
    opacity: 0;
    width: 0;
    height: 0;
    position: absolute;
  }
  .switch-tahoe .track {
    position: absolute;
    inset: 0;
    background: rgba(120, 120, 128, 0.36);
    border-radius: 999px;
    transition: background 200ms ease;
  }
  .switch-tahoe .thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #fff;
    box-shadow:
      0 2px 4px rgba(0, 0, 0, 0.3),
      0 1px 1px rgba(0, 0, 0, 0.2);
    transition: transform 220ms cubic-bezier(0.4, 0, 0.2, 1), background 200ms;
  }
  .switch-tahoe input:checked + .track {
    background: var(--accent-ok);
  }
  .switch-tahoe input:checked + .track .thumb {
    transform: translateX(16px);
  }

  .warning-banner {
    margin-top: 10px;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 8px;
    background: rgba(255, 69, 58, 0.12);
    border: 1px solid rgba(255, 69, 58, 0.3);
    font-size: 11px;
    color: var(--accent-hot);
    animation: fadeIn 200ms ease;
  }

  /* Update */
  .update-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 7px 14px;
    font-weight: 600;
    font-size: 12px;
  }
  .update-btn:disabled {
    opacity: 0.6;
    cursor: wait;
  }

  .spinner {
    display: inline-block;
    width: 13px;
    height: 13px;
    border: 2px solid rgba(255, 255, 255, 0.18);
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
    border-radius: 10px;
    background: var(--glass-panel);
    border: 1px solid var(--border-soft);
    display: flex;
    flex-direction: column;
    gap: 6px;
    animation: fadeIn 200ms ease;
  }
  .update-result.has-update {
    border-color: rgba(48, 209, 88, 0.4);
    box-shadow: 0 0 0 1px rgba(48, 209, 88, 0.15) inset;
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
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
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
    color: var(--accent-ok);
  }
  .up-to-date {
    margin-top: 6px;
    font-size: 12px;
    font-weight: 600;
    color: var(--accent-ok);
  }

  .release-notes {
    margin-top: 8px;
    font-size: 11px;
  }
  .release-notes summary {
    cursor: pointer;
    color: var(--fg-dim);
    font-weight: 600;
    padding: 4px 0;
  }
  .release-notes pre {
    margin-top: 6px;
    padding: 10px;
    background: rgba(0, 0, 0, 0.25);
    border: 1px solid var(--border-soft);
    border-radius: 8px;
    font-size: 11px;
    white-space: pre-wrap;
    word-break: break-word;
    color: var(--fg-dim);
    max-height: 200px;
    overflow-y: auto;
    font-family: ui-monospace, "SF Mono", Menlo, monospace;
  }

  .update-error {
    margin-top: 10px;
    padding: 8px 12px;
    border-radius: 8px;
    background: rgba(255, 69, 58, 0.10);
    border: 1px solid rgba(255, 69, 58, 0.25);
    font-size: 12px;
    color: var(--accent-hot);
  }

  .panel-footer {
    padding: 12px 22px;
    border-top: 1px solid var(--border-soft);
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