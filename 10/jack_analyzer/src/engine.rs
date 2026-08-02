use std::{fmt::Display, fs::File, io::Write};

use crate::{
    CompileErrKind, JAError,
    tokenizer::{Keyword, Token, TokenType, Tokenizer},
};

pub struct CompilationEngine {
    current: Option<Token>,
    // 何をメンバに持たせるか
    // 入力ストリーム？ -> Tokenizer
    tokenizer: Tokenizer,
    // 出力ストリーム？
    xml_out: File,
}

impl CompilationEngine {
    pub fn new(path_in: &str, path_out: &str) -> Result<Self, JAError> {
        let tokenizer = Tokenizer::new(path_in).map_err(|_| JAError::Io)?;
        let xml_out = File::create(path_out).map_err(|_| JAError::Io)?;

        let mut engine = Self {
            tokenizer,
            xml_out,
            current: None,
        };

        engine.advance()?;

        Ok(engine)
    }

    fn advance(&mut self) -> Result<(), JAError> {
        self.current = if self.tokenizer.has_more_tokens() {
            self.tokenizer.advance().map_err(|_| JAError::Io)?;
            Token::from(&self.tokenizer)
        } else {
            None
        };
        Ok(())
    }

    // 0 .. *
    fn accept(&mut self, pred: impl FnOnce(&Token) -> bool) -> Result<bool, JAError> {
        if !self.current.as_ref().is_some_and(pred) {
            return Ok(false);
        }
        let token = self.current.take().expect("peeked"); //peeked?

        let written = match token {
            Token::Symbol(value) => {
                // '<', '>', '&'  must be escaped considering XML output.
                let escaped = match value {
                    '<' => "&lt;".to_string(),
                    '>' => "&gt;".to_string(),
                    '&' => "&amp;".to_string(),
                    other => other.to_string(),
                };
                self.write_tag_with_value(&escaped, TokenType::Symbol)
            }
            Token::IntConst(value) => self.write_tag_with_value(&value, TokenType::IntConst),
            Token::StringConst(value) => self.write_tag_with_value(&value, TokenType::StringConst),
            Token::Keyword(value) => self.write_tag_with_value(&value, TokenType::Keyword),
            Token::Identifier(value) => self.write_tag_with_value(&value, TokenType::Identifier),
        };

        let advanced = self.advance().is_ok();

        Ok(written && advanced)
    }

    // 1 .. *
    fn expect(
        &mut self,
        pred: impl FnOnce(&Token) -> bool,
        ekind: &CompileErrKind,
    ) -> Result<(), JAError> {
        if self.accept(pred)? {
            Ok(())
        } else {
            Err(JAError::Compile(ekind.clone()))
        }
    }

    fn accept_identifier(&mut self) -> Result<bool, JAError> {
        self.accept(|tok| {
            if let Token::Identifier(typename) = tok {
                return typename.starts_with(|c: char| c == '_' || c.is_ascii_alphabetic())
                    && typename.chars().all(|c: char| c == '_' || c.is_digit(36));
            }

            false
        })
    }

    fn expect_identifier(&mut self, ekind: &CompileErrKind) -> Result<(), JAError> {
        self.expect(
            |tok| {
                if let Token::Identifier(typename) = tok {
                    return typename.starts_with(|c: char| c == '_' || c.is_ascii_alphabetic())
                        && typename.chars().all(|c: char| c == '_' || c.is_digit(36));
                }

                false
            },
            ekind,
        )
    }

    fn expect_symbol(&mut self, symbol: char, ekind: &CompileErrKind) -> Result<(), JAError> {
        self.expect(|tok| tok == &Token::Symbol(symbol), ekind)
    }

    fn accept_symbol(&mut self, symbol: char) -> Result<bool, JAError> {
        self.accept(|tok| tok == &Token::Symbol(symbol))
    }

    fn accept_symbols(&mut self, symbols: &[char]) -> Result<bool, JAError> {
        self.accept(|tok| symbols.iter().any(|&symbol| tok == &Token::Symbol(symbol)))
    }

    fn expect_keyword(&mut self, keyword: Keyword, ekind: &CompileErrKind) -> Result<(), JAError> {
        self.expect(|tok| tok == &Token::Keyword(keyword), ekind)
    }

    fn accept_keyword(&mut self, keyword: Keyword) -> Result<bool, JAError> {
        self.accept(|tok| tok == &Token::Keyword(keyword))
    }

