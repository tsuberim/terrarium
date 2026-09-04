use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WatError {
    #[error("program is empty")]
    Empty,
    #[error("parse error: {0}")]
    Parse(String),
}

pub fn compile_wat(source: &str) -> Result<Vec<u8>, WatError> {
    let source = source.trim();
    if source.is_empty() {
        return Err(WatError::Empty);
    }
    wat::parse_str(source).map_err(|e| WatError::Parse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDLE: &str = r#"
(module
  (import "terrarium" "sleep" (func $sleep))
  (func (export "main") (call $sleep))
)
"#;

    #[test]
    fn compiles_minimal_module() {
        let wasm = compile_wat(IDLE).unwrap();
        assert!(wasm.starts_with(b"\0asm"));
    }
}
