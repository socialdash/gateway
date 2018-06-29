use super::*;
use juniper::ID as GraphqlID;

#[derive(GraphQLEnum, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[graphql(description = "Warehouse kind")]
pub enum WarehouseKind {
    #[serde(rename = "distribution_center")]
    #[graphql(description = "DistributionCenter.")]
    DistributionCenter,
    #[serde(rename = "store")]
    #[graphql(description = "Store.")]
    Store,
}

#[derive(GraphQLObject, Serialize, Deserialize, Debug, Clone)]
#[graphql(description = "Geo point")]
pub struct GeoPoint {
    #[graphql(description = "x.")]
    pub x: f64,
    #[graphql(description = "y.")]
    pub y: f64,
}

#[derive(GraphQLInputObject, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[graphql(description = "Geo point")]
pub struct GeoPointInput {
    #[graphql(description = "x.")]
    pub x: f64,
    #[graphql(description = "y.")]
    pub y: f64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Warehouse {
    pub id: String,
    pub name: Option<String>,
    pub store_id: i32,
    pub location: Option<GeoPoint>,
    pub administrative_area_level_1: Option<String>,
    pub administrative_area_level_2: Option<String>,
    pub address: Option<String>,
    pub country: Option<String>,
    pub locality: Option<String>,
    pub political: Option<String>,
    pub postal_code: Option<String>,
    pub route: Option<String>,
    pub street_number: Option<String>,
    pub place_id: Option<String>,
    pub slug: String,
}

#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct ValueContainer<T> {
    pub value: Option<T>,
}

impl<T> From<T> for ValueContainer<T> {
    fn from(value: T) -> Self {
        Self { value: Some(value) }
    }
}

#[derive(GraphQLInputObject, Serialize, Debug, Clone, PartialEq)]
#[graphql(description = "Update warehouse input object")]
pub struct UpdateWarehouseInput {
    #[graphql(description = "Client mutation id.")]
    #[serde(skip_serializing)]
    pub client_mutation_id: String,
    #[graphql(description = "Id of a warehouse.")]
    #[serde(skip_serializing)]
    pub id: GraphqlID,
    #[graphql(description = "Name of a warehouse.")]
    pub name: Option<String>,
    #[graphql(description = "Location of a warehouse.")]
    pub location: Option<GeoPointInput>,
    #[graphql(description = "Address")]
    #[serde(flatten)]
    pub address_full: AddressInput,
    #[graphql(description = "Slug of a warehouse.")]
    pub slug: Option<String>,
}

impl UpdateWarehouseInput {
    pub fn is_none(&self) -> bool {
        Self {
            client_mutation_id: self.client_mutation_id.clone(),
            id: self.id.clone(),
            name: None,
            location: None,
            slug: None,
            address_full: AddressInput {
                country: None,
                administrative_area_level_1: None,
                administrative_area_level_2: None,
                locality: None,
                political: None,
                postal_code: None,
                route: None,
                street_number: None,
                value: None,
                place_id: None,
            },
        } == self.clone()
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct UpdateWarehouse {
    pub slug: Option<ValueContainer<String>>,
    pub name: Option<ValueContainer<String>>,
    pub location: Option<ValueContainer<GeoPointInput>>,
    pub administrative_area_level_1: Option<ValueContainer<String>>,
    pub administrative_area_level_2: Option<ValueContainer<String>>,
    pub country: Option<ValueContainer<String>>,
    pub locality: Option<ValueContainer<String>>,
    pub political: Option<ValueContainer<String>>,
    pub postal_code: Option<ValueContainer<String>>,
    pub route: Option<ValueContainer<String>>,
    pub street_number: Option<ValueContainer<String>>,
    pub address: Option<ValueContainer<String>>,
    pub place_id: Option<ValueContainer<String>>,
}

impl From<UpdateWarehouseInput> for UpdateWarehouse {
    fn from(value: UpdateWarehouseInput) -> Self {
        Self {
            slug: value.slug.map(From::from),
            name: value.name.map(From::from),
            location: value.location.map(From::from),
            administrative_area_level_1: value.address_full.administrative_area_level_1.map(From::from),
            administrative_area_level_2: value.address_full.administrative_area_level_2.map(From::from),
            country: value.address_full.country.map(From::from),
            locality: value.address_full.locality.map(From::from),
            political: value.address_full.political.map(From::from),
            postal_code: value.address_full.postal_code.map(From::from),
            route: value.address_full.route.map(From::from),
            street_number: value.address_full.street_number.map(From::from),
            address: value.address_full.value.map(From::from),
            place_id: value.address_full.place_id.map(From::from),
        }
    }
}

#[derive(GraphQLInputObject, Serialize, Debug, Clone)]
#[graphql(description = "Create warehouse input object")]
pub struct CreateWarehouseInput {
    #[graphql(description = "Client mutation id.")]
    #[serde(skip_serializing)]
    pub client_mutation_id: String,
    #[graphql(description = "Name of a warehouse.")]
    pub name: Option<String>,
    #[graphql(description = "Slug of a warehouse.")]
    pub slug: Option<String>,
    #[graphql(description = "Store id of a warehouse.")]
    pub store_id: i32,
    #[graphql(description = "Location of a warehouse.")]
    pub location: Option<GeoPointInput>,
    #[graphql(description = "Address")]
    #[serde(flatten)]
    pub address_full: AddressInput,
}

#[derive(Clone, Debug)]
pub struct PageInfoWarehouseProductSearch {
    pub total_pages: i32,
    pub current_page: i32,
    pub page_items_count: i32,
    pub search_term_options: Option<ProductsSearchFilters>,
}