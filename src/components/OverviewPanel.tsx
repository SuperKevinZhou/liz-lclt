import { t } from "../lib/i18n";
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
          <p className="eyebrow">{t("workspace")}</p>
          <h2>{state.workspaceRoot || t("noWorkspace")}</h2>
          <p className="muted">{t("overviewDescription")}</p>
        </div>
        <div className="hero-card__actions">
          <button
            className="button button--secondary"
            onClick={onLoadWorkspace}
            type="button"
          >
            {t("chooseWorkspace")}
          </button>
          <button
            className="button"
            onClick={() => onStartTranslation(false)}
            type="button"
          >
            {t("startTranslation")}
          </button>
          <button
            className="button button--ghost"
            onClick={() => onStartTranslation(true)}
            type="button"
          >
            {t("dryRun")}
          </button>
        </div>
      </div>

      <div className="stat-grid">
        <article className="stat-card">
          <span>{t("originLanguage")}</span>
          <strong>
            {state.currentConfig.translationSettings.originLanguage || "n/a"}
          </strong>
        </article>
        <article className="stat-card">
          <span>{t("targetDirectory")}</span>
          <strong>
            {state.currentConfig.translationSettings.targetDirection || "n/a"}
          </strong>
        </article>
        <article className="stat-card">
          <span>{t("modelSlots")}</span>
          <strong>{Object.keys(state.modelsConfig.models).length}</strong>
        </article>
        <article className="stat-card">
          <span>{t("strategies")}</span>
          <strong>
            {state.translationConfigs.translationStrategies.length}
          </strong>
        </article>
      </div>

      <div className="two-column">
        <section className="panel">
          <div className="panel__header">
            <div>
              <p className="eyebrow">{t("taskStatus")}</p>
              <h3>{task?.status ?? "idle"}</h3>
            </div>
            {task?.status === "running" || task?.status === "cancelling" ? (
              <button
                className="button button--danger"
                onClick={onCancel}
                type="button"
              >
                {t("cancel")}
              </button>
            ) : null}
          </div>

          <div className="progress-list">
            <div>
              <span>{t("scannedFiles")}</span>
              <strong>{progress?.scannedFiles ?? 0}</strong>
            </div>
            <div>
              <span>{t("pendingChars")}</span>
              <strong>{progress?.pendingChars ?? 0}</strong>
            </div>
            <div>
              <span>{t("completedBatches")}</span>
              <strong>
                {progress?.completedBatches ?? 0} /{" "}
                {progress?.totalBatches ?? 0}
              </strong>
            </div>
            <div>
              <span>{t("elapsed")}</span>
              <strong>{((progress?.elapsedMs ?? 0) / 1000).toFixed(1)}s</strong>
            </div>
          </div>

          {task?.summary ? (
            <div className="summary-card">
              <p className="eyebrow">{t("taskSummary")}</p>
              <h4>
                {t("summaryCounts", {
                  files: task.summary.translatedFiles,
                  entries: task.summary.translatedEntries,
                })}
              </h4>
              <p className="muted">
                {task.summary.outputDirectory ?? t("noOutputPath")}
              </p>
            </div>
          ) : null}
        </section>

        <section className="panel">
          <div className="panel__header">
            <div>
              <p className="eyebrow">{t("runtimeLog")}</p>
              <h3>{t("liveTaskStream")}</h3>
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
              <p className="muted">{t("noTaskOutput")}</p>
            )}
          </div>
        </section>
      </div>
    </section>
  );
}
