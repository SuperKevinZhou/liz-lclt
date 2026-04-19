import type { AppStatePayload } from "../types/app";

interface OverviewPanelProps {
  state: AppStatePayload;
  onLoadWorkspace: () => void;
  onStartTranslation: (dryRun?: boolean) => void;
  onCancel: () => void;
}

export function OverviewPanel({
  state,
  onLoadWorkspace,
  onStartTranslation,
  onCancel,
}: OverviewPanelProps) {
  const task = state.currentTask;
  const progress = task?.progress;

  return (
    <section className="panel-stack">
      <div className="hero-card">
        <div>
          <p className="eyebrow">Workspace</p>
          <h2>{state.workspaceRoot || "No workspace selected yet"}</h2>
          <p className="muted">
            Load an existing LCLT workspace to edit the original config files in
            place and run the Rust/Tauri pipeline from one window.
          </p>
        </div>
        <div className="hero-card__actions">
          <button
            className="button button--secondary"
            onClick={onLoadWorkspace}
            type="button"
          >
            Choose Workspace
          </button>
          <button
            className="button"
            onClick={() => onStartTranslation(false)}
            type="button"
          >
            Start Translation
          </button>
          <button
            className="button button--ghost"
            onClick={() => onStartTranslation(true)}
            type="button"
          >
            Dry Run
          </button>
        </div>
      </div>

      <div className="stat-grid">
        <article className="stat-card">
          <span>Origin Language</span>
          <strong>
            {state.currentConfig.translationSettings.originLanguage || "n/a"}
          </strong>
        </article>
        <article className="stat-card">
          <span>Target Directory</span>
          <strong>
            {state.currentConfig.translationSettings.targetDirection || "n/a"}
          </strong>
        </article>
        <article className="stat-card">
          <span>Model Slots</span>
          <strong>{Object.keys(state.modelsConfig.models).length}</strong>
        </article>
        <article className="stat-card">
          <span>Strategies</span>
          <strong>
            {state.translationConfigs.translationStrategies.length}
          </strong>
        </article>
      </div>

      <div className="two-column">
        <section className="panel">
          <div className="panel__header">
            <div>
              <p className="eyebrow">Task Status</p>
              <h3>{task?.status ?? "idle"}</h3>
            </div>
            {task?.status === "running" || task?.status === "cancelling" ? (
              <button
                className="button button--danger"
                onClick={onCancel}
                type="button"
              >
                Cancel
              </button>
            ) : null}
          </div>

          <div className="progress-list">
            <div>
              <span>Scanned Files</span>
              <strong>{progress?.scannedFiles ?? 0}</strong>
            </div>
            <div>
              <span>Pending Chars</span>
              <strong>{progress?.pendingChars ?? 0}</strong>
            </div>
            <div>
              <span>Completed Batches</span>
              <strong>
                {progress?.completedBatches ?? 0} /{" "}
                {progress?.totalBatches ?? 0}
              </strong>
            </div>
            <div>
              <span>Elapsed</span>
              <strong>{((progress?.elapsedMs ?? 0) / 1000).toFixed(1)}s</strong>
            </div>
          </div>

          {task?.summary ? (
            <div className="summary-card">
              <p className="eyebrow">Task Summary</p>
              <h4>
                {task.summary.translatedFiles} files,{" "}
                {task.summary.translatedEntries} entries
              </h4>
              <p className="muted">
                {task.summary.outputDirectory ?? "No output path"}
              </p>
            </div>
          ) : null}
        </section>

        <section className="panel">
          <div className="panel__header">
            <div>
              <p className="eyebrow">Runtime Log</p>
              <h3>Live task stream</h3>
            </div>
          </div>
          <div className="log-list">
            {task?.logs.length ? (
              task.logs.map((entry, index) => (
                <div
                  key={`${entry.timestampMs}-${index}`}
                  className={`log-entry log-entry--${entry.level}`}
                >
                  <span>{entry.level}</span>
                  <p>{entry.message}</p>
                </div>
              ))
            ) : (
              <p className="muted">No task output yet.</p>
            )}
          </div>
        </section>
      </div>
    </section>
  );
}
