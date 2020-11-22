//! Module for processing urlencoded data, that is, form posts. This
//! will return HTML results and support the HTML interface to the
//! network router.
//!
use crate::{
    storage::{rules::Route, Mode, Protocol, Rule},
    web::{
        error::{Error, Result},
        resources::Resource,
        Database, DatabaseRef,
    },
};
use askama::Template;
use hyper::{Body, Method, Request, Response};
use std::{collections::HashMap, net::SocketAddr, str::FromStr};
use url::form_urlencoded;

#[derive(Template)]
#[template(path = "one_rule.html")]
struct OneRuleTemplate<'a> {
    rule: &'a Rule,
}

#[derive(Template)]
#[template(path = "all_rules.html")]
struct AllRulesTemplate<'a> {
    rules: Vec<&'a Rule>,
}

#[derive(Template)]
#[template(path = "all_routes.html")]
struct AllRoutesTemplate<'a> {
    rule: &'a Rule,
    routes: Vec<&'a Route>,
}

/// Route the request to the correct submodule and process the
/// request. This function will return either a fully constructed
/// response, or an error that will be logged.
pub(crate) async fn process_request(
    database: DatabaseRef,
    resource: Resource,
    req: Request<Body>,
) -> Result<Response<Body>> {
    let bytes = hyper::body::to_bytes(req).await?;
    let params = form_urlencoded::parse(bytes.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    match get_method(&params)? {
        Method::GET => process_get(database, resource).await,
        Method::POST => process_post(database, resource, params).await,
        _ => todo!(),
    }
}

async fn process_get(database: DatabaseRef, resource: Resource) -> Result<Response<Body>> {
    match resource {
        Resource::Rule(None) => all_rules_page(database).await,
        Resource::Rule(Some(rule)) => one_rule_page(database, rule).await,
        Resource::Route(rule, None) => all_routes_page(database, rule).await,
        Resource::Route(_, Some(_)) => todo!("single route page"),
    }
}

async fn process_post(
    database: DatabaseRef,
    resource: Resource,
    params: HashMap<String, String>,
) -> Result<Response<Body>> {
    match resource {
        Resource::Rule(None) => process_add_rule_request(database, params).await,
        Resource::Rule(Some(_rule)) => todo!(),
        Resource::Route(_rule, None) => todo!(),
        Resource::Route(_rule, Some(_route)) => todo!(),
    }
}

fn get_parameter<T: FromStr>(params: &HashMap<String, String>, param: &str) -> Result<T> {
    match params.get(param) {
        Some(param) => match param.parse() {
            Ok(mode) => Ok(mode),
            Err(_err) => Err(Error::BadParameter(param.to_string())),
        },
        None => Err(Error::MissingParameter(param.to_string())),
    }
}

/// Process a rules POST request, which will add a new rule or modify
/// an existing rule.
///
/// action        GET, PUT, DELETE, or POST
/// mode          UDP or TCP
/// source        Address to listen on, including port.
/// destinations  List of destination addresses, as a comma-separated list.
///
/// # Returns
///
///
async fn process_add_rule_request(
    database: DatabaseRef,
    params: HashMap<String, String>,
) -> Result<Response<Body>> {
    let mode: Mode = get_parameter(&params, "mode")?;
    let source: SocketAddr = get_parameter(&params, "source")?;
    let protocol: Protocol = get_parameter(&params, "protocol")?;
    let destinations = match params.get("destinations") {
        Some(param) => param
            .split(',')
            .map(|addr| {
                addr.parse()
                    .map_err(|_| Error::BadParameter(addr.to_string()))
            })
            .collect::<Result<Vec<SocketAddr>>>()?,
        None => {
            return Err(Error::MissingParameter("destinations".to_string()));
        }
    };
    let rule_no = database.lock().await.add_rule(Rule {
        protocol,
        mode,
        source,
        destinations,
    });
    one_rule_page(database, rule_no).await
}

async fn all_rules_page(database: DatabaseRef) -> Result<Response<Body>> {
    let handle = database.lock().await;
    let rules: Vec<&Rule> = handle.rules.iter().map(|(_id, rule)| rule).collect();
    let page = AllRulesTemplate { rules };

    Ok(Response::new(Body::from(page.render()?)))
}

async fn one_rule_page(database: DatabaseRef, rule_no: u32) -> Result<Response<Body>> {
    let handle = database.lock().await;
    let rule = get_rule(&handle, rule_no)?;
    let page = OneRuleTemplate { rule };
    Ok(Response::new(Body::from(page.render()?)))
}

async fn all_routes_page(database: DatabaseRef, rule_no: u32) -> Result<Response<Body>> {
    let handle = database.lock().await;
    let page = AllRoutesTemplate {
        rule: get_rule(&handle, rule_no)?,
        routes: get_routes(&handle, rule_no)?,
    };
    Ok(Response::new(Body::from(page.render()?)))
}

/// Forms only support GET and POST according to the standard, so we
/// pick the right method based on the "action" field instead.
fn get_method(params: &HashMap<String, String>) -> Result<Method> {
    match params.get("method") {
        Some(action) if action == "GET" => Ok(Method::GET),
        None => Ok(Method::GET),
        Some(action) if action == "PUT" => Ok(Method::PUT),
        Some(action) if action == "DELETE" => Ok(Method::DELETE),
        Some(action) if action == "POST" => Ok(Method::POST),
        Some(action) => Err(Error::BadMethod(action.to_string())),
    }
}

/// Helper function to get a rule or return the apropriate result.
fn get_rule<'a>(
    handle: &'a futures::lock::MutexGuard<'a, Database>,
    rule_no: u32,
) -> Result<&'a Rule> {
    handle.rules.get(&rule_no).ok_or(Error::ResourceNotFound)
}

/// Helper function to get a route or return the apropriate result.
fn get_routes<'a>(
    handle: &'a futures::lock::MutexGuard<'a, Database>,
    rule_no: u32,
) -> Result<Vec<&'a Route>> {
    let routes = handle
        .routes
        .get(&rule_no)
        .ok_or(Error::UnsatisfiedInvariant)?;
    Ok(routes.iter().collect::<Vec<&'a Route>>())
}
