import { useEffect, useState } from "react";

import { commands } from "../lib/tauri";
import { t } from "../lib/i18n";
import type {
  BlacklistConfig,
  ResourceFile,
  TerminologyDictionary,
  UserFacingError,
} from "../types/app";

interface ResourcesPanelProps {
  promptFiles: ResourceFile[];
  terminologyFiles: ResourceFile[];
  blacklistConfig: BlacklistConfig;
  onChangeBlacklist: (config: BlacklistConfig) => void;
  onPersistBlacklist: () => void;
  setError: (error: UserFacingError | null) => void;
}

function defaultTerminology(): TerminologyDictionary {
  return { terminology: {} };
}

function normalizeTerminology(payload: unknown): {
  data: TerminologyDictionary;
  error: UserFacingError | null;
} {
  if (
    payload &&
    typeof payload === "object" &&
    "terminology" in payload &&
    payload.terminology &&
    typeof payload.terminology === "object" &&
    !Array.isArray(payload.terminology)
  ) {
    const terminology = Object.fromEntries(
      Object.entries(payload.terminology as Record<string, unknown>).map(
        ([key, value]) => [key, String(value ?? "")],
      ),
    );
    return {
      data: {
        terminology,
      },
      error: null,
    };
  }

  return {
    data: defaultTerminology(),
    error: {
      title: t("invalidResourceTitle"),
      message: t("invalidTerminologyJson"),
      details:
        "Expected an object shaped like { terminology: { key: value } }.",
    },
  };
}

function renameTerminologyKey(
  source: Record<string, string>,
  index: number,
  nextKey: string,
): Record<string, string> {
  return Object.fromEntries(
    Object.entries(source).map(([key, value], entryIndex) =>
      entryIndex === index ? [nextKey, value] : [key, value],
    ),
  );
}

