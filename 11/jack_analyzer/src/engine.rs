use core::fmt;
use std::io::Write;
use std::{fs::File, io};

use crate::{
    CompileErrKind, JAError,
    symbol_table::{SymbolKind, SymbolTable},
    tokenizer::{Keyword, Token, Tokenizer},
    vm_writer::VmWriter,
};

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum Category {
    Symbol(SymbolKind),
    Class,
    SubRoutine,
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Symbol(SymbolKind::Field) => write!(f, "field"),
            Category::Symbol(SymbolKind::Static) => write!(f, "static"),
            Category::Symbol(SymbolKind::Var) => write!(f, "var"),
            Category::Symbol(SymbolKind::Arg) => write!(f, "arg"),
            Category::Class => write!(f, "class"),
            Category::SubRoutine => write!(f, "subroutine"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Usage {
    Declared,
    Used,
}

impl fmt::Display for Usage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Usage::Declared => write!(f, "declared"),
            Usage::Used => write!(f, "used"),
        }
    }
}

#[derive(Debug)]
pub struct IdentifierDetail {
    name: String,
    category: Category,
    index: Option<u32>,
    usage: Usage,
}

impl fmt::Display for IdentifierDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.name,
            self.category,
            self.usage,
            self.index.map_or("".to_string(), |i| i.to_string()),
        )
    }
}

pub struct CompilationEngine {
    current: Option<Token>,
    next: Option<Token>,
    // 何をメンバに持たせるか
    // 入力ストリーム？ -> Tokenizer
    tokenizer: Tokenizer,
    // 出力ストリーム？
    xml_out: File,
    writer: VmWriter,
    class_st: SymbolTable,
    method_st: SymbolTable,
}

impl CompilationEngine {
    pub fn new(path_in: &str, path_out: &str) -> Result<Self, JAError> {
        let tokenizer = Tokenizer::new(path_in)
            .map_err(|_| JAError::Io("Failed to read source".to_string()))?;
        let xml_out =
            File::create(path_out).map_err(|_| JAError::Io("Failed to open .xml".to_string()))?;

        let writer = VmWriter::new(path_out);

        let mut engine = Self {
            tokenizer,
            xml_out,
            current: None,
            next: None,
            writer,
            class_st: SymbolTable::new(),
            method_st: SymbolTable::new(),
        };

        engine.advance()?;

        Ok(engine)
    }

    fn advance(&mut self) -> Result<(), JAError> {
        self.next = if self.tokenizer.has_more_tokens() {
            self.tokenizer
                .advance()
                .map_err(|_| JAError::Io("Failed to advance".to_string()))?;
            Token::from(&self.tokenizer)
        } else {
            None
        };
        Ok(())
    }

    // 0 .. *
    fn accept(
        &mut self,
        // name: &str,
        // category: &Category,
        // usage: &Usage,
        pred: impl FnOnce(&Token) -> bool,
    ) -> Result<Option<&Token>, JAError> {
        // 条件に合わないとスルー
        if !self.next.as_ref().is_some_and(pred) {
            return Ok(None);
        }

        self.current = self.next.take();
        self.advance()?;

        Ok(self.current.as_ref())
    }

    // 1 .. *
    // 条件に合致する必要あり
    // トークンを返す
    fn expect(
        &mut self,
        pred: impl FnOnce(&Token) -> bool,
        ekind: &CompileErrKind,
    ) -> Result<&Token, JAError> {
        if let Some(tok) = self.accept(pred)? {
            Ok(tok)
        } else {
            Err(JAError::Compile(*ekind))
        }
    }

    fn accept_identifier(&mut self) -> Result<Option<String>, JAError> {
        let tok = self.accept(|tok| {
            if let Token::Identifier(typename) = tok {
                return typename.starts_with(|c: char| c == '_' || c.is_ascii_alphabetic())
                    && typename.chars().all(|c: char| c == '_' || c.is_digit(36));
            }

            false
        })?;

        if let Some(Token::Identifier(id)) = tok {
            Ok(Some(id.clone()))
        } else {
            Ok(None)
        }
    }

