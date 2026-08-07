use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{self, BufRead, BufReader},
    ops::{self},
    sync::OnceLock,
};

#[derive(Debug, PartialEq)]
pub enum Token {
    Symbol(char),
    IntConst(i16),
    StringConst(String),
    Keyword(Keyword),
    Identifier(String),
}

impl Token {
    pub fn from(tokenizer: &Tokenizer) -> Option<Self> {
        match tokenizer.token_type() {
            Some(t) => match t {
                TokenType::Keyword => {
                    let keyword = tokenizer.keyword()?.clone();
                    Some(Token::Keyword(keyword))
                }
                TokenType::Symbol => {
                    let symbol = tokenizer.symbol()?;
                    Some(Token::Symbol(symbol))
                }
                TokenType::Identifier => {
                    let id = tokenizer.identifier()?;
                    Some(Token::Identifier(id.to_string()))
                }
                TokenType::IntConst => {
                    let i = tokenizer.int_val().map(|n| n.parse::<i16>().ok())?;
                    Some(Token::IntConst(i?))
                }
                TokenType::StringConst => {
                    let s = tokenizer.string_val().map(|s| s.to_string())?;
                    Some(Token::StringConst(s))
                }
            },
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenType {
    Keyword,
    Symbol,
    Identifier,
    IntConst,
    StringConst,
}

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Keyword => write!(f, "keyword"),
            TokenType::Symbol => write!(f, "symbol"),
            TokenType::Identifier => write!(f, "identifier"),
            TokenType::IntConst => write!(f, "integerConstant"),
            TokenType::StringConst => write!(f, "stringConstant"),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    Class,
    Method,
    Function,
    Constructor,
    Int,
    Boolean,
    Char,
    Void,
    Var,
    Static,
    Field,
    Let,
    Do,
    If,
    Else,
    While,
    Return,
    True,
    False,
    Null,
    This,
}

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Keyword::Class => write!(f, "class"),
            Keyword::Method => write!(f, "method"),
            Keyword::Function => write!(f, "function"),
            Keyword::Constructor => write!(f, "constructor"),
            Keyword::Int => write!(f, "int"),
            Keyword::Boolean => write!(f, "boolean"),
            Keyword::Char => write!(f, "char"),
            Keyword::Void => write!(f, "void"),
            Keyword::Var => write!(f, "var"),
            Keyword::Static => write!(f, "static"),
            Keyword::Field => write!(f, "field"),
            Keyword::Let => write!(f, "let"),
            Keyword::Do => write!(f, "do"),
            Keyword::If => write!(f, "if"),
            Keyword::Else => write!(f, "else"),
            Keyword::While => write!(f, "while"),
            Keyword::Return => write!(f, "return"),
            Keyword::True => write!(f, "true"),
            Keyword::False => write!(f, "false"),
            Keyword::Null => write!(f, "null"),
            Keyword::This => write!(f, "this"),
        }
    }
}

#[derive(Debug)]
pub struct Tokenizer {
    // [beg, end)
    token: ops::Range<usize>,
    source: String,
}

impl Tokenizer {
    fn keywords() -> Option<&'static HashMap<&'static str, Keyword>> {
        static KEYS: OnceLock<HashMap<&str, Keyword>> = OnceLock::new();
        Some(KEYS.get_or_init(|| {
            let mut m = HashMap::new();
            m.insert("class", Keyword::Class);
            m.insert("method", Keyword::Method);
            m.insert("function", Keyword::Function);
            m.insert("constructor", Keyword::Constructor);
            m.insert("int", Keyword::Int);
            m.insert("boolean", Keyword::Boolean);
            m.insert("char", Keyword::Char);
            m.insert("void", Keyword::Void);
            m.insert("var", Keyword::Var);
            m.insert("static", Keyword::Static);
            m.insert("field", Keyword::Field);
            m.insert("let", Keyword::Let);
            m.insert("do", Keyword::Do);
            m.insert("if", Keyword::If);
            m.insert("else", Keyword::Else);
            m.insert("while", Keyword::While);
            m.insert("return", Keyword::Return);
            m.insert("true", Keyword::True);
            m.insert("false", Keyword::False);
            m.insert("null", Keyword::Null);
            m.insert("this", Keyword::This);

            m
        }))
    }

