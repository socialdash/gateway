//! File containing node object of graphql schema
//! File containing store object of graphql schema
use std::cmp;
use std::str::FromStr;

use chrono::prelude::*;
use futures::Future;
use hyper::Method;
use juniper;
use juniper::ID as GraphqlID;
use juniper::{FieldError, FieldResult};
use serde_json;

use stq_api::orders::{OrderClient, OrderSearchTerms};
use stq_api::types::ApiFutureExt;
use stq_api::warehouses::WarehouseClient;
use stq_routes::model::Model;
use stq_routes::service::Service;
use stq_static_resources::{Language, ModerationStatus, Translation};
use stq_types::{OrderIdentifier, OrderSlug};

use super::*;
use errors::into_graphql;
use graphql::context::Context;
use graphql::models::*;

const MIN_ID: i32 = 0;

graphql_object!(Store: Context as "Store" |&self| {
    description: "Store's info."

    interfaces: [&Node]

    field id() -> GraphqlID as "Base64 Unique id"{
        ID::new(Service::Stores, Model::Store, self.id.0).to_string().into()
    }

    field raw_id() -> &i32 as "Unique int id"{
        &self.id.0
    }

    field user_id() -> &i32 as "Store manager id"{
        &self.user_id.0
    }

    field store_manager(&executor) -> FieldResult<Option<User>> as "Fetches store manager by user_id." {
        let context = executor.context();

        let url = format!(
            "{}/{}/{}",
            &context.config.service_url(Service::Users),
            Model::User.to_url(),
            self.user_id.to_string()
        );

        context.request::<Option<User>>(Method::Get, url, None)
            .wait()
    }

    field name() -> &[Translation] as "Full Name" {
        &self.name
    }

    field isActive() -> &bool as "If the store was disabled (deleted), isActive is false" {
        &self.is_active
    }

    field short_description() -> &[Translation] as "Short description" {
        &self.short_description
    }

    field long_description() -> &Option<Vec<Translation>> as "Long description" {
        &self.long_description
    }

    field slug() -> &str as "Slug" {
        &self.slug
    }

    field created_at() -> String as "Created at" {
        let datetime: DateTime<Utc> = self.created_at.into();
        datetime.to_rfc3339()
    }

    field updated_at() -> String as "Updated at" {
        let datetime: DateTime<Utc> = self.updated_at.into();
        datetime.to_rfc3339()
    }

    field cover() -> &Option<String> as "Cover" {
        &self.cover
    }

    field logo() -> &Option<String> as "Logo" {
        &self.logo
    }

    field phone() -> &Option<String> as "Phone" {
        &self.phone
    }

    field email() -> &Option<String> as "Email" {
        &self.email
    }

    field facebook_url() -> &Option<String> as "Facebook url" {
        &self.facebook_url
    }

    field twitter_url() -> &Option<String> as "Twitter url" {
        &self.twitter_url
    }

    field instagram_url() -> &Option<String> as "Instagram url" {
        &self.instagram_url
    }

    field default_language() -> &Language as "Default language" {
        &self.default_language
    }

    field slogan() -> &Option<String> as "Slogan" {
        &self.slogan
    }

    field rating() -> &f64 as "Rating" {
        &self.rating
    }

    field status() -> &ModerationStatus as "Moderation Status" {
        &self.status
    }

    field deprecated "Use address_full -> value" address() -> &Option<String> as "address" {
        &self.address
    }

    field deprecated "Use address_full -> country" country() -> &Option<String> as "country" {
        &self.country
    }

    field address_full() -> Address as "full address" {
        self.clone().into()
    }

    field base_products(&executor,
        first = None : Option<i32> as "First edges",
        after = None : Option<GraphqlID> as "After base_product GraphQL id",
        skip_base_prod_id = None : Option<i32> as "Skip base prod id",
        visibility: Option<Visibility> as "Specifies visibility of the base products")
            -> FieldResult<Option<Connection<BaseProduct, PageInfo>>> as "Fetches base products of the store." {
        let context = executor.context();

        let offset = after
            .and_then(|val| ID::from_str(&*val).map(|id| id.raw_id + 1).ok())
            .unwrap_or_default();
        let visibility = visibility.unwrap_or(Visibility::Active);

        let records_limit = context.config.gateway.records_limit;
        let count = cmp::min(first.unwrap_or(records_limit as i32), records_limit as i32);

            let url = match skip_base_prod_id {
                None => format!(
                        "{}/{}/{}/products?offset={}&count={}&visibility={}",
                        &context.config.service_url(Service::Stores),
                        Model::Store.to_url(),
                        self.id,
                        offset,
                        count + 1,
                        visibility,
                    ),
                Some(id) => format!(
                        "{}/{}/{}/products?skip_base_product_id={}&offset={}&count={}&visibility={}",
                        &context.config.service_url(Service::Stores),
                        Model::Store.to_url(),
                        self.id,
                        id,
                        offset,
                        count + 1,
                        visibility,
                    )
            };

            context.request::<Vec<BaseProduct>>(Method::Get, url, None)
                .map (|base_products| {
                    let mut base_product_edges: Vec<Edge<BaseProduct>> =  vec![];
                    for base_product in base_products {
                        let edge = Edge::new(
                                juniper::ID::from(ID::new(Service::Stores, Model::BaseProduct, base_product.id.0).to_string()),
                                base_product.clone()
                            );
                        base_product_edges.push(edge);
                    }
                    let has_next_page = base_product_edges.len() as i32 > count;
                    if has_next_page {
                        base_product_edges.pop();
                    };
                    let has_previous_page = true;
                    let start_cursor =  base_product_edges.get(0).map(|e| e.cursor.clone());
                    let end_cursor = base_product_edges.iter().last().map(|e| e.cursor.clone());
                    let page_info = PageInfo {
                        has_next_page,
                        has_previous_page,
                        start_cursor,
                        end_cursor};
                    Connection::new(base_product_edges, page_info)
                })
                .wait()
                .map(Some)
        }

    field products_count(&executor) -> FieldResult<i32> as "Fetches products count of the store." {
        let context = executor.context();

        let url = format!(
            "{}/{}/{}/products/count",
            &context.config.service_url(Service::Stores),
            Model::Store.to_url(),
            self.id,
        );

        context.request::<i32>(Method::Get, url, None)
            .wait()
    }

    field moderator_comment(&executor) -> FieldResult<Option<ModeratorStoreComments>> as "Fetches moderator comment by id." {
        let context = executor.context();

        let url = format!(
            "{}/{}/{}",
            &context.config.service_url(Service::Stores),
            Model::ModeratorStoreComment.to_url(),
            self.id.to_string()
        );

        context.request::<Option<ModeratorStoreComments>>(Method::Get, url, None)
            .wait()
    }

    field warehouses(&executor) -> FieldResult<Vec<GraphQLWarehouse>> as "Fetches store warehouses." {
        let context = executor.context();

        let rpc_client = context.get_rest_api_client(Service::Warehouses);
        rpc_client.get_warehouses_for_store(self.id)
            .sync()
            .map_err(into_graphql)
            .map(|res| res.into_iter().map(GraphQLWarehouse).collect())
    }

    field orders(&executor,
        current_page : i32 as "Current page",
        items_count : i32 as "Items count",
        search_term_options : SearchOrderOptionInput as "Search options pattern")
            -> FieldResult<Option<Connection<GraphQLOrder, PageInfoOrdersSearch>>> as "Fetches orders using relay connection." {
        let context = executor.context();

        let offset = items_count * (current_page - 1);

        let records_limit = context.config.gateway.records_limit;
        let count = cmp::min(items_count, records_limit as i32);

        let created_from = match search_term_options.created_from.clone() {
            Some(value) => {
                match value.parse() {
                    Ok(v) => Some(v),
                    Err(_) => return Err(FieldError::new(
                        "Parsing created_from error",
                        graphql_value!({ "code": 300, "details": { "created_from has wrong format." }}),
                    )),
                }
            },
            None => None
        };

        let created_to = match search_term_options.created_to.clone() {
            Some(value) => {
                match value.parse() {
                    Ok(v) => Some(v),
                    Err(_) => return Err(FieldError::new(
                        "Parsing created_to error",
                        graphql_value!({ "code": 300, "details": { "created_to has wrong format." }}),
                    )),
                }
            },
            None => None
        };

        let customer = search_term_options.email.clone().and_then(|email| {
            let url = format!("{}/{}/by_email?email={}",
                context.config.service_url(Service::Users),
                Model::User.to_url(),
                email);

            context.request::<Option<User>>(Method::Get, url, None)
                .wait()
                .ok()
                .and_then (|user| user.map(|u|u.id))
        });

        let search_term = OrderSearchTerms {
                slug: search_term_options.slug.map(OrderSlug),
                customer,
                store: Some(self.id),
                created_from,
                created_to,
                payment_status: search_term_options.payment_status,
                state: search_term_options.order_status,
                ..OrderSearchTerms::default()
            };

        let rpc_client = context.get_rest_api_client(Service::Orders);
        rpc_client.search(search_term)
            .sync()
            .map_err(into_graphql)
            .map(|res| res.into_iter().map(GraphQLOrder).collect())
            .map (move |orders: Vec<GraphQLOrder>| {
                let total_pages = (orders.iter().count() as f32 / items_count as f32).ceil() as i32;

                let mut orders_edges: Vec<Edge<GraphQLOrder>> = orders
                    .into_iter()
                    .skip(offset as usize)
                    .take(count as usize)
                    .map(|order| Edge::new(
                                juniper::ID::from(order.0.id.to_string()),
                                order.clone()
                            ))
                    .collect();

                let page_info = PageInfoOrdersSearch {
                    total_pages,
                    current_page,
                    page_items_count: items_count,
                    search_term_options: search_term_options.into()
                };
                Connection::new(orders_edges, page_info)
            })
            .map(Some)
    }

    field order(&executor, slug: i32 as "Order slug" ) -> FieldResult<Option<GraphQLOrder>> as "Fetches order." {
        let context = executor.context();

        let rpc_client = context.get_rest_api_client(Service::Orders);
        rpc_client.get_order(OrderIdentifier::Slug(OrderSlug(slug)))
            .sync()
            .map_err(into_graphql)
            .map(|res| res.map(GraphQLOrder))
    }

    field find_most_viewed_products(&executor,
        first = None : Option<i32> as "First edges",
        after = None : Option<GraphqlID>  as "Offset from begining",
        search_term : MostViewedProductsInput as "Most viewed search pattern")
            -> FieldResult<Option<Connection<BaseProduct, PageInfo>>> as "Find most viewed base products each one contains one variant." {
        let context = executor.context();

        let offset = after
            .and_then(|id|{
                i32::from_str(&id).map(|i| i + 1).ok()
            })
            .unwrap_or_default();

        let records_limit = context.config.gateway.records_limit;
        let count = cmp::min(first.unwrap_or(records_limit as i32), records_limit as i32);

        let url = format!("{}/{}/most_viewed?offset={}&count={}",
            context.config.service_url(Service::Stores),
            Model::BaseProduct.to_url(),
            offset,
            count + 1
            );

        let options = if let Some(mut options) = search_term.options.clone() {
            options.store_id = Some(self.id.0);
            options
        } else {
            ProductsSearchOptionsInput{
                store_id : Some(self.id.0),
                status : Some(ModerationStatus::Published),
                ..ProductsSearchOptionsInput::default()
            }
        };

        let mut search_term = search_term;
        search_term.options = Some(options);

        let body = serde_json::to_string(&search_term)?;

        context.request::<Vec<BaseProduct>>(Method::Post, url, Some(body))
            .map (|base_products| {
                let mut base_product_edges = Edge::create_vec(base_products, offset);
                let has_next_page = base_product_edges.len() as i32 == count + 1;
                if has_next_page {
                    base_product_edges.pop();
                };
                let has_previous_page = true;
                let start_cursor =  base_product_edges.get(0).map(|e| e.cursor.clone());
                let end_cursor = base_product_edges.iter().last().map(|e| e.cursor.clone());
                let page_info = PageInfo {
                    has_next_page,
                    has_previous_page,
                    start_cursor,
                    end_cursor};
                Connection::new(base_product_edges, page_info)
            })
            .wait()
            .map(Some)
    }


    field find_most_discount_products(&executor,
        first = None : Option<i32> as "First edges",
        after = None : Option<GraphqlID>  as "Offset from begining",
        search_term : MostDiscountProductsInput as "Most discount search pattern")
            -> FieldResult<Option<Connection<BaseProduct, PageInfo>>> as "Find base products each one with most discount variant." {
        let context = executor.context();

        let offset = after
            .and_then(|id|{
                i32::from_str(&id).map(|i| i + 1).ok()
            })
            .unwrap_or_default();

        let records_limit = context.config.gateway.records_limit;
        let count = cmp::min(first.unwrap_or(records_limit as i32), records_limit as i32);

        let url = format!("{}/{}/most_discount?offset={}&count={}",
            context.config.service_url(Service::Stores),
            Model::BaseProduct.to_url(),
            offset,
            count + 1
            );

        let options = if let Some(mut options) = search_term.options.clone() {
            options.store_id = Some(self.id.0);
            options
        } else {
            ProductsSearchOptionsInput{
                store_id : Some(self.id.0),
                status : Some(ModerationStatus::Published),
                ..ProductsSearchOptionsInput::default()
            }
        };

        let mut search_term = search_term;
        search_term.options = Some(options);

        let body = serde_json::to_string(&search_term)?;

        context.request::<Vec<BaseProduct>>(Method::Post, url, Some(body))
            .map (|base_products| {
                let mut base_product_edges = Edge::create_vec(base_products, offset);
                let has_next_page = base_product_edges.len() as i32 == count + 1;
                if has_next_page {
                    base_product_edges.pop();
                };
                let has_previous_page = true;
                let start_cursor =  base_product_edges.get(0).map(|e| e.cursor.clone());
                let end_cursor = base_product_edges.iter().last().map(|e| e.cursor.clone());
                let page_info = PageInfo {
                    has_next_page,
                    has_previous_page,
                    start_cursor,
                    end_cursor};
                Connection::new(base_product_edges, page_info)
            })
            .wait()
            .map(Some)
    }

    field find_product(&executor,
        first = None : Option<i32> as "First edges",
        after = None : Option<GraphqlID>  as "Offset form begining",
        search_term : SearchProductInput as "Search pattern")
            -> FieldResult<Option<Connection<BaseProduct, PageInfoProductsSearch>>> as "Find products by name using relay connection." {

        let context = executor.context();

        let offset = after
            .and_then(|id|{
                i32::from_str(&id).map(|i| i + 1).ok()
            })
            .unwrap_or_default();


        let records_limit = context.config.gateway.records_limit;
        let count = cmp::min(first.unwrap_or(records_limit as i32), records_limit as i32);

        let url = format!("{}/{}/search?offset={}&count={}",
            context.config.service_url(Service::Stores),
            Model::BaseProduct.to_url(),
            offset,
            count + 1
            );

        let options = if let Some(mut options) = search_term.options.clone() {
            options.store_id = Some(self.id.0);
            options.status = Some(ModerationStatus::Published);
            options
        } else {
            ProductsSearchOptionsInput{
                store_id : Some(self.id.0),
                status : Some(ModerationStatus::Published),
                ..ProductsSearchOptionsInput::default()
            }
        };

        let mut search_term = search_term;
        search_term.options = Some(options);

        let body = serde_json::to_string(&search_term)?;

        context.request::<Vec<BaseProduct>>(Method::Post, url, Some(body))
            .map (|products| {
                let mut product_edges = Edge::create_vec(products, offset);
                let search_filters = ProductsSearchFilters::new(search_term);
                let has_next_page = product_edges.len() as i32 == count + 1;
                if has_next_page {
                    product_edges.pop();
                };
                let has_previous_page = true;
                let start_cursor =  product_edges.get(0).map(|e| e.cursor.clone());
                let end_cursor = product_edges.iter().last().map(|e| e.cursor.clone());
                let page_info = PageInfoProductsSearch {
                    has_next_page,
                    has_previous_page,
                    search_filters: Some(search_filters),
                    start_cursor,
                    end_cursor};
                Connection::new(product_edges, page_info)
            })
            .wait()
            .map(Some)
    }

    field auto_complete_product_name(&executor,
        first = None : Option<i32> as "First edges",
        after = None : Option<GraphqlID>  as "Offset form begining",
        name : String as "Name part")
            -> FieldResult<Option<Connection<String, PageInfo>>> as "Finds products full name by part of the name." {

        let context = executor.context();

        let offset = after
            .and_then(|id|{
                i32::from_str(&id).map(|i| i + 1).ok()
            })
            .unwrap_or_default();

        let records_limit = context.config.gateway.records_limit;
        let count = cmp::min(first.unwrap_or(records_limit as i32), records_limit as i32);

        let url = format!("{}/{}/auto_complete?offset={}&count={}",
            context.config.service_url(Service::Stores),
            Model::BaseProduct.to_url(),
            offset,
            count + 1,
            );

        let search_term = AutoCompleteProductNameInput {
            name,
            store_id : Some(self.id.0),
            status: Some(ModerationStatus::Published),
        };

        let body = serde_json::to_string(&search_term)?;

        context.request::<Vec<String>>(Method::Post, url, Some(body))
            .map (|full_names| {
                let mut full_name_edges = Edge::create_vec(full_names, offset);
                let has_next_page = full_name_edges.len() as i32 == count + 1;
                if has_next_page {
                    full_name_edges.pop();
                };
                let has_previous_page = true;
                let start_cursor =  full_name_edges.get(0).map(|e| e.cursor.clone());
                let end_cursor = full_name_edges.iter().last().map(|e| e.cursor.clone());
                let page_info = PageInfo {
                    has_next_page,
                    has_previous_page,
                    start_cursor,
                    end_cursor};
                Connection::new(full_name_edges, page_info)
            })
            .wait()
            .map(Some)
    }

    field balance(&executor) -> FieldResult<Vec<MerchantBalance>> as "Fetches balance of current store." {
        let context = executor.context();

        let url = format!("{}/merchants/store/{}/balance",
            &context.config.service_url(Service::Billing),
            self.id,
            );

        context.request::<Vec<MerchantBalance>>(Method::Get, url, None)
            .wait()
    }

    field find_products_admin(&executor,
        first = None : Option<i32> as "First edges",
        after = None : Option<GraphqlID>  as "Base64 Id of a base_product",
        search_term : SearchModeratorBaseProductInput as "Search pattern"
        )
            -> FieldResult<Option<Connection<BaseProduct, PageInfo>>> as "Searching base_products by moderator using relay connection." {
        let context = executor.context();

        let mut search_term = search_term;
        search_term.store_id = Some(self.id.0);

        let body = serde_json::to_string(&search_term)?;

        let raw_id = match after {
            Some(val) => ID::from_str(&*val)?.raw_id + 1,
            None => MIN_ID
        };

        let records_limit = context.config.gateway.records_limit;
        let first = cmp::min(first.unwrap_or(records_limit as i32), records_limit as i32);

        let url = format!("{}/{}/moderator_search?offset={}&count={}",
            context.config.service_url(Service::Stores),
            Model::BaseProduct.to_url(),
            raw_id,
            first + 1);

        context.request::<Vec<BaseProduct>>(Method::Post, url, Some(body))
            .map (|base_products| {
                let mut base_product_edges: Vec<Edge<BaseProduct>> = base_products
                    .into_iter()
                    .map(|base_product| Edge::new(
                                juniper::ID::from(ID::new(Service::Stores, Model::BaseProduct, base_product.id.0).to_string()),
                                base_product.clone()
                            ))
                    .collect();
                let has_next_page = base_product_edges.len() as i32 == first + 1;
                if has_next_page {
                    base_product_edges.pop();
                };
                let has_previous_page = true;
                let start_cursor =  base_product_edges.get(0).map(|e| e.cursor.clone());
                let end_cursor = base_product_edges.iter().last().map(|e| e.cursor.clone());
                let page_info = PageInfo {
                    has_next_page,
                    has_previous_page,
                    start_cursor,
                    end_cursor};
                Connection::new(base_product_edges, page_info)
            })
            .wait()
            .map(Some)
    }

    field coupons(&executor) -> FieldResult<Vec<Coupon>> {
        let context = executor.context();
        let url = format!("{}/{}/stores/{}",
            context.config.service_url(Service::Stores),
            Model::Coupon.to_url(),
            self.id);
        context.request::<Vec<Coupon>>(Method::Get, url, None).wait()
    }

});

