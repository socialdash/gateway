use std::time::SystemTime;

use stq_static_resources::{Currency, OrderState};
use stq_types::{InvoiceId, ProductPrice};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Invoice {
    pub invoice_id: InvoiceId,
    pub transactions: Vec<Transaction>,
    pub amount: ProductPrice,
    pub currency: Currency,
    pub price_reserved: SystemTime,
    pub state: OrderState,
    pub wallet: Option<String>,
    pub amount_captured: ProductPrice,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub amount_captured: ProductPrice,
}
