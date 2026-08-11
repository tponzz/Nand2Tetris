use core::fmt;
use std::fs::File;

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
            self.index.map_or("".to_string(), |i| i.to_string()),
            self.usage
        )
    }
}

pub struct CompilationEngine {
    current: Option<Token>,
    // 何をメンバに持たせるか
    // 入力ストリーム？ -> Tokenizer
    tokenizer: Tokenizer,
    // 出力ストリーム？
    xml_out: File,
    writer: VmWriter,
    st: SymbolTable,
}

impl CompilationEngine {
    pub fn new(path_in: &str, path_out: &str) -> Result<Self, JAError> {
        let tokenizer = Tokenizer::new(path_in)
            .map_err(|_| JAError::Io("Failed to read source".to_string()))?;
        let xml_out =
            File::create(path_out).map_err(|_| JAError::Io("Failed to open .xml".to_string()))?;

        let writer = VmWriter::new(path_out);
        let st = SymbolTable::new();

        let mut engine = Self {
            tokenizer,
            xml_out,
            current: None,
            writer,
            st,
        };

        engine.advance()?;

        Ok(engine)
    }

    fn advance(&mut self) -> Result<(), JAError> {
        self.current = if self.tokenizer.has_more_tokens() {
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
        if !self.current.as_ref().is_some_and(pred) {
            return Ok(None);
        }

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

    fn accept_identifier(&mut self) -> Result<Option<&String>, JAError> {
        let tok = self.accept(|tok| {
            if let Token::Identifier(typename) = tok {
                return typename.starts_with(|c: char| c == '_' || c.is_ascii_alphabetic())
                    && typename.chars().all(|c: char| c == '_' || c.is_digit(36));
            }

            false
        })?;

        if let Some(Token::Identifier(id)) = tok {
            Ok(Some(id))
        } else {
            Ok(None)
        }
    }

    fn expect_identifier(&mut self, ekind: &CompileErrKind) -> Result<&String, JAError> {
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
            Ok(id)
        } else {
            Err(JAError::Compile(*ekind))
        }
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

    fn peek_keyword(&self, keyword: Keyword) -> bool {
        self.current == Some(Token::Keyword(keyword))
    }

    fn accept_type(&mut self) -> Result<Option<String>, JAError> {
        todo!("identifier detail");
        if let Some(t) = self.accept_identifier()? {
            Ok(Some(t.clone()))
        } else if let Some(Token::Identifier(t)) = self.accept(|tok| match tok {
            Token::Keyword(keyword) => {
                [Keyword::Boolean, Keyword::Int, Keyword::Char].contains(keyword)
            }
            _ => false,
        })? {
            Ok(Some(t.clone()))
        } else {
            Ok(None)
        }
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
                [Keyword::True, Keyword::False, Keyword::Null, Keyword::This].contains(k)
            }
            _ => false,
        })
        .map(|tok| tok.is_some())
    }

    pub fn compile_class(&mut self) -> Result<(), JAError> {
        // Keyword tag for class
        self.expect_keyword(Keyword::Class, &CompileErrKind::Class)?;

        // Class identifier
        todo!("identifier detail");
        self.expect_identifier(&CompileErrKind::Class)?;

        // { token
        self.expect_symbol('{', &CompileErrKind::Class)?;

        // class var declarations (0 or more)
        while matches!(
            self.current,
            Some(Token::Keyword(Keyword::Static)) | Some(Token::Keyword(Keyword::Field))
        ) {
            self.compile_class_var_dec()?;
        }

        // class method declarations (0 or more)
        while matches!(
            self.current,
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
        // function keyword
        self.expect(
            |tok| {
                [
                    Token::Keyword(Keyword::Constructor),
                    Token::Keyword(Keyword::Function),
                    Token::Keyword(Keyword::Method),
                ]
                .contains(tok)
            },
            &CompileErrKind::Subroutine,
        )?;

        // type keyword tag
        // TODO: 関数名は要取得
        // identifierのissomeは一時的
        if !self.accept_keyword(Keyword::Void)? && self.accept_identifier()?.is_none() {
            return Err(JAError::Compile(CompileErrKind::Subroutine));
        }

        // identifier tag
        todo!("identifier detail");
        self.expect_identifier(&CompileErrKind::Subroutine)?;

        // Parameters
        // ( token
        self.expect_symbol('(', &CompileErrKind::Subroutine)?;

        // parameter list
        self.compile_parameter_list()?;

        // ) token
        self.expect_symbol(')', &CompileErrKind::Subroutine)?;

        // class method declarations
        // compile_subroutine_body already consumes the body's closing '}'
        self.compile_subroutine_body()?;

        Ok(())
    }

    // parameterList: ((type varName) (',' type varName)*)?
    // The surrounding '(' ')' belong to the caller (compile_subroutine), not here.
    pub fn compile_parameter_list(&mut self) -> Result<(), JAError> {
        while let Some(t) = self.accept_type()? {
            let param = self.expect_identifier(&CompileErrKind::ParameterList)?;

            // TODO:
            // lookup parameters from symbol_table
            // Name: param, Category: Arg, Usage: Used, index: get from symbol_table
            todo!("identifier detail");

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
        while matches!(self.current, Some(Token::Keyword(Keyword::Var))) {
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
    fn compile_var_dec_tail(&mut self) -> Result<(), JAError> {
        // type : int|char|boolean|className(identifier)
        let typename = self.expect_type(&CompileErrKind::VarDec)?;

        // if symbol ',', other varName(identifier)
        while !self.accept_symbol(';')? {
            todo!("identifier detail");

            let tok = self.accept_identifier()?;
            if let Some(id) = tok {
                // シンボルテーブルへ登録
                let id = id.to_string();
                self.st.define(&id, &typename, SymbolKind::Var)?;
                continue;
            }

            self.expect_symbol(',', &CompileErrKind::VarDec)?;
        }

        Ok(())
    }

    // varDec: 'var' type varName (',' varName)* ';'
    pub fn compile_var_dec(&mut self) -> Result<(), JAError> {
        self.expect_keyword(Keyword::Var, &CompileErrKind::VarDec)?;
        self.compile_var_dec_tail()?;
        Ok(())
    }

    // classVarDec: ('static'|'field') type varName (',' varName)* ';'
    pub fn compile_class_var_dec(&mut self) -> Result<(), JAError> {
        self.expect(
            |tok| {
                [
                    Token::Keyword(Keyword::Static),
                    Token::Keyword(Keyword::Field),
                ]
                .contains(tok)
            },
            &CompileErrKind::VarDec,
        )?;
        self.compile_var_dec_tail()?;
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
        if !self.peek_keyword(Keyword::Let) {
            return Err(JAError::Compile(CompileErrKind::Let));
        }
        let ekind = &CompileErrKind::Let;
        // let
        self.expect_keyword(Keyword::Let, ekind)?;

        // varName
        todo!("identifier detail");
        self.expect_identifier(ekind)?;

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
        if !self.peek_keyword(Keyword::If) {
            return Err(JAError::Compile(CompileErrKind::If));
        }
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
        if !self.peek_keyword(Keyword::While) {
            return Err(JAError::Compile(CompileErrKind::While));
        }
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
        if !self.peek_keyword(Keyword::Do) {
            return Err(JAError::Compile(CompileErrKind::Do));
        }
        let ekind = &CompileErrKind::Do;

        self.expect_keyword(Keyword::Do, ekind)?;

        // subroutineCall
        // TODO:
        // シンボルテーブルで見つからなければfunctionコール
        // 見つかればオブジェクトのmethodコール
        todo!("identifier detail");
        let id = self.expect_identifier(&CompileErrKind::Do)?.to_string();
        let index = self.st.index_of(&id);
        if let Some(index) = index {
            // TODO: detail出力
            self.expect_symbol('(', &CompileErrKind::Do)?;
            self.compile_expression_list()?;
            self.expect_symbol(')', ekind)?;
        } else {
            self.expect_symbol('.', &CompileErrKind::Do)?;
            todo!("identifier detail");
            self.expect_identifier(ekind)?;
            self.expect_symbol('(', ekind)?;
            self.compile_expression_list()?;
            self.expect_symbol(')', ekind)?;
        }

        // ;
        self.expect_symbol(';', ekind)?;

        Ok(())
    }

    pub fn compile_return(&mut self) -> Result<(), JAError> {
        if !self.peek_keyword(Keyword::Return) {
            return Err(JAError::Compile(CompileErrKind::Return));
        }
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
            return Ok(());
        }

        // varName/subroutineCall
        // シンボルテーブルで見つからなければsubroutine
        // 見つかればオブジェクト/変数/配列のmethodコール
        todo!("identifier detail");
        todo!("lookup with symbol_table");
        if let Some(id) = self.accept_identifier()? {
            // array
            if self.accept_symbol('[')? {
                self.compile_expression()?;
                self.expect_symbol(']', ekind)?;
            } else if self.accept_symbol('(')? {
                self.compile_expression_list()?;
                self.expect_symbol(')', ekind)?;
            } else if self.accept_symbol('.')? {
                todo!("identifier detail");
                self.expect_identifier(ekind)?;
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
        if !matches!(self.current, Some(Token::Symbol(')'))) {
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
