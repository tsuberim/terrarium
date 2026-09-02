use wasmparser::{Parser, Payload};

const ALLOWED_IMPORT_MODULE: &str = "terrarium";

pub fn validate_wasm(wasm: &[u8]) -> Result<(), String> {
    if wasm.len() < 8 || &wasm[0..4] != b"\0asm" {
        return Err("invalid WASM magic".into());
    }

    let mut has_tick_export = false;
    let parser = Parser::new(0);
    for payload in parser.parse_all(wasm) {
        let payload = payload.map_err(|e| format!("wasm parse error: {e}"))?;
        match payload {
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import.map_err(|e| format!("import error: {e}"))?;
                    if import.module != ALLOWED_IMPORT_MODULE {
                        return Err(format!(
                            "disallowed import module `{}` (only `{ALLOWED_IMPORT_MODULE}` allowed)",
                            import.module
                        ));
                    }
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export.map_err(|e| format!("export error: {e}"))?;
                    if export.name == "tick" {
                        has_tick_export = true;
                    }
                }
            }
            _ => {}
        }
    }

    if !has_tick_export {
        return Err("WASM must export `tick`".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_bad_magic() {
        assert!(validate_wasm(b"not wasm").is_err());
    }
}
