use std::time::SystemTime;

use chrono::prelude::*;
use futures::future;
use futures::prelude::*;
use hyper::header::{Authorization, Cookie, Headers};
use juniper;
use juniper::parser::SourcePosition;
use juniper::FieldError;
use juniper::FieldResult;
use serde::de::DeserializeOwned;
use serde::ser;
use serde::ser::SerializeMap;
use uuid::Uuid;

use config::Config;

use http::{
    header::{HeaderName, HeaderValue},
    HeaderMap,
};
use stq_api::rpc_client::RestApiClient;
use stq_http::client::{ClientHandle, Error, HttpClient, TimeLimitedHttpClient};
use stq_http::request_util::{CorrelationToken, Currency as CurrencyHeader, FiatCurrency as FiatCurrencyHeader};
use stq_routes::model::Model;
use stq_routes::service::Service;
use stq_static_resources::Currency;
use stq_types::{SessionId, StoresRole};

use graphql::models::jwt::JWTPayload;
use graphql::models::User;

use graphql::microservice::{
    BillingService, BillingServiceImpl, DeliveryService, DeliveryServiceImpl, OrdersService, OrdersServiceImpl, SagaService,
    SagaServiceImpl, StoresService, StoresServiceImpl,
};

pub struct Context {
    pub http_client: TimeLimitedHttpClient<ClientHandle>,
    pub user: Option<JWTPayload>,
    pub session_id: Option<SessionId>,
    pub currency: Option<Currency>,
    pub fiat_currency: Option<Currency>,
    pub correlation_token: Option<CorrelationToken>,
    pub uuid: String,
    pub config: Config,
}

pub struct Permissions<'r> {
    context: &'r Context,
}

impl juniper::Context for Context {}

impl Context {
    pub fn new(
        http_client: TimeLimitedHttpClient<ClientHandle>,
        user: Option<JWTPayload>,
        session_id: Option<SessionId>,
        currency: Option<Currency>,
        fiat_currency: Option<Currency>,
        config: Config,
        correlation_token: Option<CorrelationToken>,
    ) -> Self {
        let uuid = Uuid::new_v4().hyphenated().to_string();

        Self {
            http_client,
            user,
            session_id,
            currency,
            fiat_currency,
            uuid,
            config,
            correlation_token,
        }
    }

    pub fn get_rest_api_client(&self, s: Service) -> RestApiClient {
        let header_name = HeaderName::from_static("correlation-token");

        let headers = match self.correlation_token.clone() {
            Some(value) => vec![(header_name, HeaderValue::from_str(&value.0).unwrap())],
            None => vec![(header_name, HeaderValue::from_str(&self.uuid).unwrap())],
        }
        .into_iter()
        .collect::<HeaderMap>();

        RestApiClient::new_with_default_headers(&self.config.service_url(s), self.user.clone().map(|u| u.user_id), Some(headers))
    }

    pub fn request<T>(&self, method: hyper::Method, url: String, body: Option<String>) -> Box<Future<Item = T, Error = FieldError> + Send>
    where
        T: DeserializeOwned + 'static + Send,
    {
        let mut headers = Headers::new();
        if let Some(ref token_payload) = self.user {
            match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => {
                    if token_payload.exp < n.as_secs() as i64 {
                        let err = FieldError::new(
                            "JWT has been expired.",
                            graphql_value!({ "code": 111, "details": { "Current JWT in request header has been expired." }}),
                        );
                        return Box::new(future::err(err));
                    }
                }
                Err(_) => unreachable!(),
            };
            headers.set(Authorization(token_payload.to_string()));
        };
        let mut cookie = Cookie::new();
        cookie.append("UUID", self.uuid.clone());
        if let Some(ref session_id) = self.session_id {
            cookie.append("SESSION_ID", session_id.to_string());
        };
        if let Some(ref currency) = self.currency {
            headers.set(CurrencyHeader(currency.code().into()));
        };
        if let Some(ref fiat_currency) = self.fiat_currency {
            headers.set(FiatCurrencyHeader(fiat_currency.code().into()));
        }
        headers.set(cookie);

        self.set_correlation_token(&mut headers);

        let dt = Local::now();
        let correlation_token = self.correlation_token.clone().map(|token| token.0).unwrap_or(self.uuid.clone());