    fn symbols() -> &'static String {
        static SYMBOLS: OnceLock<String> = OnceLock::new();
        SYMBOLS.get_or_init(|| "{}()[].,;+-*/&|<>=~".to_string())
    }

    // source: .jack file path
    // if fail to open 'source', exit with 1
    pub fn new(source: &str) -> Result<Self, io::Error> {
        // open source file
        let reader = BufReader::new(File::open(source)?);

        // start with //
        let mut buffer: String = reader
            .lines()
            .map(|l| {
                let line = l.expect("Failed to read line");
                match line.find("//") {
                    Some(hit) => line[..hit].to_string(),
                    None => line,
                }
            })
            .collect();

        // /* */ or /** */
        let mut beg = 0;
        while let Some(rel_start) = buffer[beg..].find("/*") {
            let start = beg + rel_start;
            match buffer[start..].find("*/") {
                Some(rel_end) => {
                    let end = start + rel_end + 2;
                    buffer.drain(start..end);
                    beg = start;
                }
                None => break,
            }
        }

        // trim whitespac
        let _ = buffer.trim();

        // construct Tokenizer
        // token points at Before-First
        Ok(Self {
            source: buffer,
            token: ops::Range::default(),
        })
    }

    pub fn has_more_tokens(&mut self) -> bool {
        self.token.end < self.source.len()
    }

    pub fn advance(&mut self) -> Result<(), std::io::Error> {
        let mut win = self.token.end..self.token.end;

        // skip whitespaces
        while self
            .source
            .as_bytes()
            .get(win.end)
            .is_some_and(|b| b.is_ascii_whitespace())
        {
            win.end += 1;
        }

        win.start = win.end;

        // string constant: consume up to the matching closing quote as one
        // token, ignoring any whitespace/symbol characters inside it
        if self.source.as_bytes().get(win.start) == Some(&b'"') {
            win.end = match self.source[win.start + 1..].find('"') {
                Some(pos) => win.start + 1 + pos + 1,
                None => self.source.len(),
            };
            self.token = win.clone();
            return Ok(());
        }

        // return if head is a symbol
        if self
            .source
            .as_bytes()
            .get(win.start)
            .is_some_and(|&ch| Self::symbols().contains(ch as char))
        {
            win.end += 1;
            self.token = win.clone();
            return Ok(());
        }

        // find end of token
        let end = self.source[win.start..]
            .find(|byte: char| byte.is_ascii_whitespace() || Self::symbols().contains(byte));

        win.end = match end {
            Some(pos) => win.start + pos,
            None => self.source.len(),
        };

        self.token = win.clone();

        Ok(())
    }

    pub fn token_type(&self) -> Option<TokenType> {
        let token = &self.source[self.token.clone()];

        // symbol
        let symbols = Self::symbols();
        if symbols.contains(token) {
            return Some(TokenType::Symbol);
        }

        // keyword
        let keywords = Self::keywords()?;
        if keywords.keys().any(|&t| t == token) {
            return Some(TokenType::Keyword);
        }

        // Integer const
        // 0~32767
        if token.parse::<i32>().is_ok() {
            return Some(TokenType::IntConst);
        }

        // String const
        let begin = token.chars().nth(0)?;
        let last = token.chars().nth_back(0)?;
        if begin == '\"'
            && last == '\"'
            && token.len() >= 2
            && token
                .chars()
                .skip(1)
                .take(token.len() - 2)
                .all(|c| c != '\n' && c != '\"' && c != '\r')
        {
            return Some(TokenType::StringConst);
        }

        // identifier
        // 1. Alphabets
        // 2. Numbers
        // 3. Under score(_)
        // 4. Not start from numbers
        if !begin.is_ascii_digit()
            && token
                .chars()
                .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || c == '_')
        {
            return Some(TokenType::Identifier);
        }

        None
    }

    pub fn keyword(&self) -> Option<&Keyword> {
        if !self.token_type()?.eq(&TokenType::Keyword) {
            return None;
        }

        let keywords = Self::keywords()?;
        let key = &self.source[self.token.clone()];

        keywords.get(key)
    }

    pub fn symbol(&self) -> Option<char> {
        if self.source.len() >= self.token.end {
            return self.source.chars().nth(self.token.start);
        }

        None
    }
    pub fn identifier(&self) -> Option<&str> {
        self.eq_token_type(&TokenType::Identifier)
    }
    pub fn int_val(&self) -> Option<&str> {
        self.eq_token_type(&TokenType::IntConst)
    }
    pub fn string_val(&self) -> Option<&str> {
        // strip the surrounding quotes; the value itself never contains one
        let with_quotes = self.eq_token_type(&TokenType::StringConst)?;
        Some(&with_quotes[1..with_quotes.len() - 1])
    }

    fn eq_token_type(&self, t: &TokenType) -> Option<&str> {
        if !self.token_type()?.eq(t) {
            return None;
        }

        Some(&self.source[self.token.clone()])
    }
}

#[cfg(test)]
mod test {
    use rstest::{fixture, rstest};
    use std::io::{Read, Write};
    use tempfile::NamedTempFile;

    use super::*;

    fn set_token_text(tokenizer: &mut Tokenizer, text: &str) {
        tokenizer.source = text.to_string();
        tokenizer.token = 0..text.len();
    }

    #[fixture]
    pub fn void_function_main() -> Tokenizer {
        let src = "function void main() {
                            var Array a;
                            var int length;
                            var int i;

                            let length = Keyboard.readInt(\"HOW MANY NUMBERS? \");
                            let a = Array.new(length);
                            let i = 0;
                        }";
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "{}", src).unwrap();

