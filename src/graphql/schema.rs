use std::str::FromStr;
use std::cmp;


use juniper;
use juniper::FieldResult;
use super::context::Context;
use super::models::*;
use hyper::{Method, StatusCode};
use futures::Future;
use juniper::ID as GraphqlID;
use serde_json;

use ::http::client::{Error, ErrorMessage};


pub struct Viewer;
pub struct StaticNodeIds;
pub struct Query;
pub struct Mutation;
pub enum Node {
    User(User),
    Store(Store),
    Product(Product),
    Viewer(Viewer),
    Query(Query)
}

pub type Schema = juniper::RootNode<'static, Query, Mutation>;

const MIN_ID: i32 = 0; 
const VIEWER_NODE_ID: i32 = 0;
const QUERY_NODE_ID: i32 = 1;


pub fn create() -> Schema {
    let query = Query {};
    let mutation = Mutation {};
    Schema::new(query, mutation)
}

graphql_interface!(Node: Context as "Node" |&self| {
    description: "The Node interface contains a single field, 
        id, which is a ID!. The node root field takes a single argument, 
        a ID!, and returns a Node. These two work in concert to allow refetching."
    
    field id() -> GraphqlID {
        match *self {
            Node::User(User { ref id, .. })  => ID::new(Service::Users, Model::User, *id).to_string().into(),
            Node::Store(Store { ref id, .. })  => ID::new(Service::Stores, Model::Store, *id).to_string().into(),
            Node::Product(Product { ref id, .. })  => ID::new(Service::Stores, Model::Product, *id).to_string().into(),
            Node::Viewer(_)  => VIEWER_NODE_ID.to_string().into(),
            Node::Query(_)  => QUERY_NODE_ID.to_string().into(),
        }
    }

    instance_resolvers: |_| {
        &User => match *self { Node::User(ref h) => Some(h), _ => None },
        &Store => match *self { Node::Store(ref h) => Some(h), _ => None },
        &Product => match *self { Node::Product(ref h) => Some(h), _ => None },
        &Viewer => match *self { Node::Viewer(ref h) => Some(h), _ => None },
        &Query => match *self { Node::Query(ref h) => Some(h), _ => None },
    }
});

graphql_object!(User: Context as "User" |&self| {
    description: "User's profile."

    interfaces: [&Node]

    field id() -> GraphqlID as "Unique id"{
        ID::new(Service::Users, Model::User, self.id).to_string().into()
    }

    field raw_id() -> GraphqlID as "Unique id"{
        self.id.to_string().into()
    }

    field email() -> String as "Email" {
        self.email.clone()
    }

    field phone() -> Option<String> as "Phone" {
        self.phone.clone()
    }

    field first_name() -> Option<String> as "First name" {
        self.first_name.clone()
    }

    field last_name() -> Option<String> as "Last name" {
        self.last_name.clone()
    }

    field middle_name() -> Option<String> as "Middle name" {
        self.middle_name.clone()
    }

    field gender() -> Gender as "Gender" {
        self.gender.clone()
    }

    field birthdate() -> Option<String> as "Birthdate" {
        self.birthdate.clone()
    }

    field isActive() -> bool as "If the user was disabled (deleted), isActive is false" {
        self.is_active
    }

});

graphql_object!(Store: Context as "Store" |&self| {
    description: "Store's profile."

    interfaces: [&Node]

    field id() -> GraphqlID as "Unique id"{
        ID::new(Service::Stores, Model::Store, self.id).to_string().into()
    }

    field raw_id() -> GraphqlID as "Unique id"{
        self.id.to_string().into()
    }

    field name() -> String as "Full Name" {
        self.name.clone()
    }

    field isActive() -> bool as "If the store was disabled (deleted), isActive is false" {
        self.is_active
    }

});