    fn expect_identifier(&mut self, ekind: &CompileErrKind) -> Result<String, JAError> {
        let tok = self.expect(
            |tok| {
                if let Token::Identifier(typename) = tok {
                    return typename.starts_with(|c: char| c == '_' || c.is_ascii_alphabetic())
                        && typename.chars().all(|c: char| c == '_' || c.is_digit(36));
                }

                false
            },
            ekind,
        )?;

        if let Token::Identifier(id) = tok {
            Ok(id.clone())
        } else {
            Err(JAError::Compile(*ekind))
        }
    }

    fn write_id_details(
        &mut self,
        name: &str,
        category: Category,
        usage: Usage,
        index: Option<u32>,
    ) -> io::Result<()> {
        let detail = IdentifierDetail {
            name: name.to_string(),
            category,
            index,
            usage,
        };

        writeln!(self.xml_out, "{}", detail)
    }

    // シンボルが一致していることが期待される
    fn expect_symbol(&mut self, symbol: char, ekind: &CompileErrKind) -> Result<(), JAError> {
        self.expect(|tok| tok == &Token::Symbol(symbol), ekind)
            .map(|_| ()) // シンボルは自明なので返さなくてもいいかも
    }

    // シンボルが一致しているときのみ消費
    fn accept_symbol(&mut self, symbol: char) -> Result<bool, JAError> {
        self.accept_symbols(&[symbol])
    }

    fn accept_symbols(&mut self, symbols: &[char]) -> Result<bool, JAError> {
        self.accept(|tok| symbols.iter().any(|&symbol| tok == &Token::Symbol(symbol)))
            .map(|tok| tok.is_some())
    }

    fn expect_keyword(&mut self, keyword: Keyword, ekind: &CompileErrKind) -> Result<(), JAError> {
        self.expect(|tok| tok == &Token::Keyword(keyword), ekind)
            .map(|_| ())
    }

    fn accept_keyword(&mut self, keyword: Keyword) -> Result<bool, JAError> {
        self.accept(|tok| tok == &Token::Keyword(keyword))
            .map(|tok| tok.is_some())
    }

    // typeのreturnは必要か？
    fn accept_type(&mut self) -> Result<Option<String>, JAError> {
        if let Some(typename) = self.accept_identifier()? {
            // Category: class, Usage: Declared, index: None
            self.write_id_details(&typename, Category::Class, Usage::Declared, None)?;
            return Ok(Some(typename));
        }

        if let Some(Token::Keyword(t)) = self.accept(|tok| {
            matches!(
                tok,
                Token::Keyword(Keyword::Boolean)
                    | Token::Keyword(Keyword::Int)
                    | Token::Keyword(Keyword::Char)
            )
        })? {
            return Ok(Some(t.to_string()));
        }

        Ok(None)
    }

    fn expect_type(&mut self, ekind: &CompileErrKind) -> Result<String, JAError> {
        if let Some(t) = self.accept_type()? {
            Ok(t.clone())
        } else {
            Err(JAError::Compile(*ekind))
        }
    }

    fn accept_int_const(&mut self) -> Result<bool, JAError> {
        self.accept(|tok| matches!(tok, Token::IntConst(_)))
            .map(|tok| tok.is_some())
    }

    fn accept_string_const(&mut self) -> Result<bool, JAError> {
        self.accept(|tok| matches!(tok, Token::StringConst(_)))
            .map(|tok| tok.is_some())
    }

    fn accept_keyword_const(&mut self) -> Result<bool, JAError> {
        self.accept(|tok| match tok {
            Token::Keyword(k) => {
                matches!(
                    k,
                    Keyword::True | Keyword::False | Keyword::Null | Keyword::This
                )
            }
            _ => false,
        })
        .map(|tok| tok.is_some())
    }

    pub fn compile_class(&mut self) -> Result<(), JAError> {
        // Keyword tag for class
        self.expect_keyword(Keyword::Class, &CompileErrKind::Class)?;

        // Class identifier
        let class = self.expect_identifier(&CompileErrKind::Class)?;
        self.write_id_details(&class, Category::Class, Usage::Declared, None)?;

        // { token
        self.expect_symbol('{', &CompileErrKind::Class)?;

        // class var declarations (0 or more)
        while matches!(
            self.next,
            Some(Token::Keyword(Keyword::Static)) | Some(Token::Keyword(Keyword::Field))
        ) {
            self.compile_class_var_dec()?;
        }

        // class method declarations (0 or more)
        while matches!(
            self.next,
            Some(Token::Keyword(Keyword::Constructor))
                | Some(Token::Keyword(Keyword::Function))
                | Some(Token::Keyword(Keyword::Method))
        ) {
            self.compile_subroutine()?;
        }

        // }
        self.expect_symbol('}', &CompileErrKind::Class)?;

        Ok(())
    }