graphql_object!(Connection<Store, PageInfo>: Context as "StoresConnection" |&self| {
    description:"Stores Connection"

    field edges() -> &[Edge<Store>] {
        &self.edges
    }

    field page_info() -> &PageInfo {
        &self.page_info
    }
});

graphql_object!(Connection<Store, PageInfoSegments>: Context as "StoresConnectionPages" |&self| {
    description: "Stores Connection"

    field edges() -> &[Edge<Store>] {
        &self.edges
    }

    field page_info() -> &PageInfoSegments {
        &self.page_info
    }
});

graphql_object!(Edge<Store>: Context as "StoresEdge" |&self| {
    description:"Stores Edge"

    field cursor() -> &juniper::ID {
        &self.cursor
    }

    field node() -> &Store {
        &self.node
    }
});

graphql_object!(Connection<String, PageInfo>: Context as "FullNameConnection" |&self| {
    description:"Name Connection"

    field edges() -> &[Edge<String>] {
        &self.edges
    }

    field page_info() -> &PageInfo {
        &self.page_info
    }
});

graphql_object!(Connection<Store, PageInfoStoresSearch>: Context as "StoresWithTotalCountConnection" |&self| {
    description:"Stores Connection"

    field edges() -> &[Edge<Store>] {
        &self.edges
    }

    field page_info() -> &PageInfoStoresSearch {
        &self.page_info
    }
});

graphql_object!(Edge<String>: Context as "FullNameEdge" |&self| {
    description:"Name Edge"

    field cursor() -> &juniper::ID {
        &self.cursor
    }

    field node() -> &str {
        &self.node
    }
});

graphql_object!(Connection<GraphQLOrder, PageInfoOrdersSearch>: Context as "OrderSearchConnection" |&self| {
    description:"Order Search Connection"

    field edges() -> &[Edge<GraphQLOrder>] {
        &self.edges
    }

    field page_info() -> &PageInfoOrdersSearch {
        &self.page_info
    }
});
