import { t } from "../lib/i18n";
import type { ModelsConfig } from "../types/app";

interface ModelsPanelProps {
  modelsConfig: ModelsConfig;
  onChange: (config: ModelsConfig) => void;
  onSave: () => void;
}

export function ModelsPanel({
  modelsConfig,
  onChange,
  onSave,
}: ModelsPanelProps) {
  const entries = Object.entries(modelsConfig.models);

  function updateEntries(
    producer: (
      current: Array<[string, ModelsConfig["models"][string]]>,
    ) => Array<[string, ModelsConfig["models"][string]]>,
  ) {
    onChange({
      models: Object.fromEntries(producer(entries)),
    });
  }

  return (
    <section className="panel-stack">
      <div className="panel">
        <div className="panel__header">
          <div>
            <p className="eyebrow">{t("models")}</p>
            <h3>{t("modelSlotDefinitions")}</h3>
          </div>
          <div className="button-group">
            <button
              className="button button--secondary"
              onClick={() =>
                onChange({
                  models: {
                    ...modelsConfig.models,
                    [`slot_${entries.length + 1}`]: {
                      apiKey: "",
                      baseUrl: "",
                      model: "",
                      temperature: 0.2,
                      enableThinking: false,
                    },
                  },
                })
              }
              type="button"
            >
              {t("addSlot")}
            </button>
            <button className="button" onClick={onSave} type="button">
              {t("saveModels")}
            </button>
          </div>
        </div>

        <div className="card-list">
          {entries.map(([slotName, model], index) => (
            <article className="editor-card" key={`model-row-${index}`}>
              <div className="editor-card__header">
                <input
                  className="slot-name"
                  value={slotName}
                  onChange={(event) => {
                    const nextName = event.target.value;
                    updateEntries((current) =>
                      current.map((entry, entryIndex) =>
                        entryIndex === index ? [nextName, entry[1]] : entry,
                      ),
                    );
                  }}
                />
                <button
                  className="button button--ghost"
                  onClick={() => {
                    updateEntries((current) =>
                      current.filter((_, entryIndex) => entryIndex !== index),
                    );
                  }}
                  type="button"
                >
                  {t("remove")}
                </button>
              </div>
              <div className="form-grid">
                <label className="form-grid__full">
                  <span>{t("apiKey")}</span>
                  <input
                    type="password"
                    value={model.apiKey}
                    onChange={(event) =>
                      onChange({
                        models: {
                          ...modelsConfig.models,
                          [slotName]: { ...model, apiKey: event.target.value },
                        },
                      })
                    }
                  />
                </label>
                <label className="form-grid__full">
                  <span>{t("baseUrl")}</span>
                  <input
                    value={model.baseUrl}
                    onChange={(event) =>
                      onChange({
                        models: {
                          ...modelsConfig.models,
                          [slotName]: { ...model, baseUrl: event.target.value },
                        },
                      })
                    }
                  />
                </label>
                <label>
                  <span>{t("model")}</span>
                  <input
                    value={model.model}
                    onChange={(event) =>
                      onChange({
                        models: {
                          ...modelsConfig.models,
                          [slotName]: { ...model, model: event.target.value },
                        },
                      })
                    }
                  />
                </label>
                <label>
                  <span>{t("temperature")}</span>
                  <input
                    type="number"
                    step="0.1"
                    value={model.temperature}
                    onChange={(event) =>
                      onChange({
                        models: {
                          ...modelsConfig.models,
                          [slotName]: {
                            ...model,
                            temperature: Number(event.target.value),
                          },
                        },
                      })
                    }
                  />
                </label>
              </div>
              <label className="toggle">
                <input
                  type="checkbox"
                  checked={model.enableThinking}
                  onChange={(event) =>
                    onChange({
                      models: {
                        ...modelsConfig.models,
                        [slotName]: {
                          ...model,
                          enableThinking: event.target.checked,
                        },
                      },
                    })
                  }
                />
                <span>{t("enableThinking")}</span>
              </label>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}