    fn peek_keyword(&self, keyword: Keyword) -> bool {
        self.current == Some(Token::Keyword(keyword))
    }

    fn accept_type(&mut self) -> Result<bool, JAError> {
        Ok(self.accept_identifier()?
            || self.accept(|tok| match tok {
                Token::Keyword(keyword) => {
                    [Keyword::Boolean, Keyword::Int, Keyword::Char].contains(keyword)
                }
                _ => false,
            })?)
    }

    fn expect_type(&mut self, ekind: &CompileErrKind) -> Result<(), JAError> {
        if self.accept_type()? {
            Ok(())
        } else {
            Err(JAError::Compile(ekind.clone()))
        }
    }

    fn accept_int_const(&mut self) -> Result<bool, JAError> {
        self.accept(|tok| matches!(tok, Token::IntConst(_)))
    }

    fn accept_string_const(&mut self) -> Result<bool, JAError> {
        self.accept(|tok| matches!(tok, Token::StringConst(_)))
    }

    fn accept_keyword_const(&mut self) -> Result<bool, JAError> {
        self.accept(|tok| match tok {
            Token::Keyword(k) => {
                [Keyword::True, Keyword::False, Keyword::Null, Keyword::This].contains(k)
            }
            _ => false,
        })
    }

    fn write_tag_with_value<T: Display + ?Sized>(&mut self, value: &T, tag: TokenType) -> bool {
        writeln!(self.xml_out, "<{tag}> {value} </{tag}>").is_ok()
    }

    fn write_tag(&mut self, tag: &str) {
        writeln!(self.xml_out, "<{tag}>").expect("Failed to write a tag")
    }

    pub fn compile_class(&mut self) -> Result<(), JAError> {
        // Class tag
        self.write_tag("class");

        // Keyword tag for class
        self.expect_keyword(Keyword::Class, &CompileErrKind::Class)?;

        // Class identifier
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

        self.write_tag("/class");

        Ok(())
    }

    pub fn compile_subroutine(&mut self) -> Result<(), JAError> {
        // Sub-routine tag
        self.write_tag("subroutineDec");

        // function keyword tag
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
        if !self.accept_keyword(Keyword::Void)? && !self.accept_identifier()? {
            return Err(JAError::Compile(CompileErrKind::Subroutine));
        }

        // identifier tag
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

        self.write_tag("/subroutineDec");

        Ok(())
    }

    // parameterList: ((type varName) (',' type varName)*)?
    // The surrounding '(' ')' belong to the caller (compile_subroutine), not here.
    pub fn compile_parameter_list(&mut self) -> Result<(), JAError> {
        self.write_tag("parameterList");

        while self.accept_type()? {
            self.expect_identifier(&CompileErrKind::ParameterList)?;
            if !self.accept_symbol(',')? {
                break;
            }
        }

        self.write_tag("/parameterList");

        Ok(())
    }

    pub fn compile_subroutine_body(&mut self) -> Result<(), JAError> {
        self.write_tag("subroutineBody");

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

        self.write_tag("/subroutineBody");

        Ok(())
    }

    // type varName (',' varName)* ';' -- shared by varDec and classVarDec,
    // which differ only in their leading keyword.
    fn compile_var_dec_tail(&mut self) -> Result<(), JAError> {
        // type : int|char|boolean|className(identifier)
        self.expect_type(&CompileErrKind::VarDec)?;

        // varName(identifier)
        self.expect_identifier(&CompileErrKind::VarDec)?;

        // if symbol ',', other varName(identifier)
        while !self.accept_symbol(';')? {
            if self.accept_identifier()? {
                continue;
            }

            self.expect_symbol(',', &CompileErrKind::VarDec)?;
        }

        Ok(())
    }

    // varDec: 'var' type varName (',' varName)* ';'
    pub fn compile_var_dec(&mut self) -> Result<(), JAError> {
        self.write_tag("varDec");
        self.expect_keyword(Keyword::Var, &CompileErrKind::VarDec)?;
        self.compile_var_dec_tail()?;
        self.write_tag("/varDec");
        Ok(())
    }

    // classVarDec: ('static'|'field') type varName (',' varName)* ';'
    pub fn compile_class_var_dec(&mut self) -> Result<(), JAError> {
        self.write_tag("classVarDec");
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
        self.write_tag("/classVarDec");
        Ok(())
    }

