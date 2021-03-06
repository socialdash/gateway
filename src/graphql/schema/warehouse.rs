//! File containing warehouse object of graphql schema
use std::cmp;
use std::str::FromStr;

use futures::Future;
use hyper::Method;
use juniper::ID as GraphqlID;
use juniper::{FieldError, FieldResult};
use serde_json;

use stq_api::types::ApiFutureExt;
use stq_api::warehouses::{Stock, Warehouse, WarehouseClient};
use stq_routes::model::Model;
use stq_routes::service::Service;
use stq_types::{ProductId, Quantity, StockId, StoreId, WarehouseIdentifier};

use super::*;
use errors::into_graphql;
use graphql::context::Context;
use graphql::models::*;

graphql_object!(GraphQLWarehouse: Context as "Warehouse" |&self| {
    description: "Warehouse info."

    interfaces: [&Node]

    field id() -> GraphqlID as "Unique id"{
        self.0.id.to_string().into()
    }

    field name() -> &Option<String> as "Name"{
        &self.0.name
    }

    field location() -> Option<GeoPoint> as "Location"{
        self.0.clone().location.map(|p| GeoPoint{x: p.x(), y: p.y()})
    }

    field slug() -> String as "Slug"{
        self.0.slug.clone().to_string()
    }

    field address_full() -> Address as "Address full"{
        self.0.clone().into()
    }

    field store_id() -> &i32 as "Store_id"{
        &self.0.store_id.0
    }

    field store(&executor,
        visibility: Option<Visibility> as "Specifies allowed visibility of the store",
    ) -> FieldResult<Option<Store>> as "Fetches store." {
        let context = executor.context();
        let visibility = visibility.unwrap_or(Visibility::Active);

        let url = format!(
            "{}/{}/{}?visibility={}",
            &context.config.service_url(Service::Stores),
            Model::Store.to_url(),
            self.0.store_id.to_string(),
            visibility
        );

        context.request::<Option<Store>>(Method::Get, url, None)
            .wait()
    }

    field products(&executor,
        first = None : Option<i32> as "First edges",
        after = None : Option<GraphqlID>  as "Base64 Id of base product",
        current_page : i32 as "Current page",
        items_count : i32 as "Items count",
        search_term : Option<SearchProductInput> as "Search pattern",
        visibility : Option<Visibility> as "Specifies allowed visibility of the base products"
    ) -> FieldResult<Option<Connection<GraphQLStock, PageInfoWarehouseProductSearch>>> as "Find products of the warehouse using relay connection." {

        let context = executor.context();
        let visibility = visibility.unwrap_or(Visibility::Active);

        let offset = items_count * (current_page - 1);

        let records_limit = context.config.gateway.records_limit;
        let count = cmp::min(items_count, records_limit as i32);

        let url = format!("{}/{}/search?offset={}&count={}&visibility={}",
            context.config.service_url(Service::Stores),
            Model::BaseProduct.to_url(),
            offset,
            count,
            visibility
        );

        let search_term = if let Some(search_term) = search_term {
            let options = if let Some(mut options) = search_term.options {
                options.store_id = Some(self.0.store_id.0);
                options
            } else {
                ProductsSearchOptionsInput{
                    store_id : Some(self.0.store_id.0),
                    ..ProductsSearchOptionsInput::default()
                }
            };
            SearchProductInput {
                name: search_term.name,
                options: Some(options)
            }
        } else {
            SearchProductInput {
                name: "".to_string(),
                options: Some(ProductsSearchOptionsInput{
                    store_id : Some(self.0.store_id.0),
                    ..ProductsSearchOptionsInput::default()
                })
            }
        };

        let body = serde_json::to_string(&search_term)?;

        context.request::<Vec<BaseProduct>>(Method::Post, url, Some(body))
            .map(|base_products|
                base_products
                    .into_iter()
                    .flat_map(|base_product|
                        base_product
                            .variants
                            .unwrap_or_default()
                            .into_iter()
                            .map(|mut variant| variant.id)
                    )
                    .collect()
            )
            .wait()
            .and_then (|products: Vec<ProductId>| {
                products.into_iter().map(|product_id| {

                    let rpc_client = context.get_rest_api_client(Service::Warehouses);
                    rpc_client.get_product_in_warehouse(self.0.id, product_id)
                        .sync()
                        .map_err(into_graphql)
                        .map (|stock| {
                            if let Some(stock) = stock {
                                stock
                            } else {
                                Stock {
                                    id: StockId::new(),
                                    product_id,
                                    warehouse_id: self.0.id,
                                    quantity: Quantity::default(),
                                }
                            }
                        })
                        .map(GraphQLStock)
                }).collect::<FieldResult<Vec<GraphQLStock>>>()
                .and_then (|products| {
                    let mut product_edges = Edge::create_vec(products, offset);

                    let body = serde_json::to_string(&search_term)?;

                    let url = format!("{}/{}/search/filters/count",
                        context.config.service_url(Service::Stores),
                        Model::BaseProduct.to_url(),
                        );

                    let total_items = context.request::<i32>(Method::Post, url, Some(body))
                        .wait()?;
                    let total_pages = (total_items as f32 / items_count as f32).ceil() as i32;
                    let search_filters = ProductsSearchFilters::new(search_term);
                    let page_info = PageInfoWarehouseProductSearch {
                        total_pages,
                        current_page,
                        page_items_count: items_count,
                        search_term_options: Some(search_filters)
                    };
                    Ok(Connection::new(product_edges, page_info))
                })
            })
            .map(Some)
    }

    field auto_complete_product_name(&executor,
        first = None : Option<i32> as "First edges",
        after = None : Option<GraphqlID>  as "Offset form beginning",
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
            store_id : Some(self.0.store_id.0),
            status: None,
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

});

graphql_object!(Connection<GraphQLStock, PageInfoWarehouseProductSearch>: Context as "StocksConnection" |&self| {
    description:"Warehouse Products Connection"

    field edges() -> &[Edge<GraphQLStock>] {
        &self.edges
    }

    field page_info() -> &PageInfoWarehouseProductSearch {
        &self.page_info
    }
});

graphql_object!(Edge<GraphQLStock>: Context as "StocksEdge" |&self| {
    description:"Warehouse Product Edge"

    field cursor() -> &juniper::ID {
        &self.cursor
    }

    field node() -> &GraphQLStock {
        &self.node
    }
});

pub fn get_warehouses_for_store(context: &Context, store_id: StoreId) -> FieldResult<Vec<Warehouse>> {
    let url = format!(
        "{}/{}/by-store/{}",
        context.config.service_url(Service::Warehouses),
        Model::Warehouse.to_url(),
        store_id,
    );

    context.request::<Vec<Warehouse>>(Method::Get, url, None).wait()
}

pub fn try_get_warehouse(context: &Context, warehouse_id: WarehouseIdentifier) -> FieldResult<Option<GraphQLWarehouse>> {
    let warehouse_route = match warehouse_id {
        WarehouseIdentifier::Id(id) => format!("by-id/{}", id),
        WarehouseIdentifier::Slug(slug) => format!("by-slug/{}", slug),
    };

    let url = format!(
        "{}/{}/{}",
        context.config.service_url(Service::Warehouses),
        Model::Warehouse.to_url(),
        warehouse_route
    );

    context
        .request::<Option<Warehouse>>(Method::Get, url, None)
        .wait()
        .map(|res| res.map(GraphQLWarehouse))
}

pub fn get_warehouse(context: &Context, warehouse_id: WarehouseIdentifier) -> FieldResult<GraphQLWarehouse> {
    try_get_warehouse(context, warehouse_id.clone())?.ok_or_else(move || {
        let message = match warehouse_id {
            WarehouseIdentifier::Id(id) => format!("by id: {}", id),
            WarehouseIdentifier::Slug(slug) => format!("by slug: {}", slug),
        };

        FieldError::new(
            "Warehouse not found",
            graphql_value!({ "code": 400, "details": { format!("warehouse {} not found", message) }}),
        )
    })
}
