use std::fmt;

use crate::error::{ApiError, ApiResult};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TokenScopes {
    pub spawn: bool,
    pub read: bool,
}

impl TokenScopes {
    pub fn all() -> Self {
        Self {
            spawn: true,
            read: true,
        }
    }

    pub fn from_parts(spawn: bool, read: bool) -> Self {
        Self { spawn, read }
    }

    pub fn from_db(raw: &str) -> Self {
        let mut scopes = Self::default();
        for part in raw.split(',') {
            match part.trim() {
                "spawn" => scopes.spawn = true,
                "read" => scopes.read = true,
                _ => {}
            }
        }
        scopes
    }

    pub fn from_request(values: &[String]) -> ApiResult<Self> {
        if values.is_empty() {
            return Ok(Self::all());
        }
        let mut scopes = Self::default();
        for v in values {
            match v.as_str() {
                "spawn" => scopes.spawn = true,
                "read" => scopes.read = true,
                other => {
                    return Err(ApiError::BadRequest(format!("unknown scope: {other}")));
                }
            }
        }
        if !scopes.spawn && !scopes.read {
            return Err(ApiError::BadRequest("at least one scope required".into()));
        }
        Ok(scopes)
    }

    pub fn to_db_string(self) -> String {
        let mut parts = Vec::new();
        if self.spawn {
            parts.push("spawn");
        }
        if self.read {
            parts.push("read");
        }
        parts.join(",")
    }

    pub fn require_spawn(self) -> ApiResult<()> {
        if self.spawn {
            Ok(())
        } else {
            Err(ApiError::Forbidden)
        }
    }

    pub fn require_read(self) -> ApiResult<()> {
        if self.read {
            Ok(())
        } else {
            Err(ApiError::Forbidden)
        }
    }
}

impl fmt::Display for TokenScopes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_db_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_db_string() {
        let s = TokenScopes::all();
        assert_eq!(TokenScopes::from_db(&s.to_db_string()), s);
    }
}