graphql_object!(Product: Context as "Product" |&self| {
    description: "Product's profile."

    interfaces: [&Node]

    field id() -> GraphqlID as "Unique id"{
        ID::new(Service::Stores, Model::Product, self.id).to_string().into()
    }

    field raw_id() -> GraphqlID as "Unique id"{
        self.id.to_string().into()
    }

    field name() -> String as "Full Name" {
        self.name.clone()
    }

    field isActive() -> bool as "If the product was disabled (deleted), isActive is false" {
        self.is_active
    }

});


graphql_object!(Connection<User>: Context as "UsersConnection" |&self| {
    description:"Users Connection"

    field edges() -> Vec<Edge<User>> {
        self.edges.to_vec()
    }

    field page_info() -> PageInfo {
        self.page_info.clone()
    }
});

graphql_object!(Edge<User>: Context as "UsersEdge" |&self| {
    description:"Users Edge"
    
    field cursor() -> juniper::ID {
        self.cursor.clone()
    }

    field node() -> User {
        self.node.clone()
    }
});

graphql_object!(Connection<Store>: Context as "StoresConnection" |&self| {
    description:"Stores Connection"

    field edges() -> Vec<Edge<Store>> {
        self.edges.to_vec()
    }

    field page_info() -> PageInfo {
        self.page_info.clone()
    }
});

graphql_object!(Edge<Store>: Context as "StoresEdge" |&self| {
    description:"Stores Edge"
    
    field cursor() -> juniper::ID {
        self.cursor.clone()
    }

    field node() -> Store {
        self.node.clone()
    }
});


graphql_object!(Connection<Product>: Context as "ProductsConnection" |&self| {
    description:"Products Connection"

    field edges() -> Vec<Edge<Product>> {
        self.edges.to_vec()
    }

    field page_info() -> PageInfo {
        self.page_info.clone()
    }
});

graphql_object!(Edge<Product>: Context as "ProductsEdge" |&self| {
    description:"Products Edge"
    
    field cursor() -> juniper::ID {
        self.cursor.clone()
    }

    field node() -> Product {
        self.node.clone()
    }
});

