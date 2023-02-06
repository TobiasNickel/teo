use crate::core::field::builder::FieldBuilder;

use crate::parser::ast::argument::Argument;

pub(crate) fn auth_identity_decorator(_args: Vec<Argument>, field: &mut FieldBuilder) {
    field.auth_identity();
}