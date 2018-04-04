//! File containing query object of graphql schema
use std::str::FromStr;

use juniper::ID as GraphqlID;
use juniper::{FieldError, FieldResult};
use hyper::Method;
use futures::Future;
use stq_static_resources::currency::{Currency, CurrencyGraphQl};
use stq_static_resources::{Language, LanguageGraphQl};
use stq_routes::model::Model;
use stq_routes::service::Service;

use graphql::context::Context;
use graphql::models::*;
use super::*;

pub const QUERY_NODE_ID: i32 = 1;

pub struct Query;

graphql_object!(Query: Context |&self| {

    description: "Top level query.

    Remote mark

    Some fields are marked as `Remote`. That means that they are
    part of microservices and their fetching can fail.
    In this case null will be returned (even if o/w
    type signature declares not-null type) and corresponding errors
    will be returned in errors section. Each error is guaranteed
    to have a `code` field and `details field`.

    Codes:
    - 100 - microservice responded,
    but with error http status. In this case `details` is guaranteed
    to have `status` field with http status and
    probably some additional details.

    - 200 - there was a network error while connecting to microservice.

    - 300 - there was a parse error - that usually means that
    graphql couldn't parse api json response
    (probably because of mismatching types on graphql and microservice)
    or api url parse failed.

    - 400 - Unknown error."

    interfaces: [&Node]

    field id() -> GraphqlID as "Base64 Unique id"{
        QUERY_NODE_ID.to_string().into()
    }

    field apiVersion() -> &str as "Current api version." {
        "1.0"
    }

    field static_node_id() -> FieldResult<StaticNodeIds> as "Static node id dictionary." {
        Ok(StaticNodeIds{})
    }

    field me(&executor) -> FieldResult<Option<User>> as "Fetches viewer for users." {
        let context = executor.context();
        let url = format!("{}/{}/current",
            context.config.service_url(Service::Users),
            Model::User.to_url());
        context.request::<User>(Method::Get, url, None)
                    
                    .wait()
                    .map(|u| Some(u))
    }

    field node(&executor, id: GraphqlID as "Base64 Id of a node.") -> FieldResult<Node> as "Fetches graphql interface node by Base64 id."  {
        let context = executor.context();
        if id.to_string() == QUERY_NODE_ID.to_string() {
             Ok(Node::Query(Query{}))
        } else {
            let identifier = ID::from_str(&*id)?;
            match (&identifier.service, &identifier.model) {
                (&Service::Users, &Model::User) => {
                                context.request::<User>(Method::Get, identifier.url(&context.config), None)
                                    .map(|res| Node::User(res))
                                    
                                    .wait()
                },
                (&Service::Users, _) => {
                                Err(FieldError::new(
                                    "Could not get model from users microservice.",
                                    graphql_value!({ "internal_error": "Unknown model" })
                                ))
                },
                (&Service::Stores, &Model::Store) => {
                                context.request::<Store>(Method::Get, identifier.url(&context.config), None)
                                    .map(|res| Node::Store(res))
                                    
                                    .wait()
                },
                (&Service::Stores, &Model::Product) => {
                                context.request::<Product>(Method::Get, identifier.url(&context.config), None)
                                    .map(|res| Node::Product(res))
                                    
                                    .wait()
                },
                (&Service::Stores, &Model::BaseProduct) => {
                                context.request::<BaseProduct>(Method::Get, identifier.url(&context.config), None)
                                    .map(|res| Node::BaseProduct(res))
                                    
                                    .wait()
                },
                (&Service::Stores, &Model::Category) => {
                                context.request::<Category>(Method::Get, identifier.url(&context.config), None)
                                    .map(|res| Node::Category(res))
                                    
                                    .wait()
                },
                (&Service::Stores, &Model::Attribute) => {
                                context.request::<Attribute>(Method::Get, identifier.url(&context.config), None)
                                    .map(|res| Node::Attribute(res))
                                    
                                    .wait()
                },
                (&Service::Stores, _) => {
                                Err(FieldError::new(
                                    "Could not get model from stores microservice.",
                                    graphql_value!({ "internal_error": "Unknown model" })
                                ))
                }
                (&Service::Orders, _) => {
                                Err(FieldError::new(
                                    "Could not get model from orders microservice.",
                                    graphql_value!({ "internal_error": "Unknown model" })
                                ))
                }
            }
        }
    }

    field languages(&executor) -> FieldResult<Vec<LanguageGraphQl>> as "Fetches languages." {
        Ok(Language::as_vec())
    }


    field currencies(&executor) -> FieldResult<Vec<CurrencyGraphQl>> as "Fetches currencies." {
        Ok(Currency::as_vec())
    }

    field categories(&executor) -> FieldResult<Category> as "Fetches categories tree." {
        let context = executor.context();
        let url = format!("{}/{}",
            context.config.service_url(Service::Stores),
            Model::Category.to_url());

        context.request::<Category>(Method::Get, url, None)
            
            .wait()
    }

    field search(&executor) -> FieldResult<Search> as "Search endpoint" {
        Ok(Search{})
    }

    field main_page(&executor) -> FieldResult<MainPage> as "Main page endpoint" {
        Ok(MainPage{})
    }    

});
