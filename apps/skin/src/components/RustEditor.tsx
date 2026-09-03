import Editor, { type Monaco } from "@monaco-editor/react";
import type { editor } from "monaco-editor";
import { useEffect, useRef } from "react";
import type { CompileDiagnostic } from "../lib/creatureEditor";
import { diagnosticMarkers, setupRustLanguageService } from "../lib/rustLanguageService";

type Props = {
  value: string;
  onChange: (value: string) => void;
  diagnostics: CompileDiagnostic[];
  readOnly?: boolean;
  height?: string | number;
};

export function RustEditor({ value, onChange, diagnostics, readOnly, height = "220px" }: Props) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  const monacoRef = useRef<Monaco | null>(null);
  const diagnosticsRef = useRef(diagnostics);
  diagnosticsRef.current = diagnostics;

  useEffect(() => {
    const ed = editorRef.current;
    const monaco = monacoRef.current;
    const model = ed?.getModel();
    if (!ed || !monaco || !model) return;

    monaco.editor.setModelMarkers(model, "terrarium", diagnosticMarkers(monaco, diagnostics));

    const first = diagnostics.find((d) => d.level === "error" && d.line != null);
    if (first?.line) {
      ed.revealLineInCenter(first.line);
      ed.setSelection({
        startLineNumber: first.line,
        startColumn: first.column ?? 1,
        endLineNumber: first.line,
        endColumn: first.column ?? 1,
      });
    }
  }, [diagnostics]);

  return (
    <Editor
      height={height}
      language="rust"
      theme="vs-dark"
      value={value}
      onChange={(v) => onChange(v ?? "")}
      options={{
        readOnly,
        minimap: { enabled: false },
        fontSize: 12,
        lineNumbers: "on",
        scrollBeyondLastLine: false,
        wordWrap: "on",
        padding: { top: 8, bottom: 8 },
        hover: { enabled: true, delay: 200, sticky: true },
        parameterHints: { enabled: true, cycle: true },
        quickSuggestions: { other: true, strings: false, comments: false },
        suggestOnTriggerCharacters: true,
        wordBasedSuggestions: "off",
        glyphMargin: true,
        renderValidationDecorations: "on",
        showUnused: true,
        tabCompletion: "on",
      }}
      onMount={(ed, monaco) => {
        editorRef.current = ed;
        monacoRef.current = monaco;
        setupRustLanguageService(monaco, () => diagnosticsRef.current);
        const model = ed.getModel();
        if (model) {
          monaco.editor.setModelMarkers(model, "terrarium", diagnosticMarkers(monaco, diagnosticsRef.current));
        }
      }}
      loading={<div className="p-3 text-[11px] text-white/40">Loading editor…</div>}
    />
  );
}
