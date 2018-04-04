//! File containing Category object of graphql schema
use juniper::ID as GraphqlID;
use juniper::FieldResult;
use hyper::Method;
use futures::Future;
use stq_routes::model::Model;
use stq_routes::service::Service;
use stq_static_resources::Translation;

use graphql::context::Context;
use graphql::models::*;
use super::*;

graphql_object!(Category: Context as "Category" |&self| {
    description: "Category info."

    interfaces: [&Node]

    field id() -> GraphqlID as "Base64 Unique id"{
        ID::new(Service::Stores, Model::Category, self.id).to_string().into()
    }

    field raw_id() -> i32 as "Unique int id"{
        self.id
    }

    field name() -> Vec<Translation> as "Full Name" {
        self.name.clone()
    }

    field meta_field() -> Option<String> as "Meta field" {
        self.meta_field.clone()
    }
    
    field parent_id() -> Option<i32> as "Parent id" {
        self.parent_id.clone()
    }
    
    field level() -> i32 as "Level" {
        self.level.clone()
    }

    field children() -> Vec<Category> as "Children categories" {
        self.children.clone()
    }

    field get_attributes(&executor) -> FieldResult<Vec<Attribute>> as "Fetches category attributes." {
        let context = executor.context();
        let url = format!("{}/{}/{}/attributes",
            context.config.service_url(Service::Stores),
            Model::Category.to_url(),
            self.id
            );

        context.request::<Vec<Attribute>>(Method::Get, url, None)
            .wait()
    }
});
