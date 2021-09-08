use rust_decimal::Decimal;
use serde::Serialize;

const CLIENT: Client = Client {
    held: rust_decimal::Decimal::ZERO,
    total: rust_decimal::Decimal::ZERO,
    locked: false,
};

#[derive(Debug)]
pub(crate) struct Client {
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
}

impl Client {
    pub fn new() -> Self {
        CLIENT
    }

    pub fn available(&self) -> Decimal {
        self.total - self.held
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ClientOutput {
    client: u16,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl ClientOutput {
    pub(crate) fn from_client(item: Client, id: u16) -> Self {
        ClientOutput {
            client: id,
            available: item.total - item.held,
            held: item.held,
            total: item.total,
            locked: item.locked,
        }
    }
}
