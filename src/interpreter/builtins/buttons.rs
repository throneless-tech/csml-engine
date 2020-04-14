use crate::data::primitive::{object::PrimitiveObject};
use crate::data::{ast::Interval, tokens::DEFAULT, Literal};
use crate::error_format::*;
use crate::interpreter::builtins::tools::*;
use std::collections::HashMap;

pub fn button(args: HashMap<String, Literal>, interval: Interval) -> Result<Literal, ErrorInfo> {
    let mut button: HashMap<String, Literal> = args.clone();

    let title = match (button.remove("title"), button.remove(DEFAULT)) {
        (Some(title), ..) | (.., Some(title)) => {
            title
        },
        _ => return Err(gen_error_info(interval, ERROR_BUTTON.to_owned())),
    };

    button.insert("payload".to_owned(), title.clone());
    button.insert("title".to_owned(), title.to_owned());

    button.insert(
        "accepts".to_owned(),
        format_accept(button.get("accepts"), title),
    );

    // let button_type = match args.get("button_type") {
    //     Some(button_type) => button_type.to_owned(),
    //     None => PrimitiveString::get_literal("quick_button", interval.to_owned()),
    // };
    // button.insert("button_type".to_owned(), button_type);
    // match args.get("theme") {
    //     Some(theme) => {
    //         button.insert("theme".to_owned(), theme.to_owned());
    //     }
    //     None => {
    //         button.insert(
    //             "theme".to_owned(),
    //             PrimitiveString::get_literal("primary", interval),
    //         );
    //     }
    // };
    // if let Some(icon) = args.get("icon") {
    //     button.insert("icon".to_owned(), icon.to_owned());
    // }

    let mut result = PrimitiveObject::get_literal(&button, interval);
    result.set_content_type("button");

    Ok(result)
}

pub fn card(args: HashMap<String, Literal>, interval: Interval) -> Result<Literal, ErrorInfo> {
    let mut card: HashMap<String, Literal> = args.clone();

    match (card.remove("title"), card.remove(DEFAULT)) {
        (Some(title), ..) | (.., Some(title)) => {
            card.insert("title".to_owned(), title.to_owned());
        },
        _ => return Err(gen_error_info(interval, ERROR_CARD_TITLE.to_owned())),
    };

    match card.get("buttons") {
        Some(..) => {},
        _ => return Err(gen_error_info(interval, ERROR_CARD_BUTTON.to_owned())),
    };

    // if let Some(subtitle) = args.get("subtitle") {
    //     card.insert("subtitle".to_owned(), subtitle.to_owned());
    // }
    // if let Some(image_url) = card.get("image_url") {
    //     card.insert("image_url".to_owned(), image_url.to_owned());
    // }
    let mut result = PrimitiveObject::get_literal(&card, interval);
    result.set_content_type("card");

    Ok(result)
}
