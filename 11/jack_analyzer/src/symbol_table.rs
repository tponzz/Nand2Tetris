use std::collections::HashMap;

use crate::JAError;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum SymbolKind {
    Static,
    Field,
    Arg,
    Var,
}

#[derive(Debug)]
struct Record {
    name: String,
    typename: String,
    kind: SymbolKind,
    index: u32,
}

#[derive(Debug)]
pub struct SymbolTable {
    table: Vec<Record>,
    count: HashMap<SymbolKind, u32>,
}

impl SymbolTable {
    pub fn new() -> Self {
        SymbolTable {
            table: vec![],
            count: HashMap::default(),
        }
    }

    pub fn reset(&mut self) -> Result<(), JAError> {
        self.table.clear();
        self.count.clear();
        Ok(())
    }

    pub fn define(&mut self, name: &str, typename: &str, kind: SymbolKind) -> Result<(), JAError> {
        let counter = self.count.entry(kind.clone()).or_default();
        let index = *counter;

        *counter += 1;

        self.table.push(Record {
            name: name.to_string(),
            typename: typename.to_string(),
            kind,
            index,
        });

        Ok(())
    }

    pub fn var_count(&self, kind: &SymbolKind) -> u32 {
        *self.count.get(kind).unwrap_or(&0)
    }

    pub fn kind_of(&self, name: &str) -> Option<&SymbolKind> {
        self.table.iter().find(|r| r.name == name).map(|r| &r.kind)
    }

    pub fn type_of(&self, name: &str) -> Option<&String> {
        self.table
            .iter()
            .find(|r| r.name == name)
            .map(|r| &r.typename)
    }

    pub fn index_of(&self, name: &str) -> Option<u32> {
        self.table.iter().find(|r| r.name == name).map(|r| r.index)
    }
}

#[cfg(test)]
mod symbol_table_test {

    use rstest::fixture;
    use rstest::rstest;

    use super::*;

    type Input<'a> = Vec<(&'a str, &'a str, SymbolKind)>;

    #[fixture]
    fn make_records() -> Input<'static> {
        vec![
            ("a1", "A", SymbolKind::Var),
            ("a2", "A", SymbolKind::Var),
            ("static_a", "A", SymbolKind::Static),
            ("arg_a", "A", SymbolKind::Arg),
        ]
    }

    #[rstest]
    fn test_define_ok(make_records: Input) -> Result<(), Box<dyn std::error::Error>> {
        let mut st = SymbolTable::new();

        for (n, t, k) in make_records {
            st.define(n, t, k).unwrap();
        }

        assert_eq!(st.table.len(), 4);

        Ok(())
    }
    #[rstest]
    fn test_reset_ok(make_records: Input) -> Result<(), Box<dyn std::error::Error>> {
        let mut st = SymbolTable::new();

        for (n, t, k) in make_records {
            st.define(n, t, k).unwrap();
        }

        st.reset().unwrap();

        assert_eq!(st.table.len(), 0);
        assert_eq!(st.count.len(), 0);

        Ok(())
    }

    #[rstest]
    fn test_kind_of_ok(make_records: Input) -> Result<(), Box<dyn std::error::Error>> {
        let mut st = SymbolTable::new();

        for (n, t, k) in &make_records {
            st.define(n, t, k.clone()).unwrap();
        }

        for (n, _, k) in &make_records {
            assert_eq!(st.kind_of(n), Some(k));
        }

        Ok(())
    }

    #[rstest]
    fn test_type_of_ok(make_records: Input) -> Result<(), Box<dyn std::error::Error>> {
        let mut st = SymbolTable::new();

        for (n, t, k) in &make_records {
            st.define(n, t, k.clone()).unwrap();
        }

        for (n, t, _) in make_records {
            assert_eq!(st.type_of(n), Some(&t.to_string()));
        }

        Ok(())
    }

    #[rstest]
    fn test_index_of_ok(make_records: Input) -> Result<(), Box<dyn std::error::Error>> {
        let mut st = SymbolTable::new();

        for (n, t, k) in &make_records {
            st.define(n, t, k.clone()).unwrap();
        }

        let expected = [0, 1, 0, 0];
        for (i, (n, _, _)) in make_records.iter().enumerate() {
            assert_eq!(st.index_of(n), Some(expected[i]));
        }

        Ok(())
    }

    #[rstest]
    fn test_var_count_ok(make_records: Input) -> Result<(), Box<dyn std::error::Error>> {
        let mut st = SymbolTable::new();

        for (n, t, k) in &make_records {
            st.define(n, t, k.clone()).unwrap();
        }

        let expected = [
            (SymbolKind::Var, 2u32),
            (SymbolKind::Static, 1),
            (SymbolKind::Arg, 1),
            (SymbolKind::Field, 0),
        ];

        for (ek, ec) in expected {
            assert_eq!(st.var_count(&ek), ec);
        }

        Ok(())
    }
}