graphql_object!(Viewer: Context as "Viewer" |&self| {
    description: "Viewer for users, stores, products.
    To access users data one must receive viewer object, 
    by passing jwt in bearer authentification header of http request.
    All requests without it or with wrong jwt will recieve null."

    interfaces: [&Node]

    field id() -> GraphqlID as "Unique id"{
        VIEWER_NODE_ID.to_string().into()
    }

    field user(&executor, id: GraphqlID as "Id of a user.") -> FieldResult<User> as "Fetches user by id." {
        let context = executor.context();
        let user = context.user.clone().ok_or("Authentification of Json web token failure")?;

        let identifier = ID::from_str(&*id)?;
        let url = identifier.url(&context.config);

        context.http_client.request_with_auth_header::<User>(Method::Get, url, None, user)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field users(&executor, first = None : Option<i32> as "First edges", after = None : Option<GraphqlID>  as "Id of a user") -> FieldResult<Connection<User>> as "Fetches users using relay connection." {
        let context = executor.context();
        let user = context.user.clone().ok_or("Authentification of Json web token failure")?;
        
        let raw_id = match after {
            Some(val) => ID::from_str(&*val)?.raw_id,
            None => MIN_ID
        };
        
        let records_limit = context.config.gateway.records_limit;
        let first = cmp::min(first.unwrap_or(records_limit as i32), records_limit as i32);

        let url = format!("{}/{}/?from={}&count={}",
            Service::Users.to_url(&context.config), 
            Model::User.to_url(),
            raw_id,
            first + 1);

        context.http_client.request_with_auth_header::<Vec<User>>(Method::Get, url, None, user)
            .or_else(|err| Err(err.to_graphql()))
            .map (|users| {
                let mut user_edges: Vec<Edge<User>> = users
                    .into_iter()
                    .map(|user| Edge::new(
                                juniper::ID::from(ID::new(Service::Users, Model::User, user.id.clone()).to_string()),
                                user.clone()
                            ))
                    .collect();
                let has_next_page = user_edges.len() as i32 == first + 1;
                if has_next_page { 
                    user_edges.pop(); 
                };
                let has_previous_page = true;
                let page_info = PageInfo {has_next_page: has_next_page, has_previous_page: has_previous_page};
                Connection::new(user_edges, page_info)
            })
            .wait()
    }

    field store(&executor, id: GraphqlID as "Id of a store.") -> FieldResult<Store> as "Fetches store by id." {
        let context = executor.context();
        let user = context.user.clone().ok_or("Authentification of Json web token failure")?;

        let identifier = ID::from_str(&*id)?;
        let url = identifier.url(&context.config);

        context.http_client.request_with_auth_header::<Store>(Method::Get, url, None, user)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field stores(&executor, first = None : Option<i32> as "First edges", after = None : Option<GraphqlID>  as "Id of a store") -> FieldResult<Connection<Store>> as "Fetches stores using relay connection." {
        let context = executor.context();
        let user = context.user.clone().ok_or("Authentification of Json web token failure")?;
        
        let raw_id = match after {
            Some(val) => ID::from_str(&*val)?.raw_id,
            None => MIN_ID
        };
        
        let records_limit = context.config.gateway.records_limit;
        let first = cmp::min(first.unwrap_or(records_limit as i32), records_limit as i32);

        let url = format!("{}/{}/?from={}&count={}",
            Service::Stores.to_url(&context.config), 
            Model::Store.to_url(),
            raw_id,
            first + 1);

        context.http_client.request_with_auth_header::<Vec<Store>>(Method::Get, url, None, user)
            .or_else(|err| Err(err.to_graphql()))
            .map (|stores| {
                let mut store_edges: Vec<Edge<Store>> = stores
                    .into_iter()
                    .map(|store| Edge::new(
                                juniper::ID::from(ID::new(Service::Stores, Model::Store, store.id.clone()).to_string()),
                                store.clone()
                            ))
                    .collect();
                let has_next_page = store_edges.len() as i32 == first + 1;
                if has_next_page { 
                    store_edges.pop(); 
                };
                let has_previous_page = true;
                let page_info = PageInfo {has_next_page: has_next_page, has_previous_page: has_previous_page};
                Connection::new(store_edges, page_info)
            })
            .wait()
    }

    field product(&executor, id: GraphqlID as "Id of a product.") -> FieldResult<Product> as "Fetches product by id." {
        let context = executor.context();
        let user = context.user.clone().ok_or("Authentification of Json web token failure")?;

        let identifier = ID::from_str(&*id)?;
        let url = identifier.url(&context.config);

        context.http_client.request_with_auth_header::<Product>(Method::Get, url, None, user)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field products(&executor, first = None : Option<i32> as "First edges", after = None : Option<GraphqlID>  as "Id of a product") -> FieldResult<Connection<Product>> as "Fetches products using relay connection." {
        let context = executor.context();
        let user = context.user.clone().ok_or("Authentification of Json web token failure")?;
        
        let raw_id = match after {
            Some(val) => ID::from_str(&*val)?.raw_id,
            None => MIN_ID
        };
        
        let records_limit = context.config.gateway.records_limit;
        let first = cmp::min(first.unwrap_or(records_limit as i32), records_limit as i32);

        let url = format!("{}/{}/?from={}&count={}",
            Service::Stores.to_url(&context.config), 
            Model::Product.to_url(),
            raw_id,
            first + 1);

        context.http_client.request_with_auth_header::<Vec<Product>>(Method::Get, url, None, user)
            .or_else(|err| Err(err.to_graphql()))
            .map (|products| {
                let mut product_edges: Vec<Edge<Product>> = products
                    .into_iter()
                    .map(|product| Edge::new(
                                juniper::ID::from(ID::new(Service::Stores, Model::Product, product.id.clone()).to_string()),
                                product.clone()
                            ))
                    .collect();
                let has_next_page = product_edges.len() as i32 == first + 1;
                if has_next_page { 
                    product_edges.pop(); 
                };
                let has_previous_page = true;
                let page_info = PageInfo {has_next_page: has_next_page, has_previous_page: has_previous_page};
                Connection::new(product_edges, page_info)
            })
            .wait()
    }

    field current_user(&executor) -> FieldResult<User> as "Fetches current user information." {
        let context = executor.context();
        let user = context.user.clone().ok_or("Authentification of Json web token failure")?;
        let url = format!("{}/{}/current",
            Service::Users.to_url(&context.config), 
            Model::User.to_url());
        context.http_client.request_with_auth_header::<User>(Method::Get, url, None, user)
                    .or_else(|err| Err(err.to_graphql()))
                    .wait()
                            
    }
});

graphql_object!(StaticNodeIds: Context as "StaticNodeIds" |&self| {
    field viewer_id(&executor) -> FieldResult<i32> as "Static viewer id." {
        Ok(VIEWER_NODE_ID)
    }

    field query_id(&executor) -> FieldResult<i32> as "Static query id." {
        Ok(QUERY_NODE_ID)
    }
});

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

    field id() -> GraphqlID as "Unique id"{
        QUERY_NODE_ID.to_string().into()
    }

    field apiVersion() -> &str as "Current api version." {
        "1.0"
    }

    field static_node_id() -> FieldResult<StaticNodeIds> as "Static node id dictionary." {
        Ok(StaticNodeIds{})
    }

    field viewer(&executor) -> FieldResult<Option<Viewer>> as "Fetches viewer for users." {
        let context = executor.context();

        match context.user {
            Some(_) => return Ok(Some(Viewer{})),
            None => return Err (
                Error::Api( 
                    StatusCode::Unauthorized, 
                    Some(ErrorMessage { code: 401, message: "Authentification of Json web token failure".to_string() })
                    )
                .to_graphql())
        }
    }

    field node(&executor, id: GraphqlID as "Id of a node.") -> FieldResult<Node> as "Fetches graphql interface node by id."  {
        let context = executor.context();
        if id.to_string() == VIEWER_NODE_ID.to_string() {
             match context.user {
                Some(_) => return Ok(Node::Viewer(Viewer{})),
                None => return Err (
                    Error::Api( 
                        StatusCode::Unauthorized, 
                        Some(ErrorMessage { code: 401, message: "Authentification of Json web token failure".to_string() })
                        )
                    .to_graphql())
            }
        } else if id.to_string() == QUERY_NODE_ID.to_string() {
             Ok(Node::Query(Query{}))
        } else {
            let identifier = ID::from_str(&*id)?;
            match (&identifier.service, &identifier.model) {
                (&Service::Users, _) => {
                                context.http_client.request::<User>(Method::Get, identifier.url(&context.config), None, None)
                                    .map(|res| Node::User(res))
                                    .or_else(|err| Err(err.to_graphql()))
                                    .wait()
                },
                (&Service::Stores, &Model::Store) => {
                                context.http_client.request::<Store>(Method::Get, identifier.url(&context.config), None, None)
                                    .map(|res| Node::Store(res))
                                    .or_else(|err| Err(err.to_graphql()))
                                    .wait()
                },
                (&Service::Stores, &Model::Product) => {
                                context.http_client.request::<Product>(Method::Get, identifier.url(&context.config), None, None)
                                    .map(|res| Node::Product(res))
                                    .or_else(|err| Err(err.to_graphql()))
                                    .wait()
                },
                (&Service::Stores, _) => {
                                context.http_client.request::<Store>(Method::Get, identifier.url(&context.config), None, None)
                                    .map(|res| Node::Store(res))
                                    .or_else(|err| Err(err.to_graphql()))
                                    .wait()
                }
            }
        }
    }
});

graphql_object!(Mutation: Context |&self| {
     
    description: "Top level mutation.

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

    field createUser(&executor, email: String as "Email of a user.", password: String as "Password of a user.") -> FieldResult<User> as "Creates new user." {
        let context = executor.context();
        let url = format!("{}/{}", 
            Service::Users.to_url(&context.config),
            Model::User.to_url());
        let user = json!({"email": email, "password": password});
        let body: String = user.to_string();

        context.http_client.request::<User>(Method::Post, url, Some(body), None)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field updateUser(
        &executor, 
        id: GraphqlID as "Id of a user.", 
        is_active: Option<bool> as "Activate/deactivate user.", 
        phone = None : Option<String> as "New phone of a user",
        first_name = None : Option<String> as "New first name of a user",
        last_name = None : Option<String> as "New last name of a user",
        middle_name = None : Option<String> as "New middle name of a user",
        gender = None : Option<Gender> as "gender of a user",
        birthdate = None : Option<String> as "Birthdate of a user") 
            -> FieldResult<User>  as "Updates existing user."{

        let context = executor.context();
        let identifier = ID::from_str(&*id)?;
        let url = identifier.url(&context.config);
        let user = UpdateUser {
                is_active,
                phone, 
                first_name, 
                last_name, 
                middle_name, 
                gender, 
                birthdate, 
            };
        let body: String = serde_json::to_string(&user)?.to_string();

        context.http_client.request::<User>(Method::Put, url, Some(body), None)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field deactivateUser(&executor, id: GraphqlID as "Id of a user.") -> FieldResult<User>  as "Deactivates existing user." {
        let context = executor.context();
        let identifier = ID::from_str(&*id)?;
        let url = identifier.url(&context.config);

        context.http_client.request::<User>(Method::Delete, url, None, None)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field createStore(
        &executor, 
        name: String as "Full name of a store.",
        user_id : i32 as "User id",
        currency_id : i32 as "Default currency id",
        short_description : String as "short_description",
        long_description = None : Option<String> as "long_description",
        slug : String as "slug",
        cover = None : Option<String> as "cover",
        logo = None : Option<String> as "logo",
        phone : String as "phone",
        email : String as "email",
        address : String as "address",
        facebook_url = None : Option<String> as "facebook_url",
        twitter_url = None : Option<String> as "twitter_url",
        instagram_url = None : Option<String> as "instagram_url") 
            -> FieldResult<Store> as "Creates new store." {

        let context = executor.context();
        let url = format!("{}/{}", 
            Service::Stores.to_url(&context.config),
            Model::Store.to_url());
        let store = NewStore {
            name,
            user_id,
            currency_id,
            short_description,
            long_description,
            slug,
            cover,
            logo,
            phone,
            email,
            address,
            facebook_url,
            twitter_url,
            instagram_url,
        };
        let body: String = serde_json::to_string(&store)?.to_string();

        context.http_client.request::<Store>(Method::Post, url, Some(body), None)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field updateStore(
        &executor, 
        id: GraphqlID as "Id of a store.", 
        name = None : Option<String> as "New name of a store.",
        currency_id = None : Option<i32> as "currency_id",
        short_description = None : Option<String> as "short_description",
        long_description = None : Option<String> as "long_description",
        slug = None : Option<String> as "slug",
        cover = None : Option<String> as "cover",
        logo = None : Option<String> as "logo",
        phone = None : Option<String> as "phone",
        email = None : Option<String> as "email",
        address = None : Option<String> as "address",
        facebook_url = None : Option<String> as "facebook_url",
        twitter_url = None : Option<String> as "twitter_url",
        instagram_url = None : Option<String> as "instagram_url")
            -> FieldResult<Store>  as "Updates existing store."{

        let context = executor.context();
        let identifier = ID::from_str(&*id)?;
        let url = identifier.url(&context.config);

        let store = UpdateStore {
            name,
            currency_id,
            short_description,
            long_description,
            slug,
            cover,
            logo,
            phone,
            email,
            address,
            facebook_url,
            twitter_url,
            instagram_url,
        };
        let body: String = serde_json::to_string(&store)?.to_string();

        context.http_client.request::<Store>(Method::Put, url, Some(body), None)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field deactivateStore(&executor, id: GraphqlID as "Id of a store.") -> FieldResult<Store>  as "Deactivates existing store." {
        let context = executor.context();
        let identifier = ID::from_str(&*id)?;
        let url = identifier.url(&context.config);

        context.http_client.request::<Store>(Method::Delete, url, None, None)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field createProduct(
        &executor, 
        name: String as "Full name of a product.",
        store_id: i32 as "store_id",
        currency_id: i32 as "currency_id",
        short_description: String as "short_description",
        long_description = None : Option<String> as "long_description",
        price: f64 as "price",
        discount = None : Option<f64> as "discount",
        category = None : Option<i32> as "category",
        photo_main = None : Option<String> as "photo_main") 
            -> FieldResult<Product> as "Creates new product." {

        let context = executor.context();
        let url = format!("{}/{}", 
            Service::Stores.to_url(&context.config),
            Model::Product.to_url());

        let product = NewProduct {
            name,
            store_id,
            currency_id,
            short_description,
            long_description,
            price,
            discount,
            category,
            photo_main,
        };
        let body: String = serde_json::to_string(&product)?.to_string();

        context.http_client.request::<Product>(Method::Post, url, Some(body), None)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field updateProduct(
        &executor, 
        id: GraphqlID as "Id of a product.", 
        name = None : Option<String> as "New name of a product.",
        currency_id = None : Option<i32> as "currency_id",
        short_description = None : Option<String> as "short_description",
        long_description = None : Option<String> as "long_description",
        price = None : Option<f64> as "price",
        discount = None : Option<f64> as "discount",
        category = None : Option<i32> as "category",
        photo_main = None : Option<String> as "photo_main") 
            -> FieldResult<Product>  as "Updates existing product."{

        let context = executor.context();
        let identifier = ID::from_str(&*id)?;
        let url = identifier.url(&context.config);
        
        let product = UpdateProduct {
            name,
            currency_id,
            short_description,
            long_description,
            price,
            discount,
            category,
            photo_main
        };
        let body: String = serde_json::to_string(&product)?.to_string();

        context.http_client.request::<Product>(Method::Put, url, Some(body), None)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field deactivateProduct(&executor, id: GraphqlID as "Id of a product.") -> FieldResult<Product>  as "Deactivates existing product." {
        let context = executor.context();
        let identifier = ID::from_str(&*id)?;
        let url = identifier.url(&context.config);

        context.http_client.request::<Product>(Method::Delete, url, None, None)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field getJWTByEmail(&executor, email: String as "Email of a user.", password: String as "Password of a user") -> FieldResult<JWT> as "Get JWT Token by email." {
        let context = executor.context();
        let url = format!("{}/{}/email", 
            Service::Users.to_url(&context.config),
            Model::JWT.to_url());
        let account = json!({"email": email, "password": password});
        let body: String = account.to_string();

        context.http_client.request::<JWT>(Method::Post, url, Some(body), None)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

    field getJWTByProvider(&executor, provider: Provider as "Token provider", token: String as "Token.") -> FieldResult<JWT> as "Get JWT Token from token provider." {
        let context = executor.context();
        let url = format!("{}/{}/{}", 
            Service::Users.to_url(&context.config), 
            Model::JWT.to_url(),
            provider);
        let oauth = ProviderOauth { token: token };
        let body: String = serde_json::to_string(&oauth)?;

        context.http_client.request::<JWT>(Method::Post, url, Some(body), None)
            .or_else(|err| Err(err.to_graphql()))
            .wait()
    }

});
