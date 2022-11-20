use crate::parser::ast::expression::StringLiteral;
use crate::parser::ast::identifier::Identifier;
use crate::parser::ast::span::Span;

#[derive(Clone, Debug)]
pub(crate) struct Import {
    pub(crate) id: usize,
    pub(crate) source_id: usize,
    pub(crate) identifiers: Vec<Identifier>,
    pub(crate) source: StringLiteral,
    pub(crate) span: Span,
}
