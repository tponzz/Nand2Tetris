use std::collections::HashMap;

use crate::JAError;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum SymbolKind {
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
struct SymbolTable {
    table: Vec<Record>,
    count: HashMap<SymbolKind, u32>,
}

impl SymbolTable {
    fn new() -> Self {
        SymbolTable {
            table: vec![],
            count: HashMap::default(),
        }
    }

    fn reset(&mut self) -> Result<(), JAError> {
        self.table.clear();
        self.count.clear();
        Ok(())
    }

    fn define(&mut self, name: &str, typename: &str, kind: SymbolKind) -> Result<(), JAError> {
        let index = self.count[&kind];
        self.count.insert(kind.clone(), index + 1);

        self.table.push(Record {
            name: name.to_string(),
            typename: typename.to_string(),
            kind,
            index,
        });

        Ok(())
    }

    fn var_count(&self, kind: &SymbolKind) -> u32 {
        if let Some(cnt) = self.count.get(kind) {
            return cnt - 1;
        }

        0
    }

    pub fn kind_of(&self, name: &str) -> Option<&SymbolKind> {
        self.table
            .iter()
            .find_map(|r| if r.name == name { Some(&r.kind) } else { None })
    }

    pub fn type_of(&self, name: &str) -> Option<&String> {
        self.table.iter().find_map(|r| {
            if r.name == name {
                Some(&r.typename)
            } else {
                None
            }
        })
    }

    pub fn index_of(&self, name: &str) -> Option<u32> {
        self.table
            .iter()
            .find_map(|r| if r.name == name { Some(r.index) } else { None })
    }
}
