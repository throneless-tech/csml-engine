use crate::data::error_info::ErrorInfo;
use crate::data::position::Position;
use crate::data::primitive::{object::PrimitiveObject, string::PrimitiveString, PrimitiveType};
use crate::data::{ast::Interval, ArgsType, Literal};
use crate::error_format::*;
use std::collections::HashMap;
use std::env;

////////////////////////////////////////////////////////////////////////////////
/// PRIVATE FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

fn get_value<'lifetime, T: 'static>(
    key: &str,
    object: &'lifetime HashMap<String, Literal>,
    flow_name: &str,
    interval: Interval,
    error: &'static str,
) -> Result<&'lifetime T, ErrorInfo> {
    if let Some(literal) = object.get(key) {
        Literal::get_value::<T>(&literal.primitive, flow_name, interval, format!("'{}' {}", key, error))
    } else {
        Err(gen_error_info(
            Position::new(interval, flow_name),
            format!("'{}' {}", key, error),
        ))
    }
}

////////////////////////////////////////////////////////////////////////////////
/// PUBLIC FUNCTIONS
////////////////////////////////////////////////////////////////////////////////

pub fn get_url(object: &HashMap<String, Literal>, flow_name: &str, interval: Interval) -> Result<String, ErrorInfo> {
    let url = &mut get_value::<String>("url", object, flow_name,interval, ERROR_HTTP_GET_VALUE)?.to_owned();
    let query =
        get_value::<HashMap<String, Literal>>("query", object, flow_name,interval, ERROR_HTTP_GET_VALUE)?;

    if !query.is_empty() {
        let length = query.len();

        url.push_str("?");

        for (index, key) in query.keys().enumerate() {
            let value = get_value::<String>(key, query, flow_name, interval, ERROR_HTTP_QUERY_VALUES)?;

            url.push_str(key);
            url.push_str("=");
            url.push_str(value);

            if index + 1 < length {
                url.push_str("&");
            }
        }
    }

    Ok(url.to_owned())
}

use ureq::{Agent, AgentBuilder};
use std::time::Duration;
use std::sync::Arc;

use rustls::{
    ClientConfig, Certificate,
    RootCertStore, TLSError,
    client::{ServerCertVerified, ServerName, ServerCertVerifier}
};
// use webpki::DNSNameRef;
// use webpki_roots::TLS_SERVER_ROOTS;

// NoVerifiger taken from https://github.com/seanmonstar/reqwest/blob/a2acf91d9bc43605c1e1d2bb73953dacb02a51c8/src/tls.rs#L320
pub(crate) struct NoVerifier;

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &Certificate,
        _intermediates: &[Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

// pub fn call_danger_accept_invalid_certs(url: &str) -> Result<attohttpc::Response> {
//     let mut client_config = ClientConfig::new();

//     // add cert store
//     client_config
//         .root_store
//         .add_server_trust_anchors(&TLS_SERVER_ROOTS);

//     // disable cert verification
//     client_config
//         .dangerous()
//         .set_certificate_verifier(Arc::new(NoVerifier));

//     let response = attohttpc::get(url).client_config(client_config).send()?;

//     Ok(response)
// }

pub fn http_request(
    object: &HashMap<String, Literal>,
    method: &str,
    flow_name: &str,
    interval: Interval,
) -> Result<serde_json::Value, ErrorInfo> {
    let url = get_url(object, flow_name, interval)?;

    let header =
        get_value::<HashMap<String, Literal>>("header", object, flow_name, interval, ERROR_HTTP_GET_VALUE)?;

    let root_store = rustls::RootCertStore::empty();

    let mut tls_config = rustls::ClientConfig::builder()
    .with_safe_defaults()
    .with_root_certificates(root_store)
    .with_no_client_auth();

    tls_config.dangerous().set_certificate_verifier(Arc::new(NoVerifier));

    let agent: ureq::Agent = ureq::AgentBuilder::new()
    .tls_config(Arc::new(tls_config))
    .build();

    // let mut request =  match method {
    //     delete if delete == "delete" => ureq::delete(&url),
    //     put if put == "put" => ureq::put(&url),
    //     patch if patch == "patch" => ureq::request("PATCH",&url),
    //     post if post == "post" => ureq::post(&url),
    //     get if get == "get" => ureq::get(&url),
    //     _ => {
    //         return Err(gen_error_info(
    //             Position::new(interval, flow_name),
    //             ERROR_HTTP_UNKNOWN_METHOD.to_string(),
    //         ))
    //     }
    // };

    let mut request =  match method {
        delete if delete == "delete" => agent.delete(&url),
        put if put == "put" => agent.put(&url),
        patch if patch == "patch" => agent.request("PATCH",&url),
        post if post == "post" => agent.post(&url),
        get if get == "get" => agent.get(&url),
        _ => {
            return Err(gen_error_info(
                Position::new(interval, flow_name),
                ERROR_HTTP_UNKNOWN_METHOD.to_string(),
            ))
        }
    };

    for key in header.keys() {
        let value = get_value::<String>(key, header, flow_name, interval, ERROR_HTTP_GET_VALUE)?;

        request = request.set(key, value);
    }

    let response = match object.get("body") {
        Some(body) => request.send_json(body.primitive.to_json()),
        None => request.call(),
    };

    match response {
        Ok(response) => {
            match response.into_json() {
                Ok(value) => Ok(value),
                Err(_) => Err(gen_error_info(
                    Position::new(interval, flow_name),
                    ERROR_FAIL_RESPONSE_JSON.to_owned(),
                )),
            }
        }
        Err(err) => {
            if let Ok(var) = env::var("DEBUG") {
                if var == "true" {
                    eprintln!("FN request failed: {:?}", err.to_string());
                }
            }
            return Err(gen_error_info(Position::new(interval, flow_name), err.to_string()));
        }
    }
}

pub fn http(args: ArgsType, flow_name: &str, interval: Interval) -> Result<Literal, ErrorInfo> {
    let mut http: HashMap<String, Literal> = HashMap::new();
    let mut header = HashMap::new();

    match args.get("url", 0) {
        Some(literal) if literal.primitive.get_type() == PrimitiveType::PrimitiveString => {
            header.insert(
                "content-type".to_owned(),
                PrimitiveString::get_literal("application/json", interval),
            );
            header.insert(
                "accept".to_owned(),
                PrimitiveString::get_literal("application/json,text/*", interval),
            );
            header.insert(
                "User-Agent".to_owned(),
                PrimitiveString::get_literal("csml/v1", interval),
            );

            http.insert("url".to_owned(), literal.to_owned());
            http.insert(
                "method".to_owned(),
                PrimitiveString::get_literal("get", interval),
            );

            let lit_header = PrimitiveObject::get_literal(&header, interval);
            http.insert("header".to_owned(), lit_header);
            let lit_query = PrimitiveObject::get_literal(&HashMap::default(), interval);
            http.insert("query".to_owned(), lit_query);
            let lit_body = PrimitiveObject::get_literal(&HashMap::default(), interval);
            http.insert("body".to_owned(), lit_body);

            args.populate(&mut http, &["url", "header", "query", "body"], flow_name, interval)?;

            let mut result = PrimitiveObject::get_literal(&http, interval);

            result.set_content_type("http");

            Ok(result)
        }
        _ => Err(gen_error_info(
            Position::new(interval, flow_name),
            ERROR_HTTP.to_owned(),
        )),
    }
}
