use crate::{
    error::TokenError,
    lexer::Lexer,
    lexer::{AssignOrder, Statement},
    prefix_extension::PrefixState,
    token::{Category, Token},
    unclosed_token, unexpected_token,
};

pub(crate) trait Grouping {
    /// Parses (...)
    fn parse_paren(&mut self, token: Token) -> Result<Statement, TokenError>;
    /// Parses {...}
    fn parse_block(&mut self, token: Token) -> Result<Statement, TokenError>;
    /// General Grouping parsing. Is called within prefix_extension.
    fn parse_grouping(&mut self, token: Token) -> Result<(PrefixState, Statement), TokenError>;
}

impl<'a> Grouping for Lexer<'a> {
    fn parse_paren(&mut self, token: Token) -> Result<Statement, TokenError> {
        let lhs = self.expression_bp(0, Category::RightParen)?;
        let actual = self.end_category.unwrap_or(Category::Equal);
        if actual != Category::RightParen {
            Err(unclosed_token!(token))
        } else {
            self.unhandled_token = None;
            match lhs {
                Statement::Assign(category, _, token, stmt) => Ok(Statement::Assign(
                    category,
                    AssignOrder::AssignReturn,
                    token,
                    stmt,
                )),
                _ => Ok(lhs),
            }
        }
    }

    fn parse_block(&mut self, token: Token) -> Result<Statement, TokenError> {
        let mut results = vec![];
        while let Some(token) = self.tokenizer.next() {
            if token.category() == Category::RightCurlyBracket {
                self.unhandled_token = None;
                return Ok(Statement::Block(results));
            }
            self.unhandled_token = Some(token);
            // use min_bp 1 to skip the unhandled_token reset due to self.tokenizer.next call
            results.push(self.expression_bp(1, Category::Semicolon)?);
        }
        Err(unclosed_token!(token))
    }

    fn parse_grouping(&mut self, token: Token) -> Result<(PrefixState, Statement), TokenError> {
        match token.category() {
            Category::LeftParen => self
                .parse_paren(token)
                .map(|stmt| (PrefixState::Continue, stmt)),
            Category::LeftCurlyBracket => self
                .parse_block(token)
                .map(|stmt| (PrefixState::Break, stmt)),
            _ => Err(unexpected_token!(token)),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        lexer::{AssignOrder, Statement},
        parse,
        token::{Base, Category, Token},
    };

    use Base::*;
    use Category::*;
    use Statement::*;

    fn result(code: &str) -> Statement {
        parse(code)[0].as_mut().unwrap().clone()
    }

    #[test]
    fn variables() {
        assert_eq!(
            result(
                r"
            {
                a = b + 1;
                b = a - --c;
                {
                   d = 23;
                }
            }
            "
            ),
            Block(vec![
                Assign(
                    Equal,
                    AssignOrder::Assign,
                    Token {
                        category: Identifier(None),
                        position: (31, 32)
                    },
                    Box::new(Operator(
                        Plus,
                        vec![
                            Variable(Token {
                                category: Identifier(None),
                                position: (35, 36)
                            }),
                            Primitive(Token {
                                category: Number(Base10),
                                position: (39, 40)
                            })
                        ]
                    ))
                ),
                Assign(
                    Equal,
                    AssignOrder::Assign,
                    Token {
                        category: Identifier(None),
                        position: (58, 59)
                    },
                    Box::new(Operator(
                        Minus,
                        vec![
                            Variable(Token {
                                category: Identifier(None),
                                position: (62, 63)
                            }),
                            Assign(
                                MinusMinus,
                                AssignOrder::AssignReturn,
                                Token {
                                    category: Identifier(None),
                                    position: (68, 69)
                                },
                                Box::new(Operator(
                                    Minus,
                                    vec![
                                        Variable(Token {
                                            category: Identifier(None),
                                            position: (68, 69)
                                        }),
                                        RawNumber(1)
                                    ]
                                ))
                            )
                        ]
                    ))
                ),
                Block(vec![Assign(
                    Equal,
                    AssignOrder::Assign,
                    Token {
                        category: Identifier(None),
                        position: (108, 109)
                    },
                    Box::new(Primitive(Token {
                        category: Number(Base10),
                        position: (112, 114)
                    }))
                )])
            ])
        );
    }
}