        Box::new(
            self.http_client
                .request_json(method, url.clone(), body, Some(headers))
                .map_err(Error::into_graphql)
                .then(move |r| {
                    let d = Local::now() - dt;
                    match r {
                        Err(e) => {
                            info!(
                                "Request to microservice: {:?} failed with error `{:?}`, elapsed time: {}.{:03}, correlation token: {}",
                                url,
                                e,
                                d.num_seconds(),
                                d.num_milliseconds(),
                                correlation_token,
                            );
                            Err(e)
                        }
                        Ok(x) => {
                            info!(
                                "Request to microservice: {:?}, elapsed time: {}.{:03}, correlation token: {}",
                                url,
                                d.num_seconds(),
                                d.num_milliseconds(),
                                correlation_token,
                            );
                            Ok(x)
                        }
                    }
                }),
        )
    }

    pub fn request_without_auth<T>(
        &self,
        method: hyper::Method,
        url: String,
        body: Option<String>,
    ) -> Box<Future<Item = T, Error = FieldError> + Send>
    where
        T: DeserializeOwned + 'static + Send,
    {
        let mut headers = Headers::new();
        self.set_correlation_token(&mut headers);
        let dt = Local::now();
        let correlation_token = self.correlation_token.clone().map(|token| token.0).unwrap_or(self.uuid.clone());

        Box::new(
            self.http_client
                .request_json(method, url.clone(), body, Some(headers))
                .map_err(Error::into_graphql)
                .then(move |r| {
                    let d = Local::now() - dt;
                    match r {
                        Err(e) => {
                            info!(
                                "Request to microservice: {:?} failed with error `{:?}`, elapsed time: {}.{:03}, correlation token: {}",
                                url,
                                e,
                                d.num_seconds(),
                                d.num_milliseconds(),
                                correlation_token,
                            );
                            Err(e)
                        }
                        Ok(x) => {
                            info!(
                                "Request to microservice: {:?}, elapsed time: {}.{:03}, correlation token: {}",
                                url,
                                d.num_seconds(),
                                d.num_milliseconds(),
                                correlation_token,
                            );
                            Ok(x)
                        }
                    }
                }),
        )
    }

    fn set_correlation_token(&self, headers: &mut hyper::Headers) {
        match self.correlation_token.as_ref() {
            Some(value) => headers.set(value.clone()),
            None => headers.set(CorrelationToken(self.uuid.clone())),
        }
    }

    pub fn get_saga_microservice<'r>(&'r self) -> Box<dyn SagaService + 'r> {
        Box::new(SagaServiceImpl::new(self))
    }

    pub fn get_billing_microservice<'r>(&'r self) -> Box<dyn BillingService + 'r> {
        Box::new(BillingServiceImpl::new(self))
    }

    pub fn get_delivery_microservice<'r>(&'r self) -> Box<dyn DeliveryService + 'r> {
        Box::new(DeliveryServiceImpl::new(self))
    }

    pub fn get_stores_microservice<'r>(&'r self) -> Box<dyn StoresService + 'r> {
        Box::new(StoresServiceImpl::new(self))
    }

    pub fn get_orders_microservice<'r>(&'r self) -> Box<dyn OrdersService + 'r> {
        Box::new(OrdersServiceImpl::new(self))
    }

    pub fn permissions(&self) -> Permissions {
        Permissions { context: &self }
    }
}

impl<'r> Permissions<'r> {
    pub fn store_roles(&self) -> FieldResult<Vec<StoresRole>> {
        if let Some(current_user_id) = self.context.user.as_ref().map(|jwt| jwt.user_id) {
            return self.context.get_stores_microservice().roles(current_user_id);
        }
        Ok(Vec::new())
    }
}

pub fn check_jwt_not_revoked(
    http_client: &TimeLimitedHttpClient<ClientHandle>,
    jwt_payload: &JWTPayload,
    users_url: String,
) -> Result<(), FieldError> {
    let mut headers = Headers::new();
    headers.set(Authorization(jwt_payload.to_string()));
    let url = format!("{}/{}/current", users_url, Model::User.to_url());
    let user = http_client
        .request_json::<Option<User>>(hyper::Method::Get, url, None, Some(headers))
        .map_err(Error::into_graphql)
        .wait()?;
    if let Some(user) = user {
        //jwt exp must be greater or equal than revoke timestamp
        match user.revoke_before.duration_since(SystemTime::UNIX_EPOCH) {
            Ok(n) => {
                if jwt_payload.exp >= n.as_secs() as i64 {
                    Ok(())
                } else {
                    Err(FieldError::new(
                        "JWT has been revoked.",
                        graphql_value!({ "code": 112, "details": { "Current JWT can not be used anymore." }}),
                    ))
                }
            }
            Err(_) => unreachable!(),
        }
    } else {
        Err(FieldError::new(
            "Could not get user info by jwt token.",
            graphql_value!({ "code": 100, "details": { "User not found." }}),
        ))
    }
}

pub struct GraphQLResponse(juniper::Value, Vec<juniper::ExecutionError>);

impl GraphQLResponse {
    pub fn from_field_error(err: FieldError) -> GraphQLResponse {
        let exec_err = juniper::ExecutionError::new(SourcePosition::new_origin(), &vec![], err);
        GraphQLResponse(juniper::Value::Null, vec![exec_err])
    }
}

impl<'a> ser::Serialize for GraphQLResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        let mut map = try!(serializer.serialize_map(None));

        try!(map.serialize_key("data"));
        try!(map.serialize_value(&self.0));

        if !self.1.is_empty() {
            try!(map.serialize_key("errors"));
            try!(map.serialize_value(&self.1));
        }

        map.end()
    }
}