        Tokenizer::new(file.path().to_str().unwrap()).unwrap()
    }

    #[rstest]
    fn test_has_more_token(
        mut void_function_main: Tokenizer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert!(void_function_main.has_more_tokens());

        void_function_main.token.end = void_function_main.source.len();

        assert!(!void_function_main.has_more_tokens());

        Ok(())
    }

    #[rstest]
    #[case("function", 1)]
    #[case("void", 2)]
    #[case("main", 3)]
    #[case("(", 4)]
    #[case(")", 5)]
    #[case("{", 6)]
    #[case("var", 7)]
    #[case("Array", 8)]
    #[case("a", 9)]
    #[case(";", 10)]
    fn test_advance(
        mut void_function_main: Tokenizer,
        #[case] token: String,
        #[case] skip: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for _ in 0..skip {
            void_function_main.advance().unwrap();
        }

        assert_eq!(
            void_function_main.source[void_function_main.token.clone()],
            token
        );

        Ok(())
    }
    //

    #[fixture]
    fn tokenizer_for_token_text() -> Tokenizer {
        let mut f = NamedTempFile::new().unwrap();
        let mut source = String::new();
        f.read_to_string(&mut source).unwrap();

        Tokenizer {
            source,
            token: 0..0,
        }
    }

    #[rstest]
    fn test_token_type_for_symbol(mut tokenizer_for_token_text: Tokenizer) {
        for keyword in Tokenizer::symbols().chars() {
            set_token_text(&mut tokenizer_for_token_text, &keyword.to_string());
            assert_eq!(
                tokenizer_for_token_text.token_type(),
                Some(TokenType::Symbol)
            );
        }
    }

    #[rstest]
    fn test_token_type_for_keyword(mut tokenizer_for_token_text: Tokenizer) {
        for &keyword in Tokenizer::keywords().unwrap().keys() {
            set_token_text(&mut tokenizer_for_token_text, keyword);
            assert_eq!(
                tokenizer_for_token_text.token_type(),
                Some(TokenType::Keyword)
            );
        }
    }

    #[rstest]
    fn test_token_type_for_int_const(mut tokenizer_for_token_text: Tokenizer) {
        for &keyword in Tokenizer::keywords().unwrap().keys() {
            set_token_text(&mut tokenizer_for_token_text, keyword);
            assert_eq!(
                tokenizer_for_token_text.token_type(),
                Some(TokenType::Keyword)
            );
        }
    }

    // int const valid
    #[rstest]
    fn test_token_type_for_int_const_valid(mut tokenizer_for_token_text: Tokenizer) {
        let int_const_valid = [0, 32767, 17, 314];
        for i in int_const_valid {
            set_token_text(&mut tokenizer_for_token_text, &i.to_string());
            assert_eq!(
                tokenizer_for_token_text.token_type(),
                Some(TokenType::IntConst)
            );
        }
    }

    // int const valid
    #[rstest]
    fn test_token_type_for_int_const_invalid(mut tokenizer_for_token_text: Tokenizer) {
        let int_const_invalid = [32768];
        for i in int_const_invalid {
            set_token_text(&mut tokenizer_for_token_text, &i.to_string());
            assert_eq!(tokenizer_for_token_text.token_type(), None);
        }
    }

    // string const valid
    #[rstest]
    #[case("\"abcdefg\"")]
    #[case("\"\"")]
    fn test_token_type_for_string_const_valid(
        mut tokenizer_for_token_text: Tokenizer,
        #[case] input: &str,
    ) {
        set_token_text(&mut tokenizer_for_token_text, input);
        assert_eq!(
            tokenizer_for_token_text.token_type(),
            Some(TokenType::StringConst)
        );
    }

    // string const invalid
    #[rstest]
    fn test_token_type_for_string_const_invalid(mut tokenizer_for_token_text: Tokenizer) {
        let string_const = ["\"abc\"defg\"", "\"aaa\naaa\"", "\"aaa\raaa\""];
        for s in string_const {
            set_token_text(&mut tokenizer_for_token_text, s);
            assert_eq!(tokenizer_for_token_text.token_type(), None);
        }
    }

    // identifier valid
    #[rstest]
    fn test_token_type_for_identifier_valid(mut tokenizer_for_token_text: Tokenizer) {
        let identifier = ["main", "CamelCase", "DevideBy10", "snake_case"];
        for id in identifier {
            set_token_text(&mut tokenizer_for_token_text, id);
            assert_eq!(
                tokenizer_for_token_text.token_type(),
                Some(TokenType::Identifier)
            );
        }
    }

    // identifier invalid
    #[rstest]
    fn test_token_type_for_identifier_invalid(mut tokenizer_for_token_text: Tokenizer) {
        let identifier = ["10Good", "0Bad"];
        for id in identifier {
            set_token_text(&mut tokenizer_for_token_text, id);
            assert_eq!(tokenizer_for_token_text.token_type(), None);
        }
    }
}
