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

/// Guest `Action` slot offset (`ABI_ACTION`).
pub const ABI_ACTION_OFF: i32 = 8328;

pub const WAT_IDLE: &str = r#"
(module
  (import "terrarium" "act" (func $act (result i32)))
  (memory (export "memory") 1)
  (func (export "main")
    loop $l
      br $l
    end)
)
"#;

pub const WAT_MOVE_FWD_LOOP: &str = r#"
(module
  (import "terrarium" "act" (func $act (result i32)))
  (memory (export "memory") 1)
  (func (export "main")
    loop $l
      i32.const 8328
      i32.const 1
      i32.store
      call $act
      drop
      br $l
    end)
)
"#;

pub fn wat_move_once(rel: u8) -> String {
    format!(
        r#"
(module
  (import "terrarium" "act" (func $act (result i32)))
  (memory (export "memory") 1)
  (func (export "main")
    i32.const 8328
    i32.const 1
    i32.store
    i32.const 8332
    i32.const {rel}
    i32.store8
    call $act
    drop)
)
"#
    )
}

#[cfg(test)]
mod tests {
    use super::{compile_wat, WAT_IDLE};

    #[test]
    fn compiles_minimal_module() {
        let wasm = compile_wat(WAT_IDLE).unwrap();
        assert!(wasm.starts_with(b"\0asm"));
    }
}