export function ResourcesPanel({
  promptFiles,
  terminologyFiles,
  blacklistConfig,
  onChangeBlacklist,
  onPersistBlacklist,
  setError,
}: ResourcesPanelProps) {
  const [selectedPrompt, setSelectedPrompt] = useState(
    promptFiles[0]?.path ?? "",
  );
  const [selectedTerminology, setSelectedTerminology] = useState(
    terminologyFiles[0]?.path ?? "",
  );
  const [promptText, setPromptText] = useState("");
  const [terminology, setTerminology] =
    useState<TerminologyDictionary>(defaultTerminology);
  const [promptStatus, setPromptStatus] = useState<string | null>(null);
  const [terminologyStatus, setTerminologyStatus] = useState<string | null>(
    null,
  );

  useEffect(() => {
    if (!selectedPrompt && promptFiles[0]) {
      setSelectedPrompt(promptFiles[0].path);
    } else if (
      selectedPrompt &&
      !promptFiles.some((file) => file.path === selectedPrompt)
    ) {
      setSelectedPrompt(promptFiles[0]?.path ?? "");
    }
  }, [promptFiles, selectedPrompt]);

  useEffect(() => {
    if (!selectedTerminology && terminologyFiles[0]) {
      setSelectedTerminology(terminologyFiles[0].path);
    } else if (
      selectedTerminology &&
      !terminologyFiles.some((file) => file.path === selectedTerminology)
    ) {
      setSelectedTerminology(terminologyFiles[0]?.path ?? "");
    }
  }, [terminologyFiles, selectedTerminology]);

  useEffect(() => {
    if (!selectedPrompt) {
      setPromptText("");
      setPromptStatus(t("noPromptFiles"));
      return;
    }
    setPromptStatus(t("loadingResource"));
    void commands
      .loadTextResource(selectedPrompt)
      .then((payload) => {
        setPromptText(payload.content);
        setPromptStatus(null);
        setError(null);
      })
      .catch((error) => {
        const nextError = error as UserFacingError;
        setPromptText("");
        setPromptStatus(nextError.message);
        setError(nextError);
      });
  }, [selectedPrompt, setError]);

  useEffect(() => {
    if (!selectedTerminology) {
      setTerminology(defaultTerminology());
      setTerminologyStatus(t("noTerminologyFiles"));
      return;
    }
    setTerminologyStatus(t("loadingResource"));
    void commands
      .loadTextResource(selectedTerminology)
      .then((payload) => {
        try {
          const parsed = JSON.parse(payload.content) as unknown;
          const normalized = normalizeTerminology(parsed);
          setTerminology(normalized.data);
          if (normalized.error) {
            setTerminologyStatus(t("invalidTerminologyJson"));
            setError(normalized.error);
            return;
          }
          setTerminologyStatus(null);
          setError(null);
        } catch (error) {
          setTerminology(defaultTerminology());
          setTerminologyStatus(t("invalidTerminologyJson"));
          setError({
            title: t("invalidResourceTitle"),
            message: t("invalidTerminologyJson"),
            details: error instanceof Error ? error.message : String(error),
          });
        }
      })
      .catch((error) => {
        const nextError = error as UserFacingError;
        setTerminology(defaultTerminology());
        setTerminologyStatus(nextError.message);
        setError(nextError);
      });
  }, [selectedTerminology, setError]);

  return (
    <section className="panel-stack">
      <div className="two-column">
        <section className="panel">
          <div className="panel__header">
            <div>
              <p className="eyebrow">{t("prompts")}</p>
              <h3>{t("textEditor")}</h3>
            </div>
            <button
              className="button"
              disabled={!selectedPrompt}
              onClick={() =>
                commands
                  .saveTextResource({
                    path: selectedPrompt,
                    content: promptText,
                  })
                  .catch((error) => setError(error as UserFacingError))
              }
              type="button"
            >
              {t("savePrompt")}
            </button>
          </div>
          <select
            disabled={!promptFiles.length}
            value={selectedPrompt}
            onChange={(event) => setSelectedPrompt(event.target.value)}
          >
            {promptFiles.map((file) => (
              <option key={file.path} value={file.path}>
                {file.path}
              </option>
            ))}
          </select>
          {promptStatus ? (
            <div className="resource-empty-state">
              <strong>{promptStatus}</strong>
              <p>{t("resourcePanelHint")}</p>
            </div>
          ) : (
            <textarea
              className="editor-textarea"
              value={promptText}
              onChange={(event) => setPromptText(event.target.value)}
            />
          )}
        </section>

        <section className="panel">
          <div className="panel__header">
            <div>
              <p className="eyebrow">{t("terminology")}</p>
              <h3>{t("dictionaryEditor")}</h3>
            </div>
            <button
              className="button"
              disabled={!selectedTerminology}
              onClick={() =>
                commands
                  .saveTerminology(selectedTerminology, terminology)
                  .catch((error) => setError(error as UserFacingError))
              }
              type="button"
            >
              {t("saveTerminology")}
            </button>
          </div>
          <select
            disabled={!terminologyFiles.length}
            value={selectedTerminology}
            onChange={(event) => setSelectedTerminology(event.target.value)}
          >
            {terminologyFiles.map((file) => (
              <option key={file.path} value={file.path}>
                {file.path}
              </option>
            ))}
          </select>
          {terminologyStatus ? (
            <div className="resource-empty-state">
              <strong>{terminologyStatus}</strong>
              <p>{t("resourcePanelHint")}</p>
            </div>
          ) : (
            <div className="dictionary-list">
              {Object.entries(terminology.terminology).map(
                ([key, value], index) => (
                  <div className="dictionary-row" key={`term-row-${index}`}>
                    <input
                      value={key}
                      onChange={(event) => {
                        setTerminology({
                          terminology: renameTerminologyKey(
                            terminology.terminology,
                            index,
                            event.target.value,
                          ),
                        });
                      }}
                    />
                    <input
                      value={value}
                      onChange={(event) =>
                        setTerminology({
                          terminology: {
                            ...terminology.terminology,
                            [key]: event.target.value,
                          },
                        })
                      }
                    />
                    <button
                      className="button button--ghost"
                      onClick={() => {
                        setTerminology({
                          terminology: Object.fromEntries(
                            Object.entries(terminology.terminology).filter(
                              (_, entryIndex) => entryIndex !== index,
                            ),
                          ),
                        });
                      }}
                      type="button"
                    >
                      {t("remove")}
                    </button>
                  </div>
                ),
              )}
              <button
                className="button button--ghost"
                onClick={() =>
                  setTerminology({
                    terminology: {
                      ...terminology.terminology,
                      "new-term": "",
                    },
                  })
                }
                type="button"
              >
                {t("addTerm")}
              </button>
            </div>
          )}
        </section>
      </div>

      <div className="panel">
        <div className="panel__header">
          <div>
            <p className="eyebrow">{t("blacklist")}</p>
            <h3>{t("protectedFieldNames")}</h3>
          </div>
          <button
            className="button"
            onClick={() => onPersistBlacklist()}
            type="button"
          >
            {t("saveBlacklist")}
          </button>
        </div>
        <textarea
          className="editor-textarea"
          value={blacklistConfig.blacklist.join("\n")}
          onChange={(event) =>
            onChangeBlacklist({
              blacklist: event.target.value
                .split("\n")
                .map((value) => value.trim())
                .filter(Boolean),
            })
          }
        />
      </div>
    </section>
  );
}