    pub fn compile_statements(&mut self) -> Result<(), JAError> {
        self.write_tag("statements");

        while self.compile_let().is_ok()
            || self.compile_if().is_ok()
            || self.compile_while().is_ok()
            || self.compile_do().is_ok()
            || self.compile_return().is_ok()
        {}

        self.write_tag("/statements");

        Ok(())
    }

    pub fn compile_let(&mut self) -> Result<(), JAError> {
        if !self.peek_keyword(Keyword::Let) {
            return Err(JAError::Compile(CompileErrKind::Let));
        }
        let ekind = &CompileErrKind::Let;
        self.write_tag("letStatement");
        // let
        self.expect_keyword(Keyword::Let, ekind)?;

        // varName
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

        self.write_tag("/letStatement");

        Ok(())
    }

    pub fn compile_if(&mut self) -> Result<(), JAError> {
        if !self.peek_keyword(Keyword::If) {
            return Err(JAError::Compile(CompileErrKind::If));
        }
        let ekind = &CompileErrKind::If;
        self.write_tag("ifStatement");
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

        self.write_tag("/ifStatement");

        Ok(())
    }

    pub fn compile_while(&mut self) -> Result<(), JAError> {
        if !self.peek_keyword(Keyword::While) {
            return Err(JAError::Compile(CompileErrKind::While));
        }
        let ekind = &CompileErrKind::While;
        self.write_tag("whileStatement");

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

        self.write_tag("/whileStatement");

        Ok(())
    }

    pub fn compile_do(&mut self) -> Result<(), JAError> {
        if !self.peek_keyword(Keyword::Do) {
            return Err(JAError::Compile(CompileErrKind::Do));
        }
        let ekind = &CompileErrKind::Do;
        self.write_tag("doStatement");

        self.expect_keyword(Keyword::Do, ekind)?;

        // subroutineCall
        if self.accept_identifier()? {
            if self.accept_symbol('(')? {
                self.compile_expression_list()?;
                self.expect_symbol(')', ekind)?;
            } else if self.accept_symbol('.')? {
                self.expect_identifier(ekind)?;
                self.expect_symbol('(', ekind)?;
                self.compile_expression_list()?;
                self.expect_symbol(')', ekind)?;
            }
        }

        // ;
        self.expect_symbol(';', ekind)?;

        self.write_tag("/doStatement");

        Ok(())
    }

    pub fn compile_return(&mut self) -> Result<(), JAError> {
        if !self.peek_keyword(Keyword::Return) {
            return Err(JAError::Compile(CompileErrKind::Return));
        }
        let ekind = &CompileErrKind::Return;
        self.write_tag("returnStatement");
        // return exp?
        self.expect_keyword(Keyword::Return, ekind)?;
        if !self.accept_symbol(';')? {
            self.compile_expression()?;
            self.expect_symbol(';', ekind)?;
        }
        self.write_tag("/returnStatement");
        Ok(())
    }

    pub fn compile_expression(&mut self) -> Result<(), JAError> {
        self.write_tag("expression");

        self.compile_term()?;

        while self.accept_symbols(&['+', '-', '*', '/', '&', '|', '<', '>', '='])? {
            self.compile_term()?;
        }

        self.write_tag("/expression");

        Ok(())
    }

    pub fn compile_term(&mut self) -> Result<(), JAError> {
        let ekind = &CompileErrKind::Term;
        self.write_tag("term");

        // unaryOp term -- the operand is itself a nested term
        if self.accept_symbol('-')? || self.accept_symbol('~')? {
            self.compile_term()?;
            self.write_tag("/term");
            return Ok(());
        }

        // varName/subroutineCall
        if self.accept_identifier()? {
            // array
            if self.accept_symbol('[')? {
                self.compile_expression()?;
                self.expect_symbol(']', ekind)?;
            //
            } else if self.accept_symbol('(')? {
                self.compile_expression_list()?;
                self.expect_symbol(')', ekind)?;
            } else if self.accept_symbol('.')? {
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
            return Err(JAError::Compile(ekind.clone()));
        }

        // if it comes op, another term will appear
        self.write_tag("/term");
        Ok(())
    }

    // expressionList: (expression (',' expression)*)?
    pub fn compile_expression_list(&mut self) -> Result<(), JAError> {
        self.write_tag("expressionList");

        if !matches!(self.current, Some(Token::Symbol(')'))) {
            // exp
            self.compile_expression()?;

            // if appear ',', expect additional exp
            while self.accept_symbol(',')? {
                self.compile_expression()?;
            }
        }

        self.write_tag("/expressionList");

        Ok(())
    }
}
