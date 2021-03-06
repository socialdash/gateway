//! File containing wizard store object of graphql schema
use chrono::prelude::*;
use futures::Future;
use hyper::Method;
use juniper::ID as GraphqlID;
use juniper::{FieldError, FieldResult};

use stq_routes::service::Service;
use stq_static_resources::{Currency, OrderState};
use stq_types::{OrderId, OrderIdentifier};

use graphql::context::Context;
use graphql::models::*;
use graphql::schema::order as order_module;

graphql_object!(Invoice: Context as "Invoice" |&self| {
    description: "Invoice info"

    field id() -> GraphqlID as "Base64 Unique id"{
        self.invoice_id.to_string().into()
    }

    field orders(&executor) -> FieldResult<Vec<GraphQLOrder>> as "Fetches Orders." {
        let context = executor.context();

        self.get_orders(context)
    }

    field amount() -> f64 as "amount"{
        let multiplier = 10i64.pow(8 as u32) as f64;
	    (self.amount.0 * multiplier).ceil() / multiplier
    }

    field currency() -> &Currency as "currency"{
        &self.currency
    }

    field price_reserved_due_date_time() -> String as "price reserved due to date time"{
        let datetime: DateTime<Utc> = self.price_reserved.into();
        datetime.to_rfc3339()
    }

    field state() -> &OrderState as "order state"{
        &self.state
    }

    field wallet() -> &Option<String> as "wallet"{
        &self.wallet
    }

    field transactions() -> &[Transaction] as "Transactions"{
        &self.transactions
    }

    field amount_captured() -> &f64 as "amount captured"{
        &self.amount_captured.0
    }

    field payment_intent(&executor) -> FieldResult<Option<PaymentIntent>> as "Stripe payment intent" {
        let context = executor.context();

        let billing = context.get_billing_microservice();
        billing.payment_intent_by_invoice(self.invoice_id)
    }

});

graphql_object!(Transaction: Context as "Transaction" |&self| {
    description: "Transaction info"

    field id() -> &str as "id"{
        &self.id
    }

    field amount() -> &f64 as "amount captured"{
        &self.amount_captured.0
    }

});

impl Invoice {
    fn get_orders(&self, context: &Context) -> FieldResult<Vec<GraphQLOrder>> {
        let url = format!(
            "{}/invoices/by-id/{}/order_ids",
            &context.config.service_url(Service::Billing),
            self.invoice_id.to_string()
        );

        context.request::<Vec<OrderId>>(Method::Get, url, None).wait().and_then(|ids| {
            ids.into_iter()
                .map(|id| {
                    order_module::try_get_order(context, OrderIdentifier::Id(id))
                    .and_then(|order| {
                        order.ok_or_else(|| {
                            FieldError::new(
                                "Could not find order id received from invoice in orders.",
                                graphql_value!({ "code": 100, "details": { format!("Order with id: {} does not exist in orders microservice.", id) }}),
                            )
                        })
                    })
                }).collect()
        })
    }
}
