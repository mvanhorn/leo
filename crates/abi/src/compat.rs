// Copyright (C) 2019-2026 Provable Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

//! Interface-compatibility checking between two program ABIs.
//!
//! A candidate program is *compatible* with an interface standard when every item the standard
//! declares is present in the candidate with an identical definition — that is, when the
//! standard's public interface is a subset of the candidate's. The candidate may declare
//! additional items beyond the standard.

use leo_abi_types as abi;

/// Returns the list of reasons `candidate` is not compatible with the interface standard
/// `standard`. An empty list means the candidate is compatible. Each item the standard declares
/// must appear in the candidate (by name, or by path for records and structs) with an identical
/// definition.
pub fn check_compatibility(candidate: &abi::Program, standard: &abi::Program) -> Vec<String> {
    let mut problems = Vec::new();

    for s in &standard.functions {
        match candidate.functions.iter().find(|c| c.name == s.name) {
            None => problems.push(format!("missing function `{}`", s.name)),
            Some(c) if c == s => {}
            Some(_) => problems.push(format!("function `{}` differs", s.name)),
        }
    }

    for s in &standard.views {
        match candidate.views.iter().find(|c| c.name == s.name) {
            None => problems.push(format!("missing view `{}`", s.name)),
            Some(c) if c == s => {}
            Some(_) => problems.push(format!("view `{}` differs", s.name)),
        }
    }

    for s in &standard.mappings {
        match candidate.mappings.iter().find(|c| c.name == s.name) {
            None => problems.push(format!("missing mapping `{}`", s.name)),
            Some(c) if c == s => {}
            Some(_) => problems.push(format!("mapping `{}` differs", s.name)),
        }
    }

    for s in &standard.storage_variables {
        match candidate.storage_variables.iter().find(|c| c.name == s.name) {
            None => problems.push(format!("missing storage variable `{}`", s.name)),
            Some(c) if c == s => {}
            Some(_) => problems.push(format!("storage variable `{}` differs", s.name)),
        }
    }

    for s in &standard.records {
        match candidate.records.iter().find(|c| c.path == s.path) {
            None => problems.push(format!("missing record `{}`", s.path.join("::"))),
            Some(c) if c == s => {}
            Some(_) => problems.push(format!("record `{}` differs", s.path.join("::"))),
        }
    }

    for s in &standard.structs {
        match candidate.structs.iter().find(|c| c.path == s.path) {
            None => problems.push(format!("missing struct `{}`", s.path.join("::"))),
            Some(c) if c == s => {}
            Some(_) => problems.push(format!("struct `{}` differs", s.path.join("::"))),
        }
    }

    problems
}

#[cfg(test)]
mod tests {
    use super::*;
    use abi::{
        Function,
        FunctionInput,
        FunctionOutput,
        Mapping,
        Mode,
        Plaintext,
        Primitive,
        Program,
        Record,
        RecordField,
        UInt,
    };

    fn u64t() -> Plaintext {
        Plaintext::Primitive(Primitive::UInt(UInt::U64))
    }

    fn u32t() -> Plaintext {
        Plaintext::Primitive(Primitive::UInt(UInt::U32))
    }

    fn addr() -> Plaintext {
        Plaintext::Primitive(Primitive::Address)
    }

    fn input(ty: Plaintext, mode: Mode) -> FunctionInput {
        FunctionInput::Plaintext { ty, mode }
    }

    fn output(ty: Plaintext, mode: Mode) -> FunctionOutput {
        FunctionOutput::Plaintext { ty, mode }
    }

    fn func(name: &str, inputs: Vec<FunctionInput>, outputs: Vec<FunctionOutput>) -> Function {
        Function { name: name.to_string(), inputs, outputs }
    }

    fn program(name: &str, functions: Vec<Function>) -> Program {
        Program {
            program: name.to_string(),
            structs: Vec::new(),
            records: Vec::new(),
            mappings: Vec::new(),
            storage_variables: Vec::new(),
            functions,
            views: Vec::new(),
        }
    }

