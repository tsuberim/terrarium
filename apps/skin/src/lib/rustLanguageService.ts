import type { Monaco } from "@monaco-editor/react";
import type { languages, Position } from "monaco-editor";
import type { CompileDiagnostic } from "./creatureEditor";

export type SdkSymbol = {
  name: string;
  kind: "fn" | "const" | "mod" | "macro";
  signature: string;
  returns?: string;
  doc: string;
};

export const SDK_SYMBOLS: SdkSymbol[] = [
  { name: "move_forward", kind: "fn", signature: "move_forward()", returns: "i32", doc: "Step forward one hex. Returns `0` on success." },
  { name: "eat_forward", kind: "fn", signature: "eat_forward()", returns: "i32", doc: "Eat food or corpse in front." },
  { name: "rotate", kind: "fn", signature: "rotate(delta: i32)", returns: "i32", doc: "Turn by `delta` hex directions (±1 = 60°)." },
  { name: "sense_kind", kind: "fn", signature: "sense_kind(dq: i32, dr: i32)", returns: "i32", doc: "Tile kind at relative hex, or `-1` out of vision." },
  { name: "energy", kind: "fn", signature: "energy()", returns: "i64", doc: "Current glim balance." },
  { name: "sleep", kind: "fn", signature: "sleep()", returns: "()", doc: "Yield remaining gas this step (optional)." },
  { name: "random_byte", kind: "fn", signature: "random_byte()", returns: "u8", doc: "Deterministic RNG byte." },
  { name: "tile", kind: "mod", signature: "tile", doc: "Tile kind constants: `EMPTY`, `SOLID`, `CREATURE`, `CORPSE`, `FOOD`." },
  { name: "tile::EMPTY", kind: "const", signature: "tile::EMPTY", returns: "i32", doc: "Empty cell." },
  { name: "tile::SOLID", kind: "const", signature: "tile::SOLID", returns: "i32", doc: "Wall / solid tile." },
  { name: "tile::CREATURE", kind: "const", signature: "tile::CREATURE", returns: "i32", doc: "Another creature." },
  { name: "tile::CORPSE", kind: "const", signature: "tile::CORPSE", returns: "i32", doc: "Corpse tile." },
  { name: "tile::FOOD", kind: "const", signature: "tile::FOOD", returns: "i32", doc: "Food pellet." },
  { name: "terrarium::scenario", kind: "macro", signature: "#[terrarium::scenario]", doc: "Mark a sandbox scenario. Optional arg: `#[terrarium::scenario(wall_ahead)]`." },
];

const SYMBOL_MAP = new Map(SDK_SYMBOLS.flatMap((s) => {
  const keys = [s.name];
  if (s.name.includes("::")) keys.push(s.name.split("::").pop()!);
  return keys.map((k) => [k, s] as const);
}));

function wordAt(model: languages.TextModel, position: Position): string {
  const word = model.getWordAtPosition(position);
  return word?.word ?? "";
}

function hoverMarkdown(sym: SdkSymbol): string {
  const ret = sym.returns ? `\n\n\`\`\`rust\n-> ${sym.returns}\n\`\`\`` : "";
  return `**${sym.signature}**${ret}\n\n${sym.doc}`;
}

function lookupSymbol(word: string, lineText: string): SdkSymbol | undefined {
  if (lineText.includes("tile::") && word.startsWith("EMPTY")) return SYMBOL_MAP.get("tile::EMPTY");
  if (lineText.includes("tile::") && word === "SOLID") return SYMBOL_MAP.get("tile::SOLID");
  if (lineText.includes("tile::") && word === "CREATURE") return SYMBOL_MAP.get("tile::CREATURE");
  if (lineText.includes("tile::") && word === "CORPSE") return SYMBOL_MAP.get("tile::CORPSE");
  if (lineText.includes("tile::") && word === "FOOD") return SYMBOL_MAP.get("tile::FOOD");
  if (lineText.includes("terrarium::scenario")) return SYMBOL_MAP.get("terrarium::scenario");
  return SYMBOL_MAP.get(word);
}

let registered = false;

export function setupRustLanguageService(monaco: Monaco, getDiagnostics: () => CompileDiagnostic[]) {
  if (registered) return;
  registered = true;

  monaco.languages.registerHoverProvider("rust", {
    provideHover(model, position) {
      const lineText = model.getLineContent(position.lineNumber);
      const word = wordAt(model, position);
      if (!word) return null;

      const sym = lookupSymbol(word, lineText);
      if (sym) {
        return {
          range: model.getWordAtPosition(position) ?? undefined,
          contents: [{ value: hoverMarkdown(sym) }],
        };
      }

      const diags = getDiagnostics().filter((d) => d.line === position.lineNumber);
      if (diags.length) {
        const text = diags.map((d) => `**${d.level}**: ${d.message}`).join("\n\n");
        return { contents: [{ value: text }] };
      }

      if (lineText.includes("fn ") && word.match(/^[a-z_]/)) {
        return {
          contents: [{
            value: `**fn ${word}()**\n\nUser-defined. Run compile for type errors and full rustc diagnostics.`,
          }],
        };
      }

      return null;
    },
  });

  monaco.languages.registerCompletionItemProvider("rust", {
    triggerCharacters: [":", "#", "("],
    provideCompletionItems(model, position) {
      const word = model.getWordUntilPosition(position);
      const range = {
        startLineNumber: position.lineNumber,
        endLineNumber: position.lineNumber,
        startColumn: word.startColumn,
        endColumn: word.endColumn,
      };
      const suggestions: languages.CompletionItem[] = SDK_SYMBOLS.map((s) => ({
        label: s.name,
        kind:
          s.kind === "fn"
            ? monaco.languages.CompletionItemKind.Function
            : s.kind === "const"
              ? monaco.languages.CompletionItemKind.Constant
              : s.kind === "macro"
                ? monaco.languages.CompletionItemKind.Snippet
                : monaco.languages.CompletionItemKind.Module,
        insertText: s.kind === "macro" ? "#[terrarium::scenario]\nfn ${1:name}() {}" : s.name,
        insertTextRules:
          s.kind === "macro" ? monaco.languages.CompletionItemInsertTextRule.InsertAsSnippet : undefined,
        detail: s.signature,
        documentation: s.doc,
        range,
      }));
      return { suggestions };
    },
  });

  monaco.languages.registerSignatureHelpProvider("rust", {
    signatureHelpTriggerCharacters: ["("],
    provideSignatureHelp(model, position) {
      const line = model.getLineContent(position.lineNumber).slice(0, position.column - 1);
      const match = /(\w+)\($/.exec(line);
      const name = match?.[1];
      const sym = name ? SDK_SYMBOLS.find((s) => s.kind === "fn" && s.name.startsWith(name)) : undefined;
      if (!sym) return null;
      return {
        value: {
          signatures: [{ label: sym.signature, documentation: sym.doc, parameters: [] }],
          activeSignature: 0,
          activeParameter: 0,
        },
        dispose: () => {},
      };
    },
  });
}

export function diagnosticMarkers(monaco: Monaco, diagnostics: CompileDiagnostic[]) {
  return diagnostics.map((d) => {
    const line = d.line ?? 1;
    const col = d.column ?? 1;
    return {
      severity:
        d.level === "error"
          ? monaco.MarkerSeverity.Error
          : d.level === "warning"
            ? monaco.MarkerSeverity.Warning
            : monaco.MarkerSeverity.Info,
      message: d.message,
      startLineNumber: line,
      startColumn: col,
      endLineNumber: line,
      endColumn: Math.max(col + 1, 200),
      source: "rustc",
    };
  });
}