    pub fn compile_subroutine(&mut self) -> Result<(), JAError> {
        self.method_st.reset()?;

        // function keyword
        self.expect(
            |tok| {
                matches!(
                    tok,
                    Token::Keyword(Keyword::Constructor)
                        | Token::Keyword(Keyword::Function)
                        | Token::Keyword(Keyword::Method)
                )
            },
            &CompileErrKind::Subroutine,
        )?;

        // type keyword tag
        if self.accept_keyword(Keyword::Void)? {
        } else if let Some(typename) = self.accept_type()? {
            self.write_id_details(&typename, Category::Class, Usage::Declared, None)?;
        } else {
            return Err(JAError::Compile(CompileErrKind::Subroutine));
        }

        // identifier tag
        let subroutine = self.expect_identifier(&CompileErrKind::Subroutine)?;
        self.write_id_details(&subroutine, Category::SubRoutine, Usage::Declared, None)?;

        // Parameters
        // ( token
        self.expect_symbol('(', &CompileErrKind::Subroutine)?;

        // parameter list
        self.compile_parameter_list()?;

        // ) token
        self.expect_symbol(')', &CompileErrKind::Subroutine)?;

        // class method declarations
        // compile_subroutine_body already consumes the body's '{}'
        self.compile_subroutine_body()?;

        Ok(())
    }

    // parameterList: ((type varName) (',' type varName)*)?
    // The surrounding '(' ')' belong to the caller (compile_subroutine), not here.
    pub fn compile_parameter_list(&mut self) -> Result<(), JAError> {
        while let Some(typename) = self.accept_type()? {
            let name = self.expect_identifier(&CompileErrKind::ParameterList)?;

            // Add params to symbol_table as args
            self.method_st.define(&name, &typename, SymbolKind::Arg)?;

            // Name: param, Category: Arg, Usage: Declared, index: None
            self.write_id_details(
                &name,
                Category::Symbol(SymbolKind::Arg),
                Usage::Declared,
                None,
            )?;

            if !self.accept_symbol(',')? {
                break;
            }
        }

        Ok(())
    }

    pub fn compile_subroutine_body(&mut self) -> Result<(), JAError> {
        // {
        self.expect_symbol('{', &CompileErrKind::SubroutineBody)?;

        // var dec
        // 0..*
        while matches!(self.next, Some(Token::Keyword(Keyword::Var))) {
            self.compile_var_dec()?;
        }

        // statements
        // 1
        self.compile_statements()?;

        // }
        self.expect_symbol('}', &CompileErrKind::SubroutineBody)?;

        Ok(())
    }

    // type varName (',' varName)* ';' -- shared by varDec and classVarDec,
    // which differ only in their leading keyword.
    // class -> static, field
    // method -> var
    fn compile_var_dec_tail(&mut self, skind: SymbolKind) -> Result<(), JAError> {
        // type : int|char|boolean|className(identifier)
        let typename = self.expect_type(&CompileErrKind::VarDec)?;

        // if symbol ',', other varName(identifier)
        while !self.accept_symbol(';')? {
            if let Some(id) = self.accept_identifier()? {
                // シンボルテーブルへ登録
                let name = id.to_string();
                match skind {
                    SymbolKind::Static => {
                        self.write_id_details(
                            &name,
                            Category::Symbol(SymbolKind::Static),
                            Usage::Declared,
                            None,
                        )?;
                        self.class_st.define(&name, &typename, skind)?;
                    }
                    SymbolKind::Field => {
                        self.write_id_details(
                            &name,
                            Category::Symbol(SymbolKind::Field),
                            Usage::Declared,
                            None,
                        )?;

                        self.class_st.define(&name, &typename, skind)?;
                    }
                    SymbolKind::Var => {
                        self.write_id_details(
                            &name,
                            Category::Symbol(SymbolKind::Var),
                            Usage::Declared,
                            None,
                        )?;

                        self.method_st.define(&name, &typename, skind)?;
                    }
                    _ => return Err(JAError::Compile(CompileErrKind::VarDec)),
                }
                continue;
            }

            self.expect_symbol(',', &CompileErrKind::VarDec)?;
        }

        Ok(())
    }

