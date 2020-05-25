use crate::data::literal::ContentType;
use crate::data::position::Position;
use crate::data::primitive::string::PrimitiveString;
use crate::data::{
    ast::{Interval, PathState},
    Data, Literal, MessageData, MSG,
};
use crate::error_format::*;
use crate::interpreter::{
    json_to_rust::json_to_literal,
    variable_handler::{exec_path_actions, resolve_path},
};
use std::sync::mpsc;
use crate::data::ast::PathLiteral;
use crate::interpreter::variable_handler::gen_generic_component::gen_generic_component;

////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

pub fn gen_literal_from_event(
    interval: Interval,
    path: Option<&[(Interval, PathState)]>,
    data: &mut Data,
    root: &mut MessageData,
    sender: &Option<mpsc::Sender<MSG>>,
) -> Result<Literal, ErrorInfo> {
    match path {
        Some(path) => {
            let path = resolve_path(path, data, root, sender)?;
            let mut lit = json_to_literal(&data.event.metadata, interval.to_owned())?;

            lit.set_content_type("event");

            let content_type = match ContentType::get(&lit) {
                ContentType::Event(_) => ContentType::Event(data.event.content_type.to_owned()),
                _ => {
                    return Err(gen_error_info(
                        Position::new(interval),
                        ERROR_EVENT_CONTENT_TYPE.to_owned(),
                    ))
                }
            };

            let (lit, _tmp_mem_update) =
                exec_path_actions(&mut lit, None, &Some(path), &content_type)?;

            Ok(lit)
        }
        None => Ok(PrimitiveString::get_literal(
            &data.event.content,
            interval.to_owned(),
        )),
    }
}

pub fn gen_literal_from_component(
    path: Option<&[(Interval, PathState)]>,
    data: &mut Data,
    root: &mut MessageData,
    sender: &Option<mpsc::Sender<MSG>>,
) -> Result<Literal, ErrorInfo> {
    match path {
        Some(path) => {
            let path = resolve_path(path, data, root, sender)?;

            if let Some((_interval, function_name)) = path.first() {
                if let PathLiteral::Func { name, interval, args } = function_name {

                    if let Some(component) = data.header.get(name) {
                        return gen_generic_component(name, interval, args, component);
                    }
                }
            }

            eprintln!("[!] Path must always exist with GenericComponent\n");
            unimplemented!();
        }
        None => {
            eprintln!("[!] Path must always exist with GenericComponent\n");
            unimplemented!();
        }
    }
}
