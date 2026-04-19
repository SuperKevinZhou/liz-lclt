import { t } from "../lib/i18n";
import type { AppConfig } from "../types/app";
import type { DetectedGamePaths } from "../types/app";

interface SettingsPanelProps {
  config: AppConfig;
  autoDetectedGame?: DetectedGamePaths | null;
  onChange: (config: AppConfig) => void;
  onSave: () => void;
}

function numberValue(value: string): number {
  const next = Number(value);
  return Number.isFinite(next) ? next : 0;
}

const concurrencyPresets = [
  {
    label: "Small",
    value: 256,
    description: t("presetSmall"),
  },
  {
    label: "Balanced",
    value: 2048,
    description: t("presetBalanced"),
  },
  {
    label: "Extreme",
    value: 32768,
    description: t("presetExtreme"),
  },
];

export function SettingsPanel({
  config,
  autoDetectedGame,
  onChange,
  onSave,
}: SettingsPanelProps) {
  const translation = config.translationSettings;
  const filePaths = config.filePaths;
  const options = config.options;

  return (
    <section className="panel-stack">
      <div className="panel">
        <div className="panel__header">
          <div>
            <p className="eyebrow">{t("baseSettings")}</p>
            <h3>{t("translationConfiguration")}</h3>
          </div>
          <button className="button" onClick={onSave} type="button">
            {t("saveConfig")}
          </button>
        </div>

        <div className="form-grid">
          <label>
            <span>{t("originLanguage")}</span>
            <input
              value={translation.originLanguage}
              onChange={(event) =>
                onChange({
                  ...config,
                  translationSettings: {
                    ...translation,
                    originLanguage: event.target.value,
                  },
                })
              }
            />
          </label>
          <label>
            <span>{t("targetDirectory")}</span>
            <input
              value={translation.targetDirection}
              onChange={(event) =>
                onChange({
                  ...config,
                  translationSettings: {
                    ...translation,
                    targetDirection: event.target.value,
                  },
                })
              }
            />
          </label>
          <label>
            <span>{t("maxWorkers")}</span>
            <input
              type="number"
              value={translation.maxWorkers}
              onChange={(event) =>
                onChange({
                  ...config,
                  translationSettings: {
                    ...translation,
                    maxWorkers: numberValue(event.target.value),
                  },
                })
              }
            />
          </label>
          <div className="form-grid__full preset-strip">
            {concurrencyPresets.map((preset) => (
              <button
                className={
                  translation.maxWorkers === preset.value
                    ? "preset-button preset-button--active"
                    : "preset-button"
                }
                key={preset.value}
                onClick={() =>
                  onChange({
                    ...config,
                    translationSettings: {
                      ...translation,
                      maxWorkers: preset.value,
                    },
                  })
                }
                type="button"
              >
                <strong>
                  {preset.label === "Small"
                    ? t("small")
                    : preset.label === "Balanced"
                      ? t("balanced")
                      : t("extreme")}{" "}
                  / {preset.value.toLocaleString()}
                </strong>
                <span>{preset.description}</span>
              </button>
            ))}
          </div>
          <label>
            <span>{t("maxCharsPerBatch")}</span>
            <input
              type="number"
              value={translation.maxCharsPerBatch}
              onChange={(event) =>
                onChange({
                  ...config,
                  translationSettings: {
                    ...translation,
                    maxCharsPerBatch: numberValue(event.target.value),
                  },
                })
              }
            />
          </label>
          <label>
            <span>{t("maxRetries")}</span>
            <input
              type="number"
              value={translation.maxRetries}
              onChange={(event) =>
                onChange({
                  ...config,
                  translationSettings: {
                    ...translation,
                    maxRetries: numberValue(event.target.value),
                  },
                })
              }
            />
          </label>
          <label>
            <span>{t("timeoutSeconds")}</span>
            <input
              type="number"
              value={translation.timeout}
              onChange={(event) =>
                onChange({
                  ...config,
                  translationSettings: {
                    ...translation,
                    timeout: numberValue(event.target.value),
                  },
                })
              }
            />
          </label>
          <div className="form-grid__full path-display">
            <span>{t("inputDirectory")}</span>
            <strong>{filePaths.inputDirection || t("notDetected")}</strong>
          </div>
          <div className="form-grid__full path-display">
            <span>{t("outputDirectory")}</span>
            <strong>{filePaths.outputDirection || t("notDetected")}</strong>
          </div>
        </div>

        {autoDetectedGame ? (
          <div className="detected-card">
            <p className="eyebrow">{t("detectedSteamInstall")}</p>
            <strong>{autoDetectedGame.gameRoot}</strong>
            <p className="muted">{t("detectedSteamDescription")}</p>
          </div>
        ) : (
          <div className="detected-card">
            <p className="eyebrow">{t("steamDetection")}</p>
            <strong>{t("steamNotFound")}</strong>
            <p className="muted">{t("steamNotFoundDescription")}</p>
          </div>
        )}

        <div className="toggle-row">
          <label className="toggle">
            <input
              type="checkbox"
              checked={options.confirmBeforeTranslation}
              onChange={(event) =>
                onChange({
                  ...config,
                  options: {
                    ...options,
                    confirmBeforeTranslation: event.target.checked,
                  },
                })
              }
            />
            <span>{t("confirmBeforeTranslation")}</span>
          </label>
          <label className="toggle">
            <input
              type="checkbox"
              checked={options.keepBackupFiles}
              onChange={(event) =>
                onChange({
                  ...config,
                  options: {
                    ...options,
                    keepBackupFiles: event.target.checked,
                  },
                })
              }
            />
            <span>{t("keepBackupFiles")}</span>
          </label>
        </div>
      </div>
    </section>
  );
}