    // varDec: 'var' type varName (',' varName)* ';'
    pub fn compile_var_dec(&mut self) -> Result<(), JAError> {
        self.expect_keyword(Keyword::Var, &CompileErrKind::VarDec)?;
        self.compile_var_dec_tail(SymbolKind::Var)?;
        Ok(())
    }

    // classVarDec: ('static'|'field') type varName (',' varName)* ';'
    pub fn compile_class_var_dec(&mut self) -> Result<(), JAError> {
        let tok = self.expect(
            |tok| {
                matches!(
                    tok,
                    Token::Keyword(Keyword::Static) | Token::Keyword(Keyword::Field)
                )
            },
            &CompileErrKind::VarDec,
        )?;

        if let Token::Keyword(keyword) = tok {
            match keyword {
                Keyword::Static => self.compile_var_dec_tail(SymbolKind::Static)?,
                Keyword::Field => self.compile_var_dec_tail(SymbolKind::Field)?,
                _ => return Err(JAError::Compile(CompileErrKind::VarDec)),
            };
        }

        Ok(())
    }

    pub fn compile_statements(&mut self) -> Result<(), JAError> {
        while self.compile_let().is_ok()
            || self.compile_if().is_ok()
            || self.compile_while().is_ok()
            || self.compile_do().is_ok()
            || self.compile_return().is_ok()
        {}

        Ok(())
    }

    pub fn compile_let(&mut self) -> Result<(), JAError> {
        let ekind = &CompileErrKind::Let;

        // let
        self.expect_keyword(Keyword::Let, ekind)?;

        // varName
        let var = self.expect_identifier(ekind)?;
        let index = self.method_st.index_of(&var);
        let kind = self
            .method_st
            .kind_of(&var)
            .expect("Variable not on symbol table");

        self.write_id_details(&var, Category::Symbol(*kind), Usage::Used, index)?;

        // optional `[ exp ]` for array assignment
        if self.accept_symbol('[')? {
            self.compile_expression()?;
            self.expect_symbol(']', ekind)?;
        }

        // '=' always follows, whether or not '[ exp ]' was present
        self.expect_symbol('=', ekind)?;

        // exp
        self.compile_expression()?;

        // ;
        self.expect_symbol(';', ekind)?;

        Ok(())
    }

    pub fn compile_if(&mut self) -> Result<(), JAError> {
        let ekind = &CompileErrKind::If;
        // if
        self.expect_keyword(Keyword::If, ekind)?;

        // ( exp )
        self.expect_symbol('(', ekind)?;
        self.compile_expression()?;
        self.expect_symbol(')', ekind)?;

        // { statement }
        self.expect_symbol('{', ekind)?;
        self.compile_statements()?;
        self.expect_symbol('}', ekind)?;

        // if else, appear another {}
        if self.accept_keyword(Keyword::Else)? {
            self.expect_symbol('{', ekind)?;
            self.compile_statements()?;
            self.expect_symbol('}', ekind)?;
        }

        Ok(())
    }

    pub fn compile_while(&mut self) -> Result<(), JAError> {
        let ekind = &CompileErrKind::While;

        // while
        self.expect_keyword(Keyword::While, ekind)?;

        // ( exp )
        self.expect_symbol('(', ekind)?;
        self.compile_expression()?;
        self.expect_symbol(')', ekind)?;

        // { statement }
        self.expect_symbol('{', ekind)?;
        self.compile_statements()?;
        self.expect_symbol('}', ekind)?;

        Ok(())
    }

