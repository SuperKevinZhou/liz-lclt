export type LogLevel = "info" | "warn" | "error";

export type TaskStatus =
  | "idle"
  | "running"
  | "cancelling"
  | "cancelled"
  | "succeeded"
  | "failed";

export interface TranslationSettings {
  originLanguage: string;
  targetDirection: string;
  maxWorkers: number;
  maxCharsPerBatch: number;
  maxRetries: number;
  timeout: number;
}

export interface FilePaths {
  inputDirection: string;
  outputDirection: string;
}

export interface ConfigFiles {
  models: string;
  translationConfigs: string;
}

export interface AppOptions {
  keepBackupFiles: boolean;
  confirmBeforeTranslation: boolean;
}

export interface AppConfig {
  translationSettings: TranslationSettings;
  filePaths: FilePaths;
  configFiles: ConfigFiles;
  options: AppOptions;
}

export interface ModelProfile {
  apiKey: string;
  baseUrl: string;
  model: string;
  temperature: number;
  enableThinking: boolean;
}

export interface ModelsConfig {
  models: Record<string, ModelProfile>;
}

export interface FilePatternRule {
  pattern: string;
  extractFields?: string[] | null;
}

export interface TranslationStrategy {
  name: string;
  priority: number;
  filePatterns: FilePatternRule[];
  model: string;
  promptFile: string;
  terminologyFile?: string | null;
  extractFields?: string[] | null;
}

export interface TranslationConfigs {
  translationStrategies: TranslationStrategy[];
}

export interface BlacklistConfig {
  blacklist: string[];
}

export interface TerminologyDictionary {
  terminology: Record<string, string>;
}

export interface ResourceFile {
  path: string;
  label: string;
}

export interface DetectedGamePaths {
  steamLibraryRoot: string;
  gameRoot: string;
  localizeRoot: string;
  langRoot: string;
}

export interface TextResourcePayload {
  path: string;
  content: string;
}

export interface SaveTextResourcePayload {
  path: string;
  content: string;
}

export interface UserFacingError {
  title: string;
  message: string;
  details?: string | null;
}

export interface TaskProgressSnapshot {
  scannedFiles: number;
  pendingChars: number;
  completedBatches: number;
  totalBatches: number;
  elapsedMs: number;
  outputDirectory?: string | null;
}

export interface TaskLogEntry {
  level: LogLevel;
  message: string;
  timestampMs: number;
}

export interface TaskSummary {
  translatedFiles: number;
  translatedEntries: number;
  pendingChars: number;
  outputDirectory?: string | null;
  backupDirectory?: string | null;
  logFile?: string | null;
  error?: UserFacingError | null;
}

export interface TranslationTask {
  taskId: string;
  status: TaskStatus;
  progress: TaskProgressSnapshot;
  logs: TaskLogEntry[];
  summary?: TaskSummary | null;
}

export interface AppStatePayload {
  workspaceRoot: string;
  currentConfig: AppConfig;
  modelsConfig: ModelsConfig;
  translationConfigs: TranslationConfigs;
  blacklistConfig: BlacklistConfig;
  promptFiles: ResourceFile[];
  terminologyFiles: ResourceFile[];
  autoDetectedGame?: DetectedGamePaths | null;
  problems: UserFacingError[];
  currentTask?: TranslationTask | null;
}

export type NavKey =
  | "overview"
  | "settings"
  | "models"
  | "strategies"
  | "resources";
