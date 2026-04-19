import { useMemo, useState } from "react";
import { confirm, open } from "@tauri-apps/plugin-dialog";
import { openPath } from "@tauri-apps/plugin-opener";

import { OverviewPanel } from "./components/OverviewPanel";
import { ResourcesPanel } from "./components/ResourcesPanel";
import { SettingsPanel } from "./components/SettingsPanel";
import { Sidebar } from "./components/Sidebar";
import { ModelsPanel } from "./components/ModelsPanel";
import { StrategiesPanel } from "./components/StrategiesPanel";
import { useAppState } from "./hooks/useAppState";
import { t } from "./lib/i18n";
import type { NavKey } from "./types/app";
import "./App.css";

function App() {
  const [activeNav, setActiveNav] = useState<NavKey>("overview");
  const [workspaceInput, setWorkspaceInput] = useState(
    "D:/zzh/Code/LCLT-neo/LimbusCompanyLLMTranslator",
  );
  const { state, isLoading, actionMessage, activeError, actions } =
    useAppState();

  const promptOptions = useMemo(
    () => state.promptFiles.map((file) => file.path),
    [state.promptFiles],
  );
  const terminologyOptions = useMemo(
    () => state.terminologyFiles.map((file) => file.path),
    [state.terminologyFiles],
  );

  async function chooseWorkspace() {
    const selection = await open({
      directory: true,
      multiple: false,
      title: t("chooseWorkspace"),
    });
    if (typeof selection === "string") {
      setWorkspaceInput(selection);
      await actions.load(selection);
    }
  }

  async function startTranslationWithGuards(dryRun = false) {
    const problems: string[] = [];
    const config = state.currentConfig;
    if (!state.workspaceRoot.trim()) {
      problems.push(t("problemNoWorkspace"));
    }
    if (!config.filePaths.inputDirection.trim()) {
      problems.push(t("problemNoInput"));
    }
    if (!config.filePaths.outputDirection.trim()) {
      problems.push(t("problemNoOutput"));
    }
    if (!Object.keys(state.modelsConfig.models).length) {
      problems.push(t("problemNoModels"));
    }
    if (!state.translationConfigs.translationStrategies.length) {
      problems.push(t("problemNoStrategies"));
    }

    const badStrategies = state.translationConfigs.translationStrategies.filter(
      (strategy) => {
        const hasModel = Boolean(state.modelsConfig.models[strategy.model]);
        const hasPrompt = promptOptions.includes(strategy.promptFile);
        const hasTerminology =
          !strategy.terminologyFile ||
          terminologyOptions.includes(strategy.terminologyFile);
        return !(hasModel && hasPrompt && hasTerminology);
      },
    );
    if (badStrategies.length) {
      problems.push(
        t("problemInvalidStrategies", {
          names: badStrategies.map((strategy) => strategy.name).join(", "),
        }),
      );
    }

    if (problems.length) {
      actions.setError({
        title: t("cannotStartTitle"),
        message: t("cannotStartMessage"),
        details: problems.join("\n"),
      });
      return;
    }

    const pendingChars = state.currentTask?.progress.pendingChars ?? 0;
    if (!dryRun && config.options.confirmBeforeTranslation) {
      const accepted = await confirm(
        pendingChars > 0
          ? t("confirmTranslationWithBytes", {
              bytes: pendingChars.toLocaleString(),
            })
          : t("confirmTranslationGeneric"),
        {
          title: t("confirmTranslationTitle"),
          kind: "warning",
          okLabel: t("start"),
          cancelLabel: t("cancel"),
        },
      );
      if (!accepted) {
        return;
      }
    }

    await actions.startTranslation(dryRun);
  }

  return (
    <div className="app-shell">
      <Sidebar active={activeNav} onSelect={setActiveNav} />

      <main className="main-content">
        <header className="topbar">
          <div>
            <p className="eyebrow">Limbus Company LLM Translator</p>
            <h2>{t("appSubtitle")}</h2>
          </div>
          <div className="workspace-bar">
            <input
              value={workspaceInput}
              onChange={(event) => setWorkspaceInput(event.target.value)}
              placeholder={t("workspacePlaceholder")}
            />
            <button
              className="button button--secondary"
              onClick={() => void actions.load(workspaceInput)}
              type="button"
            >
              {t("load")}
            </button>
            <button
              className="button button--secondary"
              onClick={() => void chooseWorkspace()}
              type="button"
            >
              {t("pickFolder")}
            </button>
            <button
              className="button button--ghost"
              onClick={() => {
                if (state.workspaceRoot) {
                  void openPath(state.workspaceRoot);
                }
              }}
              type="button"
            >
              {t("openFolder")}
            </button>
          </div>
        </header>

        {activeError ? (
          <section className="alert alert--error">
            <strong>{activeError.title}</strong>
            <p>{activeError.message}</p>
            {activeError.details ? <code>{activeError.details}</code> : null}
          </section>
        ) : null}

        {state.problems.length ? (
          <section className="alert alert--warn">
            <strong>{t("workspaceIssues")}</strong>
            <ul>
              {state.problems.map((problem, index) => (
                <li key={`${problem.title}-${index}`}>
                  {problem.title}: {problem.message}
                </li>
              ))}
            </ul>
          </section>
        ) : null}

        {(isLoading || actionMessage) && (
          <section className="alert alert--info">
            <strong>{isLoading ? t("loading") : t("ready")}</strong>
            <p>{isLoading ? t("readingWorkspace") : actionMessage}</p>
          </section>
        )}

        {activeNav === "overview" ? (
          <OverviewPanel
            state={state}
            onLoadWorkspace={() => void chooseWorkspace()}
            onStartTranslation={(dryRun) =>
              void startTranslationWithGuards(dryRun)
            }
            onCancel={() => void actions.cancelTranslation()}
          />
        ) : null}

        {activeNav === "settings" ? (
          <SettingsPanel
            config={state.currentConfig}
            autoDetectedGame={state.autoDetectedGame}
            onChange={actions.setConfig}
            onSave={() => void actions.saveConfig()}
          />
        ) : null}

        {activeNav === "models" ? (
          <ModelsPanel
            modelsConfig={state.modelsConfig}
            onChange={actions.setModels}
            onSave={() => void actions.saveModels()}
          />
        ) : null}

        {activeNav === "strategies" ? (
          <StrategiesPanel
            translationConfigs={state.translationConfigs}
            modelsConfig={state.modelsConfig}
            promptOptions={promptOptions}
            terminologyOptions={terminologyOptions}
            onChange={actions.setTranslationConfigs}
            onSave={() => void actions.saveTranslationConfigs()}
          />
        ) : null}

        {activeNav === "resources" ? (
          <ResourcesPanel
            promptFiles={state.promptFiles}
            terminologyFiles={state.terminologyFiles}
            blacklistConfig={state.blacklistConfig}
            onChangeBlacklist={actions.setBlacklist}
            onPersistBlacklist={() => void actions.saveBlacklist()}
            setError={actions.setError}
          />
        ) : null}
      </main>
    </div>
  );
}

export default App;