    /// `transfer(address.private, address.private, u64.public) -> ()`.
    fn transfer() -> Function {
        func(
            "transfer",
            vec![input(addr(), Mode::Private), input(addr(), Mode::Private), input(u64t(), Mode::Public)],
            vec![],
        )
    }

    #[test]
    fn identical_interfaces_are_compatible() {
        let problems =
            check_compatibility(&program("token.aleo", vec![transfer()]), &program("std.aleo", vec![transfer()]));
        assert!(problems.is_empty(), "{problems:?}");
    }

    #[test]
    fn candidate_superset_is_compatible() {
        let extra = func("mint", vec![input(addr(), Mode::Private), input(u64t(), Mode::Public)], vec![]);
        let candidate = program("token.aleo", vec![transfer(), extra]);
        assert!(check_compatibility(&candidate, &program("std.aleo", vec![transfer()])).is_empty());
    }

    #[test]
    fn missing_function_is_reported() {
        let problems = check_compatibility(&program("token.aleo", vec![]), &program("std.aleo", vec![transfer()]));
        assert_eq!(problems, vec!["missing function `transfer`".to_string()]);
    }

    #[test]
    fn input_type_mismatch_is_reported() {
        // Same name, but `amount` is u32 instead of u64.
        let candidate = program("token.aleo", vec![func(
            "transfer",
            vec![input(addr(), Mode::Private), input(addr(), Mode::Private), input(u32t(), Mode::Public)],
            vec![],
        )]);
        let problems = check_compatibility(&candidate, &program("std.aleo", vec![transfer()]));
        assert_eq!(problems, vec!["function `transfer` differs".to_string()]);
    }

    #[test]
    fn input_mode_mismatch_is_reported() {
        // `amount` is private instead of public.
        let candidate = program("token.aleo", vec![func(
            "transfer",
            vec![input(addr(), Mode::Private), input(addr(), Mode::Private), input(u64t(), Mode::Private)],
            vec![],
        )]);
        assert!(!check_compatibility(&candidate, &program("std.aleo", vec![transfer()])).is_empty());
    }

    #[test]
    fn output_mismatch_is_reported() {
        let standard = program("std.aleo", vec![func("balance_of", vec![input(addr(), Mode::Private)], vec![output(
            u64t(),
            Mode::Private,
        )])]);
        let candidate =
            program("token.aleo", vec![func("balance_of", vec![input(addr(), Mode::Private)], vec![output(
                u32t(),
                Mode::Private,
            )])]);
        assert!(!check_compatibility(&candidate, &standard).is_empty());
    }

    #[test]
    fn mapping_value_mismatch_is_reported() {
        let mut standard = program("std.aleo", vec![]);
        standard.mappings.push(Mapping { name: "balances".into(), key: addr(), value: u64t() });
        let mut candidate = program("token.aleo", vec![]);
        candidate.mappings.push(Mapping { name: "balances".into(), key: addr(), value: u32t() });
        assert_eq!(check_compatibility(&candidate, &standard), vec!["mapping `balances` differs".to_string()]);
    }

    #[test]
    fn record_field_layout_mismatch_is_reported() {
        let field = |name: &str, ty: Plaintext| RecordField { name: name.into(), ty, mode: Mode::Private };
        let mut standard = program("std.aleo", vec![]);
        standard
            .records
            .push(Record { path: vec!["Token".into()], fields: vec![field("owner", addr()), field("amount", u64t())] });
        let mut candidate = program("token.aleo", vec![]);
        candidate
            .records
            .push(Record { path: vec!["Token".into()], fields: vec![field("owner", addr()), field("amount", u32t())] });
        assert_eq!(check_compatibility(&candidate, &standard), vec!["record `Token` differs".to_string()]);
    }
}
