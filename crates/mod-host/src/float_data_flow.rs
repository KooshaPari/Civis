//! Float data-flow trace for `civlab::action_emit` call sites (CIV-0700 §3.5 / §14.5).

use serde::{Deserialize, Serialize};
use wasmparser::{Operator, Parser, Payload, TypeRef};

/// A site where a float-derived value may reach `action_emit`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FloatContaminationSite {
    /// Function index in the module (imports + defined).
    pub function_index: u32,
    /// Opcode index within the function body (for diagnostics).
    pub instruction_index: u32,
    /// Human-readable reason.
    pub reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StackSlot {
    /// Plain integer pipeline.
    Int,
    /// Active float value.
    Float,
    /// `i64`/`i32` produced from float bits without trunc (e.g. `reinterpret`).
    FloatDerivedInt,
}

fn collect_action_emit_import_index(wasm_bytes: &[u8]) -> Result<Option<u32>, String> {
    let mut import_funcs: Vec<(String, String)> = Vec::new();
    for payload in Parser::new(0).parse_all(wasm_bytes) {
        let payload = payload.map_err(|e| e.to_string())?;
        if let Payload::ImportSection(imports) = payload {
            for import in imports.into_imports() {
                let import = import.map_err(|e| e.to_string())?;
                if matches!(import.ty, TypeRef::Func(_) | TypeRef::FuncExact(_)) {
                    import_funcs.push((import.module.to_string(), import.name.to_string()));
                }
            }
        }
    }
    Ok(import_funcs
        .iter()
        .position(|(module, name)| module == "civlab" && name == "action_emit")
        .map(|idx| u32::try_from(idx).expect("import index fits u32")))
}

/// Scan WASM for float contamination at `civlab::action_emit` imports.
pub fn scan_float_action_emit_contamination(
    wasm_bytes: &[u8],
) -> Result<Vec<FloatContaminationSite>, String> {
    let Some(action_emit_idx) = collect_action_emit_import_index(wasm_bytes)? else {
        return Ok(Vec::new());
    };

    let import_count = {
        let mut count = 0u32;
        for payload in Parser::new(0).parse_all(wasm_bytes) {
            let payload = payload.map_err(|e| e.to_string())?;
            if let Payload::ImportSection(imports) = payload {
                for import in imports.into_imports() {
                    let import = import.map_err(|e| e.to_string())?;
                    if matches!(import.ty, TypeRef::Func(_) | TypeRef::FuncExact(_)) {
                        count += 1;
                    }
                }
            }
        }
        count
    };

    let mut sites = Vec::new();
    let mut defined_idx = 0u32;
    for payload in Parser::new(0).parse_all(wasm_bytes) {
        let payload = payload.map_err(|e| e.to_string())?;
        if let Payload::CodeSectionEntry(body) = payload {
            let function_index = import_count.saturating_add(defined_idx);
            scan_function_body(body, function_index, action_emit_idx, &mut sites)?;
            defined_idx += 1;
        }
    }
    Ok(sites)
}

fn scan_function_body(
    body: wasmparser::FunctionBody<'_>,
    function_index: u32,
    action_emit_idx: u32,
    sites: &mut Vec<FloatContaminationSite>,
) -> Result<(), String> {
    let mut reader = body.get_operators_reader().map_err(|e| e.to_string())?;
    let mut stack: Vec<StackSlot> = Vec::new();
    let mut instruction_index = 0u32;

    while !reader.eof() {
        let op = reader.read().map_err(|e| e.to_string())?;
        instruction_index += 1;

        match op {
            Operator::F32Const { .. } | Operator::F64Const { .. } => stack.push(StackSlot::Float),
            Operator::I32Const { .. }
            | Operator::I64Const { .. }
            | Operator::I32Add
            | Operator::I64Add
            | Operator::I32Sub
            | Operator::I64Sub
            | Operator::I32Mul
            | Operator::I64Mul => stack.push(StackSlot::Int),
            Operator::I32TruncF32S
            | Operator::I32TruncF32U
            | Operator::I32TruncF64S
            | Operator::I32TruncF64U
            | Operator::I64TruncF32S
            | Operator::I64TruncF32U
            | Operator::I64TruncF64S
            | Operator::I64TruncF64U
                if stack.pop().is_some() =>
            {
                stack.push(StackSlot::Int);
            }
            Operator::I32ReinterpretF32 | Operator::I64ReinterpretF64 if stack.pop().is_some() => {
                stack.push(StackSlot::FloatDerivedInt);
            }
            Operator::Drop => {
                stack.pop();
            }
            Operator::Call {
                function_index: callee,
            } if callee == action_emit_idx => {
                if stack.last().is_some_and(|slot| {
                    matches!(slot, StackSlot::Float | StackSlot::FloatDerivedInt)
                }) {
                    sites.push(FloatContaminationSite {
                        function_index,
                        instruction_index,
                        reason: "float-derived value passed to civlab::action_emit".to_owned(),
                    });
                }
                stack.pop();
            }
            Operator::Call { .. } => {
                stack.pop();
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_action_emit_import_is_clean() {
        const WAT: &str = r#"
            (module
              (func (export "civlab_policy_tick") (param i64) (result i32)
                i32.const 0)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let sites = scan_float_action_emit_contamination(&wasm).expect("scan");
        assert!(sites.is_empty());
    }

    #[test]
    fn rejects_reinterpret_f64_before_action_emit() {
        const WAT: &str = r#"
            (module
              (import "civlab" "action_emit" (func (param i64)))
              (func (export "civlab_policy_tick") (param i64) (result i32)
                f64.const 1.0
                i64.reinterpret_f64
                call 0
                i32.const 0)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let sites = scan_float_action_emit_contamination(&wasm).expect("scan");
        assert_eq!(sites.len(), 1);
        assert!(sites[0].reason.contains("action_emit"));
    }

    #[test]
    fn trunc_before_action_emit_is_clean() {
        const WAT: &str = r#"
            (module
              (import "civlab" "action_emit" (func (param i64)))
              (func (export "civlab_policy_tick") (param i64) (result i32)
                f64.const 1.0
                i64.trunc_f64_s
                call 0
                i32.const 0)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let sites = scan_float_action_emit_contamination(&wasm).expect("scan");
        assert!(sites.is_empty());
    }
}