    pub fn compile_do(&mut self) -> Result<(), JAError> {
        let ekind = &CompileErrKind::Do;

        // do
        self.expect_keyword(Keyword::Do, ekind)?;

        // subroutineCall
        // Type/arg.method()
        // method()
        let object = self.expect_identifier(&CompileErrKind::Do)?.to_string();
        if self.accept_symbol('.')? {
            //Type/arg
            //methodシンボルテーブルで見つかるか
            //見つかる->arg
            //見つからない->Type
            let idx = self.method_st.index_of(&object);
            let kind = self.method_st.kind_of(&object);
            self.write_id_details(
                &object,
                if let Some(kind) = kind {
                    Category::Symbol(*kind) // arg
                } else {
                    Category::Class // Type
                },
                Usage::Used,
                idx,
            )?;
        }

        let method = self.expect_identifier(ekind)?;
        self.write_id_details(&method, Category::SubRoutine, Usage::Used, None)?;

        // ( exp_list )
        self.expect_symbol('(', ekind)?;
        self.compile_expression_list()?;
        self.expect_symbol(')', ekind)?;

        // ;
        self.expect_symbol(';', ekind)?;

        Ok(())
    }

    pub fn compile_return(&mut self) -> Result<(), JAError> {
        let ekind = &CompileErrKind::Return;

        // return exp?
        self.expect_keyword(Keyword::Return, ekind)?;
        if !self.accept_symbol(';')? {
            self.compile_expression()?;
            self.expect_symbol(';', ekind)?;
        }

        Ok(())
    }

    pub fn compile_expression(&mut self) -> Result<(), JAError> {
        self.compile_term()?;

        while self.accept_symbols(&['+', '-', '*', '/', '&', '|', '<', '>', '='])? {
            self.compile_term()?;
        }

        Ok(())
    }

    pub fn compile_term(&mut self) -> Result<(), JAError> {
        let ekind = &CompileErrKind::Term;

        // unaryOp term -- the operand is itself a nested term
        if self.accept_symbol('-')? || self.accept_symbol('~')? {
            self.compile_term()?;
        } else
        // varName/subroutineCall
        // シンボルテーブルで見つからなければsubroutine
        // 見つかればオブジェクト/変数/配列のmethodコール
        // 先読みが必須？ acceptで先読みできる？
        if let Some(name) = self.accept_identifier()? {
            let class_idx = self.class_st.index_of(&name);
            let method_idx = self.method_st.index_of(&name);
            match (class_idx, method_idx) {
                (Some(index), None) => {
                    let kind = self
                        .class_st
                        .kind_of(&name)
                        .ok_or(JAError::Compile(*ekind))?;
                    self.write_id_details(
                        &name,
                        Category::Symbol(*kind),
                        Usage::Used,
                        Some(index),
                    )?;
                }
                (None, Some(index)) => {
                    let kind = self
                        .method_st
                        .kind_of(&name)
                        .ok_or(JAError::Compile(*ekind))?;

                    // method arg
                    // method local var
                    self.write_id_details(
                        &name,
                        Category::Symbol(*kind),
                        Usage::Used,
                        Some(index),
                    )?;
                }
                (_, _) => {
                    // undefined or OS native?
                    // return Err(JAError::Compile(CompileErrKind::Term));
                    self.write_id_details(&name, Category::Class, Usage::Used, None)?;
                }
            }

            // array
            if self.accept_symbol('[')? {
                self.compile_expression()?;
                self.expect_symbol(']', ekind)?;
            } else
            // subroutine call
            if self.accept_symbol('(')? {
                self.write_id_details(&name, Category::SubRoutine, Usage::Used, None)?;
                self.compile_expression_list()?;
                self.expect_symbol(')', ekind)?;
            } else
            // method call
            if self.accept_symbol('.')? {
                let method = self.expect_identifier(ekind)?;
                self.write_id_details(&method, Category::SubRoutine, Usage::Used, None)?;
                self.expect_symbol('(', ekind)?;
                self.compile_expression_list()?;
                self.expect_symbol(')', ekind)?;
            }
        }
        // (exp)
        else if self.accept_symbol('(')? && !self.accept_symbol(')')? {
            self.compile_expression()?;
            self.expect_symbol(')', ekind)?;
        }
        // const
        else if !(self.accept_int_const()?
            || self.accept_string_const()?
            || self.accept_keyword_const()?)
        {
            return Err(JAError::Compile(*ekind));
        }

        Ok(())
    }

    // expressionList: (expression (',' expression)*)?
    pub fn compile_expression_list(&mut self) -> Result<(), JAError> {
        if !matches!(self.next, Some(Token::Symbol(')'))) {
            // exp
            self.compile_expression()?;

            // if appear ',', expect additional exp
            while self.accept_symbol(',')? {
                self.compile_expression()?;
            }
        }

        Ok(())
    }
}
