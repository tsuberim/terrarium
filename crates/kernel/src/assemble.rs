use std::collections::HashMap;

use thiserror::Error;

use crate::isa::op;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("line {line}: {message}")]
pub struct AssembleError {
    pub line: usize,
    pub message: String,
}

#[derive(Clone, Debug)]
struct Line<'a> {
    number: usize,
    text: &'a str,
}

pub fn assemble(source: &str) -> Result<Vec<u8>, AssembleError> {
    let lines = parse_lines(source);
    if lines.is_empty() {
        return Err(AssembleError {
            line: 1,
            message: "program is empty".into(),
        });
    }

    let mut labels: HashMap<String, usize> = HashMap::new();
    let mut ip = 0usize;

    for line in &lines {
        if let Some((label, _)) = split_label(line.text) {
            if labels.insert(label.to_string(), ip).is_some() {
                return Err(err(line.number, format!("duplicate label `{label}`")));
            }
        }
        ip += instruction_size(line.text, line.number)?;
    }

    let mut out = Vec::with_capacity(ip);
    for line in &lines {
        emit(&mut out, line, &labels)?;
    }

    Ok(out)
}

fn parse_lines(source: &str) -> Vec<Line<'_>> {
    source
        .lines()
        .enumerate()
        .filter_map(|(i, raw)| {
            let text = strip_comment(raw).trim();
            if text.is_empty() {
                None
            } else {
                Some(Line {
                    number: i + 1,
                    text,
                })
            }
        })
        .collect()
}

fn strip_comment(line: &str) -> &str {
    let line = line.split("//").next().unwrap_or(line);
    line.split(';').next().unwrap_or(line)
}

fn split_label(text: &str) -> Option<(&str, &str)> {
    if let Some((head, rest)) = text.split_once(':') {
        let label = head.trim();
        if label.is_empty() || !is_label(label) {
            return None;
        }
        Some((label, rest.trim()))
    } else {
        None
    }
}

fn is_label(s: &str) -> bool {
    s.chars()
        .next()
        .map(|c| c.is_ascii_alphabetic() || c == '_')
        .unwrap_or(false)
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn instruction_size(text: &str, line: usize) -> Result<usize, AssembleError> {
    let (_, rest) = split_label(text).unwrap_or(( "", text));
    if rest.is_empty() {
        return Ok(0);
    }
    let mut parts = rest.split_whitespace();
    let mnemonic = parts.next().ok_or_else(|| err(line, "expected mnemonic"))?;
    size_of(mnemonic, &parts.collect::<Vec<_>>(), line)
}

fn size_of(mnemonic: &str, args: &[&str], line: usize) -> Result<usize, AssembleError> {
    Ok(match mnemonic {
        "halt" | "sleep" | "energy" | "pop" | "dup" | "eq" | "lt" | "add" | "sub" | "suicide" => {
            expect_args(args, 0, mnemonic, line)?;
            1
        }
        "move" | "dig" | "place" | "eat" | "sense" => {
            expect_args(args, 1, mnemonic, line)?;
            parse_dir(args[0], line)?;
            2
        }
        "push" => {
            expect_args(args, 1, mnemonic, line)?;
            parse_imm(args[0], line)?;
            3
        }
        "jmp" | "jz" | "jnz" => {
            expect_args(args, 1, mnemonic, line)?;
            expect_label(args[0], line)?;
            3
        }
        _ => return Err(err(line, format!("unknown mnemonic `{mnemonic}`"))),
    })
}

fn emit(out: &mut Vec<u8>, line: &Line<'_>, labels: &HashMap<String, usize>) -> Result<(), AssembleError> {
    let (_, rest) = split_label(line.text).unwrap_or(( "", line.text));
    if rest.is_empty() {
        return Ok(());
    }
    let mut parts = rest.split_whitespace();
    let mnemonic = parts.next().unwrap();
    let args: Vec<&str> = parts.collect();

    match mnemonic {
        "halt" => out.push(op::HALT),
        "sleep" => out.push(op::SLEEP),
        "energy" => out.push(op::ENERGY),
        "pop" => out.push(op::POP),
        "dup" => out.push(op::DUP),
        "eq" => out.push(op::EQ),
        "lt" => out.push(op::LT),
        "add" => out.push(op::ADD),
        "sub" => out.push(op::SUB),
        "suicide" => out.push(op::SUICIDE),
        "move" | "dig" | "place" | "eat" | "sense" => {
            let opcode = match mnemonic {
                "move" => op::MOVE,
                "dig" => op::DIG,
                "place" => op::PLACE,
                "eat" => op::EAT,
                "sense" => op::SENSE,
                _ => unreachable!(),
            };
            out.push(opcode);
            out.push(parse_dir(args[0], line.number)?);
        }
        "push" => {
            out.push(op::PUSH);
            push_i16(out, parse_imm(args[0], line.number)?);
        }
        "jmp" | "jz" | "jnz" => {
            let opcode = match mnemonic {
                "jmp" => op::JMP,
                "jz" => op::JZ,
                "jnz" => op::JNZ,
                _ => unreachable!(),
            };
            out.push(opcode);
            let target = *labels
                .get(args[0])
                .ok_or_else(|| err(line.number, format!("unknown label `{}`", args[0])))?;
            let offset = (target as i32) - (out.len() as i32 + 2);
            if !(-32_768..=32_767).contains(&offset) {
                return Err(err(line.number, "jump out of range"));
            }
            push_i16(out, offset as i16);
        }
        _ => return Err(err(line.number, format!("unknown mnemonic `{mnemonic}`"))),
    }
    Ok(())
}

fn push_i16(out: &mut Vec<u8>, value: i16) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn parse_dir(s: &str, line: usize) -> Result<u8, AssembleError> {
    match s.to_ascii_lowercase().as_str() {
        "n" => Ok(crate::isa::dir::N),
        "e" => Ok(crate::isa::dir::E),
        "s" => Ok(crate::isa::dir::S),
        "w" => Ok(crate::isa::dir::W),
        _ => Err(err(line, format!("bad direction `{s}` (use n/e/s/w)"))),
    }
}

fn parse_imm(s: &str, line: usize) -> Result<i16, AssembleError> {
    s.parse::<i16>()
        .map_err(|_| err(line, format!("bad number `{s}`")))
}

fn expect_label(s: &str, line: usize) -> Result<(), AssembleError> {
    if is_label(s) {
        Ok(())
    } else {
        Err(err(line, format!("expected label, got `{s}`")))
    }
}

fn expect_args(args: &[&str], n: usize, mnemonic: &str, line: usize) -> Result<(), AssembleError> {
    if args.len() == n {
        Ok(())
    } else {
        Err(err(
            line,
            format!("`{mnemonic}` expects {n} argument(s), got {}", args.len()),
        ))
    }
}

fn err(line: usize, message: impl Into<String>) -> AssembleError {
    AssembleError {
        line,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_idle_loop() {
        let src = "loop:\n  sleep\n  jmp loop\n";
        let bytes = assemble(src).unwrap();
        assert_eq!(bytes, vec![op::SLEEP, op::JMP, 0xFC, 0xFF]); // jmp -4
    }

    #[test]
    fn strips_line_comments() {
        let src = "// header\nloop:\n  sleep ; tick\n  jmp loop\n";
        let bytes = assemble(src).unwrap();
        assert_eq!(bytes, vec![op::SLEEP, op::JMP, 0xFC, 0xFF]);
    }

    #[test]
    fn rejects_unknown_mnemonic() {
        let err = assemble("fly n\n").unwrap_err();
        assert_eq!(err.line, 1);
    }
}
