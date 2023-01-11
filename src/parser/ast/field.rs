use crate::parser::ast::decorator::Decorator;
use crate::parser::ast::identifier::Identifier;
use crate::parser::ast::r#type::Type;
use crate::parser::ast::span::Span;

#[derive(Debug)]
pub(crate) enum FieldClass {
    Unresolved,
    Field,
    Relation,
    Property,
}

#[derive(Debug)]
pub(crate) struct Field {
    pub(crate) identifier: Identifier,
    pub(crate) r#type: Type,
    pub(crate) decorators: Vec<Decorator>,
    pub(crate) span: Span,
    pub(crate) resolved: bool,
    pub(crate) field_class: FieldClass,
}

impl Field {
    pub(crate) fn new(identifier: Identifier, r#type: Type, decorators: Vec<Decorator>, span: Span) -> Self {
        Self {
            identifier, r#type, decorators, span, resolved: false, field_class: FieldClass::Unresolved,
        }
    }

    pub(crate) fn resolve(&mut self, field_class: FieldClass) {
        self.field_class = field_class;
        self.resolved = true;
    }

    pub(crate) fn figure_out_class(&mut self) {
        for decorator in self.decorators.iter() {
            match decorator.expression.as_unit() {
                Some(unit) => {
                    let name = unit.expressions.get(0).unwrap().as_identifier().unwrap().name.as_str();
                    match name {
                        "relation" => {
                            self.field_class = FieldClass::Relation;
                            return;
                        }
                        "getter" => {
                            self.field_class = FieldClass::Property;
                            return;
                        }
                        "setter" => {
                            self.field_class = FieldClass::Property;
                            return;
                        }
                        _ => {}
                    }
                }
                _ => {},
            }
        }
        self.field_class = FieldClass::Field;
    }
}
