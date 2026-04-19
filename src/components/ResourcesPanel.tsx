import { useEffect, useState } from "react";

import { commands } from "../lib/tauri";
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

export function ResourcesPanel({
  promptFiles,
  terminologyFiles,
  blacklistConfig,
  onChangeBlacklist,
  onPersistBlacklist,
  setError,
}: ResourcesPanelProps) {
  const [selectedPrompt, setSelectedPrompt] = useState(promptFiles[0]?.path ?? "");
  const [selectedTerminology, setSelectedTerminology] = useState(terminologyFiles[0]?.path ?? "");
  const [promptText, setPromptText] = useState("");
  const [terminology, setTerminology] = useState<TerminologyDictionary>(defaultTerminology);

  useEffect(() => {
    if (!selectedPrompt && promptFiles[0]) {
      setSelectedPrompt(promptFiles[0].path);
    }
  }, [promptFiles, selectedPrompt]);

  useEffect(() => {
    if (!selectedTerminology && terminologyFiles[0]) {
      setSelectedTerminology(terminologyFiles[0].path);
    }
  }, [terminologyFiles, selectedTerminology]);

  useEffect(() => {
    if (!selectedPrompt) {
      return;
    }
    void commands
      .loadTextResource(selectedPrompt)
      .then((payload) => {
        setPromptText(payload.content);
        setError(null);
      })
      .catch((error) => setError(error as UserFacingError));
  }, [selectedPrompt, setError]);

  useEffect(() => {
    if (!selectedTerminology) {
      return;
    }
    void commands
      .loadTextResource(selectedTerminology)
      .then((payload) => {
        setTerminology(JSON.parse(payload.content) as TerminologyDictionary);
        setError(null);
      })
      .catch((error) => setError(error as UserFacingError));
  }, [selectedTerminology, setError]);

  return (
    <section className="panel-stack">
      <div className="two-column">
        <section className="panel">
          <div className="panel__header">
            <div>
              <p className="eyebrow">Prompts</p>
              <h3>Text editor</h3>
            </div>
            <button
              className="button"
              onClick={() =>
                commands
                  .saveTextResource({ path: selectedPrompt, content: promptText })
                  .catch((error) => setError(error as UserFacingError))
              }
              type="button"
            >
              Save Prompt
            </button>
          </div>
          <select value={selectedPrompt} onChange={(event) => setSelectedPrompt(event.target.value)}>
            {promptFiles.map((file) => (
              <option key={file.path} value={file.path}>
                {file.path}
              </option>
            ))}
          </select>
          <textarea className="editor-textarea" value={promptText} onChange={(event) => setPromptText(event.target.value)} />
        </section>

        <section className="panel">
          <div className="panel__header">
            <div>
              <p className="eyebrow">Terminology</p>
              <h3>Dictionary editor</h3>
            </div>
            <button
              className="button"
              onClick={() =>
                commands
                  .saveTerminology(selectedTerminology, terminology)
                  .catch((error) => setError(error as UserFacingError))
              }
              type="button"
            >
              Save Terminology
            </button>
          </div>
          <select
            value={selectedTerminology}
            onChange={(event) => setSelectedTerminology(event.target.value)}
          >
            {terminologyFiles.map((file) => (
              <option key={file.path} value={file.path}>
                {file.path}
              </option>
            ))}
          </select>
          <div className="dictionary-list">
            {Object.entries(terminology.terminology).map(([key, value]) => (
              <div className="dictionary-row" key={key}>
                <input
                  value={key}
                  onChange={(event) => {
                    const next = { ...terminology.terminology };
                    delete next[key];
                    next[event.target.value] = value;
                    setTerminology({ terminology: next });
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
                    const next = { ...terminology.terminology };
                    delete next[key];
                    setTerminology({ terminology: next });
                  }}
                  type="button"
                >
                  Remove
                </button>
              </div>
            ))}
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
              Add Term
            </button>
          </div>
        </section>
      </div>

      <div className="panel">
        <div className="panel__header">
          <div>
            <p className="eyebrow">Blacklist</p>
            <h3>Protected field names</h3>
          </div>
          <button className="button" onClick={() => onPersistBlacklist()} type="button">
            Save BlackList.json
          </button>
        </div>
        <textarea
          className="editor-textarea"
          value={blacklistConfig.blacklist.join("\n")}
          onChange={(event) =>
            onChangeBlacklist({
              blacklist: event.target.value.split("\n").map((value) => value.trim()).filter(Boolean),
            })
          }
        />
      </div>
    </section>
  );
}
