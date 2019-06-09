#[cfg(test)]
use std::vec::Drain;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
pub struct SourceLocation {
    line: u32,
    column: u32,
}

impl SourceLocation {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum Token {
    Name(String),
    String(String),
    OpeningBrace,
    ClosingBrace,
    KeywordSlide,
    KeywordTitle,
    KeywordMetadata,
}

#[derive(Debug, Eq, PartialEq)]
pub enum TokenizerResult {
    Ok(Token),
    Err(TokenizerFailure),
    End,
}

pub trait TokenStream {
    fn next(&mut self) -> TokenizerResult;
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum TokenizerFailureKind {
    UnexpectedCharacterInName { index: usize },
    UnclosedString,
    UnknownEscapeSequence(char),
    UnfinishedEscapeSequence,
    UnexpectedCharacter(char),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct TokenizerFailure {
    kind: TokenizerFailureKind,
    location: SourceLocation,
}

impl TokenizerFailure {
    pub fn new(location: SourceLocation, kind: TokenizerFailureKind) -> Self {
        Self { location, kind }
    }
}

pub struct PeekableTokenStream<'a, T: TokenStream> {
    token_stream: &'a mut T,
    peeked: Option<TokenizerResult>,
}

impl<'a, T: TokenStream> PeekableTokenStream<'a, T> {
    pub fn new(token_stream: &'a mut T) -> Self {
        PeekableTokenStream {
            token_stream,
            peeked: None,
        }
    }

    pub fn peek(&mut self) -> Option<&TokenizerResult> {
        self.peeked = Some(self.next());

        self.peeked.as_ref()
    }
}

impl<'a, T: TokenStream> TokenStream for PeekableTokenStream<'a, T> {
    fn next(&mut self) -> TokenizerResult {
        match self.peeked.take() {
            Some(p) => {
                self.peeked = None;
                p
            }
            None => self.token_stream.next(),
        }
    }
}

#[cfg(test)]
pub struct MockTokenStream<'a> {
    iter: Drain<'a, TokenizerResult>,
}

#[cfg(test)]
impl<'a> MockTokenStream<'a> {
    pub fn new(results: &'a mut Vec<TokenizerResult>) -> Self {
        MockTokenStream {
            iter: results.drain(..),
        }
    }
}

#[cfg(test)]
impl<'a> TokenStream for MockTokenStream<'a> {
    fn next(&mut self) -> TokenizerResult {
        if let Some(x) = self.iter.next() {
            x
        } else {
            TokenizerResult::End
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn without_peeking_returns_the_stream_verbatim() {
        let mut tokens = vec![
            TokenizerResult::Ok(Token::KeywordSlide),
            TokenizerResult::Ok(Token::String("some slide".into())),
            TokenizerResult::Ok(Token::OpeningBrace),
            TokenizerResult::Ok(Token::ClosingBrace),
        ];
        let mut stream = MockTokenStream::new(&mut tokens);
        let mut peekable_stream = PeekableTokenStream::new(&mut stream);

        assert_eq!(
            TokenizerResult::Ok(Token::KeywordSlide),
            peekable_stream.next()
        );
        assert_eq!(
            TokenizerResult::Ok(Token::String("some slide".into())),
            peekable_stream.next()
        );
        assert_eq!(
            TokenizerResult::Ok(Token::OpeningBrace),
            peekable_stream.next()
        );
        assert_eq!(
            TokenizerResult::Ok(Token::ClosingBrace),
            peekable_stream.next()
        );
    }

    #[test]
    pub fn returns_the_same_token_on_next_after_peek() {
        let mut tokens = vec![
            TokenizerResult::Ok(Token::OpeningBrace),
            TokenizerResult::Ok(Token::ClosingBrace),
        ];

        let mut stream = MockTokenStream::new(&mut tokens);
        let mut peekable_stream = PeekableTokenStream::new(&mut stream);

        assert_eq!(
            &TokenizerResult::Ok(Token::OpeningBrace),
            peekable_stream.peek().unwrap()
        );
        assert_eq!(
            TokenizerResult::Ok(Token::OpeningBrace),
            peekable_stream.next()
        );
    }
}
