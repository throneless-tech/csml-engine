use crate::parser::{
    ast::{Expr, Identifier, InstructionInfo, RangeInterval},
    context::*,
    parse_comments::comment,
    parse_idents::parse_idents,
    parse_scope::parse_scope,
    parse_var_types::parse_var_expr,
    tokens::{Span, COMMA, FOREACH, IN, L_PAREN, R_PAREN},
    tools::get_interval,
};
use nom::{bytes::complete::tag, combinator::opt, error::ParseError, sequence::preceded, *};

fn pars_args<'a, E: ParseError<Span<'a>>>(s: Span<'a>) -> IResult<Span<'a>, Identifier, E> {
    let (s, _) = preceded(comment, tag(COMMA))(s)?;
    let (s, idents) = parse_idents(s)?;
    Ok((s, idents))
}

pub fn parse_foreach<'a, E: ParseError<Span<'a>>>(
    s: Span<'a>,
) -> IResult<Span<'a>, (Expr, InstructionInfo), E> {
    let (s, _) = preceded(comment, tag(FOREACH))(s)?;
    let (s, start) = get_interval(s)?;

    let (s, _) = preceded(comment, tag(L_PAREN))(s)?;
    let (s, idents) = parse_idents(s)?;
    let (s, opt) = opt(pars_args)(s)?;
    let (s, _) = preceded(comment, tag(R_PAREN))(s)?;

    let (s, _) = preceded(comment, tag(IN))(s)?;
    let (s, expr) = parse_var_expr(s)?;

    let index = Context::get_index();

    Context::inc_index();

    Context::set_state(State::Loop);
    let (s, block) = parse_scope(s)?;
    Context::set_state(State::Normal);

    let (s, end) = get_interval(s)?;

    let new_index = Context::get_index() - 1;
    let instruction_info = InstructionInfo {
        index,
        total: new_index - index,
    };

    Ok((
        s,
        (
            Expr::ForEachExpr(
                idents,
                opt,
                Box::new(expr),
                block,
                RangeInterval { start, end },
            ),
            instruction_info,
        ),
    ))
}
