use rust_decimal::Decimal;

#[derive(Debug)]
pub(crate) struct ArchivedDeposit {
    pub(crate) amount: Decimal,
    pub(crate) disputed: bool,
}

impl ArchivedDeposit {
    pub fn new(amount: Decimal) -> Self {
        ArchivedDeposit {
            amount,
            disputed: false,
        }
    }
}
