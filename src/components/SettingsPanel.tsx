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
    description:
      "High enough for most providers without flooding the desktop runtime.",
  },
  {
    label: "Balanced",
    value: 2048,
    description: "Aggressive async throughput for large translation runs.",
  },
  {
    label: "Extreme",
    value: 32768,
    description:
      "Maximum Rust/Tokio pressure; provider/network limits will dominate.",
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
            <p className="eyebrow">Base Settings</p>
            <h3>Translation configuration</h3>
          </div>
          <button className="button" onClick={onSave} type="button">
            Save config.json
          </button>
        </div>

        <div className="form-grid">
          <label>
            <span>Origin Language</span>
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
            <span>Target Directory</span>
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
            <span>Max Workers</span>
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
                  {preset.label} / {preset.value.toLocaleString()}
                </strong>
                <span>{preset.description}</span>
              </button>
            ))}
          </div>
          <label>
            <span>Max Chars Per Batch</span>
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
            <span>Max Retries</span>
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
            <span>Timeout (seconds)</span>
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
            <span>Input Directory</span>
            <strong>{filePaths.inputDirection || "Not detected yet"}</strong>
          </div>
          <div className="form-grid__full path-display">
            <span>Output Directory</span>
            <strong>{filePaths.outputDirection || "Not detected yet"}</strong>
          </div>
        </div>

        {autoDetectedGame ? (
          <div className="detected-card">
            <p className="eyebrow">Detected Steam Install</p>
            <strong>{autoDetectedGame.gameRoot}</strong>
            <p className="muted">
              Input and output paths are inferred from the original project
              layout under the Steam library.
            </p>
          </div>
        ) : (
          <div className="detected-card">
            <p className="eyebrow">Steam Detection</p>
            <strong>Automatic scan did not find Limbus Company.</strong>
            <p className="muted">
              The app looks for the Steam library root and scans
              `steamapps/common/Limbus Company` for the original `Localize` and
              `Lang` folders.
            </p>
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
            <span>Confirm before translation</span>
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
            <span>Keep backup files</span>
          </label>
        </div>
      </div>
    </section>
  );
}
