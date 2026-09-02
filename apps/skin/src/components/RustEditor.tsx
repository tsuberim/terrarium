import Editor, { type Monaco } from "@monaco-editor/react";
import type { editor } from "monaco-editor";
import { useEffect, useRef } from "react";
import type { CompileDiagnostic } from "../lib/creatureEditor";

type Props = {
  value: string;
  onChange: (value: string) => void;
  diagnostics: CompileDiagnostic[];
  readOnly?: boolean;
};

export function RustEditor({ value, onChange, diagnostics, readOnly }: Props) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  const monacoRef = useRef<Monaco | null>(null);

  useEffect(() => {
    const ed = editorRef.current;
    const monaco = monacoRef.current;
    const model = ed?.getModel();
    if (!ed || !monaco || !model) return;
    monaco.editor.setModelMarkers(
      model,
      "terrarium",
      diagnostics.map((d) => ({
        severity:
          d.level === "error"
            ? monaco.MarkerSeverity.Error
            : d.level === "warning"
              ? monaco.MarkerSeverity.Warning
              : monaco.MarkerSeverity.Info,
        message: d.message,
        startLineNumber: d.line ?? 1,
        startColumn: d.column ?? 1,
        endLineNumber: d.line ?? 1,
        endColumn: (d.column ?? 1) + 1,
      })),
    );
  }, [diagnostics]);

  return (
    <Editor
      height="220px"
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
      }}
      onMount={(ed, monaco) => {
        editorRef.current = ed;
        monacoRef.current = monaco;
      }}
      loading={<div className="p-3 text-[11px] text-white/40">Loading editor…</div>}
    />
  );
}
