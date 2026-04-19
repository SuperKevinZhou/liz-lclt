import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type {
  AppConfig,
  AppStatePayload,
  BlacklistConfig,
  ModelsConfig,
  SaveTextResourcePayload,
  TaskLogEntry,
  TerminologyDictionary,
  TextResourcePayload,
  TranslationConfigs,
  TranslationTask,
} from "../types/app";

export const commands = {
  setWorkspaceRoot: (workspaceRoot: string) =>
    invoke<string>("set_workspace_root", {
      payload: { workspaceRoot },
    }),
  loadAppState: (workspaceRoot?: string) =>
    invoke<AppStatePayload>("load_app_state", { workspaceRoot }),
  loadTextResource: (path: string) =>
    invoke<TextResourcePayload>("load_text_resource", {
      payload: { path, content: "" },
    }),
  saveConfig: (payload: AppConfig) => invoke<void>("save_config", { payload }),
  saveModels: (payload: ModelsConfig) => invoke<void>("save_models", { payload }),
  saveTranslationConfigs: (payload: TranslationConfigs) =>
    invoke<void>("save_translation_configs", { payload }),
  saveBlacklist: (payload: BlacklistConfig) =>
    invoke<void>("save_blacklist", { payload }),
  saveTextResource: (payload: SaveTextResourcePayload) =>
    invoke<void>("save_text_resource", { payload }),
  saveTerminology: (path: string, terminology: TerminologyDictionary) =>
    invoke<void>("save_terminology", {
      payload: {
        path,
        payload: terminology,
      },
    }),
  startTranslation: (dryRun = false) =>
    invoke<TranslationTask>("start_translation", {
      payload: { dryRun },
    }),
  cancelTranslation: (taskId: string) =>
    invoke<TranslationTask>("cancel_translation", { taskId }),
  getTaskStatus: (taskId?: string) =>
    invoke<TranslationTask | null>("get_task_status", { taskId }),
};

export const taskEvents = {
  onProgress: (handler: (task: TranslationTask) => void) =>
    listen<TranslationTask>("task_progress", (event) => handler(event.payload)),
  onLog: (handler: (entry: TaskLogEntry) => void) =>
    listen<TaskLogEntry>("task_log", (event) => handler(event.payload)),
  onFinished: (handler: (task: TranslationTask) => void) =>
    listen<TranslationTask>("task_finished", (event) => handler(event.payload)),
};
