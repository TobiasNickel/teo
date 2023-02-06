use crate::core::field::builder::FieldBuilder;

use crate::parser::ast::argument::Argument;
use crate::parser::ast::entity::Entity;


pub(crate) fn default_decorator(args: Vec<Argument>, field: &mut FieldBuilder) {
    match args.get(0).unwrap().resolved.as_ref().unwrap() {
        Entity::Value(value) => {
            field.default(value);
        }
        _ => {
            panic!("Only value default is supported for now.")
        }
    }
}