import type { ModelsConfig, TranslationConfigs } from "../types/app";

interface StrategiesPanelProps {
  translationConfigs: TranslationConfigs;
  modelsConfig: ModelsConfig;
  promptOptions: string[];
  terminologyOptions: string[];
  onChange: (config: TranslationConfigs) => void;
  onSave: () => void;
}

export function StrategiesPanel({
  translationConfigs,
  modelsConfig,
  promptOptions,
  terminologyOptions,
  onChange,
  onSave,
}: StrategiesPanelProps) {
  const strategies = translationConfigs.translationStrategies;

  return (
    <section className="panel-stack">
      <div className="panel">
        <div className="panel__header">
          <div>
            <p className="eyebrow">Strategies</p>
            <h3>Pattern rules and bindings</h3>
          </div>
          <div className="button-group">
            <button
              className="button button--secondary"
              onClick={() =>
                onChange({
                  translationStrategies: [
                    ...strategies,
                    {
                      name: `strategy_${strategies.length + 1}`,
                      priority: strategies.length + 1,
                      filePatterns: [{ pattern: "*" }],
                      model: Object.keys(modelsConfig.models)[0] ?? "",
                      promptFile: promptOptions[0] ?? "",
                      terminologyFile: terminologyOptions[0] ?? "",
                      extractFields: [],
                    },
                  ],
                })
              }
              type="button"
            >
              Add Strategy
            </button>
            <button className="button" onClick={onSave} type="button">
              Save translation_configs.json
            </button>
          </div>
        </div>

        <div className="card-list">
          {strategies.map((strategy, index) => (
            <article className="editor-card" key={`${strategy.name}-${index}`}>
              <div className="editor-card__header">
                <input
                  className="slot-name"
                  value={strategy.name}
                  onChange={(event) => {
                    const next = [...strategies];
                    next[index] = { ...strategy, name: event.target.value };
                    onChange({ translationStrategies: next });
                  }}
                />
                <button
                  className="button button--ghost"
                  onClick={() => {
                    const next = strategies.filter((_, itemIndex) => itemIndex !== index);
                    onChange({ translationStrategies: next });
                  }}
                  type="button"
                >
                  Remove
                </button>
              </div>
              <div className="form-grid">
                <label>
                  <span>Priority</span>
                  <input
                    type="number"
                    value={strategy.priority}
                    onChange={(event) => {
                      const next = [...strategies];
                      next[index] = { ...strategy, priority: Number(event.target.value) };
                      onChange({ translationStrategies: next });
                    }}
                  />
                </label>
                <label>
                  <span>Model Slot</span>
                  <select
                    value={strategy.model}
                    onChange={(event) => {
                      const next = [...strategies];
                      next[index] = { ...strategy, model: event.target.value };
                      onChange({ translationStrategies: next });
                    }}
                  >
                    {Object.keys(modelsConfig.models).map((modelName) => (
                      <option key={modelName} value={modelName}>
                        {modelName}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  <span>Prompt</span>
                  <select
                    value={strategy.promptFile}
                    onChange={(event) => {
                      const next = [...strategies];
                      next[index] = { ...strategy, promptFile: event.target.value };
                      onChange({ translationStrategies: next });
                    }}
                  >
                    {promptOptions.map((path) => (
                      <option key={path} value={path}>
                        {path}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  <span>Terminology</span>
                  <select
                    value={strategy.terminologyFile ?? ""}
                    onChange={(event) => {
                      const next = [...strategies];
                      next[index] = { ...strategy, terminologyFile: event.target.value };
                      onChange({ translationStrategies: next });
                    }}
                  >
                    {terminologyOptions.map((path) => (
                      <option key={path} value={path}>
                        {path}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="form-grid__full">
                  <span>Extract Fields (comma separated)</span>
                  <input
                    value={(strategy.extractFields ?? []).join(", ")}
                    onChange={(event) => {
                      const next = [...strategies];
                      next[index] = {
                        ...strategy,
                        extractFields: event.target.value
                          .split(",")
                          .map((value) => value.trim())
                          .filter(Boolean),
                      };
                      onChange({ translationStrategies: next });
                    }}
                  />
                </label>
              </div>

              <div className="pattern-list">
                {strategy.filePatterns.map((pattern, patternIndex) => (
                  <div className="pattern-row" key={`${pattern.pattern}-${patternIndex}`}>
                    <input
                      value={pattern.pattern}
                      onChange={(event) => {
                        const next = [...strategies];
                        const nextPatterns = [...strategy.filePatterns];
                        nextPatterns[patternIndex] = {
                          ...pattern,
                          pattern: event.target.value,
                        };
                        next[index] = { ...strategy, filePatterns: nextPatterns };
                        onChange({ translationStrategies: next });
                      }}
                    />
                    <input
                      placeholder="extract fields"
                      value={(pattern.extractFields ?? []).join(", ")}
                      onChange={(event) => {
                        const next = [...strategies];
                        const nextPatterns = [...strategy.filePatterns];
                        nextPatterns[patternIndex] = {
                          ...pattern,
                          extractFields: event.target.value
                            .split(",")
                            .map((value) => value.trim())
                            .filter(Boolean),
                        };
                        next[index] = { ...strategy, filePatterns: nextPatterns };
                        onChange({ translationStrategies: next });
                      }}
                    />
                    <button
                      className="button button--ghost"
                      onClick={() => {
                        const next = [...strategies];
                        const nextPatterns = strategy.filePatterns.filter(
                          (_, itemIndex) => itemIndex !== patternIndex,
                        );
                        next[index] = { ...strategy, filePatterns: nextPatterns };
                        onChange({ translationStrategies: next });
                      }}
                      type="button"
                    >
                      Remove
                    </button>
                  </div>
                ))}
                <button
                  className="button button--ghost"
                  onClick={() => {
                    const next = [...strategies];
                    next[index] = {
                      ...strategy,
                      filePatterns: [...strategy.filePatterns, { pattern: "*" }],
                    };
                    onChange({ translationStrategies: next });
                  }}
                  type="button"
                >
                  Add Pattern
                </button>
              </div>
            </article>
          ))}
        </div>
      </div>
    </section>
  );
}
