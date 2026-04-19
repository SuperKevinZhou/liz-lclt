import { useEffect, useMemo, useState } from "react";

import { commands, taskEvents } from "../lib/tauri";
import type {
  AppConfig,
  AppStatePayload,
  BlacklistConfig,
  ModelsConfig,
  TaskLogEntry,
  TranslationConfigs,
  UserFacingError,
} from "../types/app";

function defaultState(): AppStatePayload {
  return {
    workspaceRoot: "",
    currentConfig: {
      translationSettings: {
        originLanguage: "jp",
        targetDirection: "LCLT_zh",
        maxWorkers: 256,
        maxCharsPerBatch: 2000,
        maxRetries: 4,
        timeout: 90,
      },
      filePaths: {
        inputDirection: "",
        outputDirection: "",
      },
      configFiles: {
        models: "models.json",
        translationConfigs: "translation_configs.json",
      },
      options: {
        keepBackupFiles: false,
        confirmBeforeTranslation: true,
      },
    },
    modelsConfig: { models: {} },
    translationConfigs: { translationStrategies: [] },
    blacklistConfig: { blacklist: [] },
    promptFiles: [],
    terminologyFiles: [],
    autoDetectedGame: null,
    problems: [],
    currentTask: null,
  };
}

export function useAppState() {
  const [state, setState] = useState<AppStatePayload>(defaultState);
  const [isLoading, setIsLoading] = useState(false);
  const [actionMessage, setActionMessage] = useState<string>("");
  const [activeError, setActiveError] = useState<UserFacingError | null>(null);

  useEffect(() => {
    let disposeProgress: (() => void) | undefined;
    let disposeLog: (() => void) | undefined;
    let disposeFinished: (() => void) | undefined;

    void (async () => {
      disposeProgress = await taskEvents.onProgress((task) => {
        setState((current) => ({ ...current, currentTask: task }));
      });
      disposeLog = await taskEvents.onLog((entry) => {
        setState((current) => {
          const currentTask = current.currentTask;
          if (!currentTask) {
            return current;
          }
          return {
            ...current,
            currentTask: {
              ...currentTask,
              logs: [...currentTask.logs, entry],
            },
          };
        });
      });
      disposeFinished = await taskEvents.onFinished((task) => {
        setState((current) => ({ ...current, currentTask: task }));
      });
    })();

    return () => {
      disposeProgress?.();
      disposeLog?.();
      disposeFinished?.();
    };
  }, []);

  const actions = useMemo(
    () => ({
      async load(workspaceRoot?: string) {
        setIsLoading(true);
        setActiveError(null);
        try {
          if (workspaceRoot) {
            await commands.setWorkspaceRoot(workspaceRoot);
          }
          const payload = await commands.loadAppState(workspaceRoot);
          setState(payload);
          setActionMessage("工作区已加载。");
        } catch (error) {
          setActiveError(error as UserFacingError);
        } finally {
          setIsLoading(false);
        }
      },
      setConfig(next: AppConfig) {
        setState((current) => ({ ...current, currentConfig: next }));
      },
      setModels(next: ModelsConfig) {
        setState((current) => ({ ...current, modelsConfig: next }));
      },
      setTranslationConfigs(next: TranslationConfigs) {
        setState((current) => ({ ...current, translationConfigs: next }));
      },
      setBlacklist(next: BlacklistConfig) {
        setState((current) => ({ ...current, blacklistConfig: next }));
      },
      async saveConfig() {
        try {
          await commands.saveConfig(state.currentConfig);
          setActionMessage("已保存 config.json");
        } catch (error) {
          setActiveError(error as UserFacingError);
        }
      },
      async saveModels() {
        try {
          await commands.saveModels(state.modelsConfig);
          setActionMessage("已保存 models.json");
        } catch (error) {
          setActiveError(error as UserFacingError);
        }
      },
      async saveTranslationConfigs() {
        try {
          await commands.saveTranslationConfigs(state.translationConfigs);
          setActionMessage("已保存 translation_configs.json");
        } catch (error) {
          setActiveError(error as UserFacingError);
        }
      },
      async saveBlacklist() {
        try {
          await commands.saveBlacklist(state.blacklistConfig);
          setActionMessage("已保存 BlackList.json");
        } catch (error) {
          setActiveError(error as UserFacingError);
        }
      },
      async startTranslation(dryRun = false) {
        try {
          const task = await commands.startTranslation(dryRun);
          setState((current) => ({ ...current, currentTask: task }));
          setActionMessage(
            dryRun ? "试运行已启动。" : "翻译任务已启动。",
          );
        } catch (error) {
          setActiveError(error as UserFacingError);
        }
      },
      async cancelTranslation() {
        const task = state.currentTask;
        if (!task) {
          return;
        }
        try {
          const nextTask = await commands.cancelTranslation(task.taskId);
          setState((current) => ({ ...current, currentTask: nextTask }));
          setActionMessage("已请求取消任务。");
        } catch (error) {
          setActiveError(error as UserFacingError);
        }
      },
      appendTaskLog(entry: TaskLogEntry) {
        setState((current) => {
          const currentTask = current.currentTask;
          if (!currentTask) {
            return current;
          }
          return {
            ...current,
            currentTask: {
              ...currentTask,
              logs: [...currentTask.logs, entry],
            },
          };
        });
      },
      setError(error: UserFacingError | null) {
        setActiveError(error);
      },
    }),
    [state],
  );

  return {
    state,
    isLoading,
    actionMessage,
    activeError,
    actions,
  };
}
