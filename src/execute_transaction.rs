use crate::{
    transaction::TransactionType, ArchivedDeposit, Client, Transaction, TransactionsDataStructure,
};

pub(crate) fn execute_transaction(
    transaction: &Transaction,
    client: &mut Client,
    archived_deposits: &mut TransactionsDataStructure,
) {
    match transaction.r#type {
        TransactionType::Deposit => {
            let amount = transaction.amount.unwrap();
            if amount.is_sign_positive() {
                client.total += amount;
                archived_deposits.insert(transaction.tx_id, ArchivedDeposit::new(amount));
            }
        }
        TransactionType::Withdrawal => {
            let amount = transaction.amount.unwrap();
            if amount.is_sign_positive() && client.available() >= amount {
                client.total -= amount;
            }
        }
        TransactionType::Dispute => {
            if let Some(referenced_transaction) = archived_deposits.get_mut(&transaction.tx_id) {
                // don't allow two disputes, otherwise the held amount would be too high
                if !referenced_transaction.disputed {
                    client.held += referenced_transaction.amount;
                    referenced_transaction.disputed = true;
                }
            }
        }
        TransactionType::Resolve => {
            if let Some(referenced_transaction) = archived_deposits.get_mut(&transaction.tx_id) {
                if referenced_transaction.disputed {
                    client.held -= referenced_transaction.amount;
                    referenced_transaction.disputed = false;
                }
            }
        }
        TransactionType::Chargeback => {
            if let Some(referenced_transaction) = archived_deposits.get(&transaction.tx_id) {
                if referenced_transaction.disputed {
                    client.held -= referenced_transaction.amount;
                    client.total -= referenced_transaction.amount;
                    client.locked = true;
                    // Remove transaction otherwise it could be resolved again even though funds were returned
                    archived_deposits.remove(&transaction.tx_id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::execute_transaction::execute_transaction;
    use crate::{
        transaction::TransactionType, ArchivedDeposit, Client, Transaction,
        TransactionsDataStructure,
    };
    use rust_decimal::Decimal;
    use std::ops::{Add, Neg, Sub};

    impl Client {
        pub fn assert_total(&self, value: Decimal) -> &Client {
            assert_eq!(self.total, value);
            self
        }

        pub fn assert_held(&self, value: Decimal) -> &Client {
            assert_eq!(self.held, value);
            self
        }
        pub fn assert_available(&self, value: Decimal) -> &Client {
            assert_eq!(self.available(), value);
            self
        }

        pub fn assert_frozen(&self, locked: bool) -> &Client {
            assert_eq!(self.locked, locked);
            self
        }
    }

    trait ArchiveTransactionAssertions {
        fn assert_amount(self, tx: u32, amount: Decimal) -> TransactionsDataStructure;
        fn assert_disputed(self, tx: u32, disputed: bool) -> TransactionsDataStructure;
        fn assert_removed(self, tx: u32) -> TransactionsDataStructure;
    }

    impl ArchiveTransactionAssertions for TransactionsDataStructure {
        fn assert_amount(self, tx: u32, amount: Decimal) -> TransactionsDataStructure {
            assert_eq!(self.get(&tx).unwrap().amount, amount);
            self
        }

        fn assert_disputed(self, tx: u32, disputed: bool) -> TransactionsDataStructure {
            assert_eq!(self.get(&tx).unwrap().disputed, disputed);
            self
        }

        fn assert_removed(self, tx: u32) -> TransactionsDataStructure {
            assert_eq!(self.get(&tx).is_none(), true);
            self
        }
    }

    impl Transaction {
        pub fn new_deposit(amount: Decimal) -> Transaction {
            Transaction {
                r#type: TransactionType::Deposit,
                client_id: 1,
                tx_id: 3,
                amount: Some(amount),
            }
        }

        pub fn new_withdrawal(amount: Decimal) -> Transaction {
            Transaction {
                r#type: TransactionType::Withdrawal,
                client_id: 1,
                tx_id: 3,
                amount: Some(amount),
            }
        }

        pub fn new_dispute(tx: u32) -> Transaction {
            Transaction {
                r#type: TransactionType::Dispute,
                client_id: 1,
                tx_id: tx,
                amount: None,
            }
        }

        pub fn new_resolve(tx: u32) -> Transaction {
            Transaction {
                r#type: TransactionType::Resolve,
                client_id: 1,
                tx_id: tx,
                amount: None,
            }
        }

        pub fn new_chargeback(tx: u32) -> Transaction {
            Transaction {
                r#type: TransactionType::Chargeback,
                client_id: 1,
                tx_id: tx,
                amount: None,
            }
        }
    }

    /// clients: id:1, available: 3000.8114, held: 0, total: 3000.8114, locked:false
    /// archived transactions: [(id:2, amount: 500.6914, disputed:false),(id:1, amount: 1500.12, disputed:false), (id:3, amount: 1000, disputed:false)]
    fn initial_state() -> (Client, TransactionsDataStructure) {
        let mut archived_transactions = TransactionsDataStructure::default();

        archived_transactions.insert(2, ArchivedDeposit::new(Decimal::new(5006914, 4)));
        archived_transactions.insert(1, ArchivedDeposit::new(Decimal::new(150012, 2)));
        archived_transactions.insert(3, ArchivedDeposit::new(Decimal::new(1000, 0)));

        let client = Client {
            held: Default::default(),
            total: initial_amount(),
            locked: false,
        };

        (client, archived_transactions)
    }

    /// 3000.8114
    fn initial_amount() -> Decimal {
        Decimal::new(30008114, 4)
    }

    ///1000.00
    fn thousand() -> Decimal {
        Decimal::new(1000, 0)
    }

    #[test]
    fn handle_deposit_0() {
        let (mut client, mut archived_transactions) = initial_state();

        let deposit = Transaction::new_deposit(Decimal::default());
        execute_transaction(&deposit, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(Decimal::default())
            .assert_available(initial_amount())
            .assert_frozen(false);
    }

    #[test]
    fn handle_deposit_1000() {
        let (mut client, mut archived_transactions) = initial_state();

        let amount = thousand();
        let deposit = Transaction::new_deposit(amount);
        execute_transaction(&deposit, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount().add(amount))
            .assert_held(Decimal::default())
            .assert_available(initial_amount().add(amount))
            .assert_frozen(false);

        archived_transactions
            .assert_amount(3, amount)
            .assert_disputed(3, false);
    }

    #[test]
    fn handle_deposit_negative() {
        let (mut client, mut archived_transactions) = initial_state();

        let deposit = Transaction::new_deposit(thousand().neg());
        execute_transaction(&deposit, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(Decimal::default())
            .assert_available(initial_amount())
            .assert_frozen(false);
    }

    #[test]
    fn handle_withdrawal_0() {
        let (mut client, mut archived_transactions) = initial_state();

        let deposit = Transaction::new_withdrawal(Decimal::default());
        execute_transaction(&deposit, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(Decimal::default())
            .assert_available(initial_amount())
            .assert_frozen(false);
    }

    #[test]
    fn handle_withdrawal_exact_balance() {
        let (mut client, mut archived_transactions) = initial_state();

        let deposit = Transaction::new_withdrawal(initial_amount());
        execute_transaction(&deposit, &mut client, &mut archived_transactions);

        client
            .assert_total(Decimal::default())
            .assert_held(Decimal::default())
            .assert_available(Decimal::default())
            .assert_frozen(false);
    }

    #[test]
    fn disallow_overdraft() {
        let (mut client, mut archived_transactions) = initial_state();

        let deposit = Transaction::new_withdrawal(initial_amount() + thousand());
        execute_transaction(&deposit, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(Decimal::default())
            .assert_available(initial_amount())
            .assert_frozen(false);
    }

    #[test]
    fn handle_withdrawal_1000() {
        let (mut client, mut archived_transactions) = initial_state();

        let amount = thousand();
        let deposit = Transaction::new_withdrawal(amount);
        execute_transaction(&deposit, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount().sub(amount))
            .assert_held(Decimal::default())
            .assert_available(initial_amount().sub(amount))
            .assert_frozen(false);
    }

    #[test]
    fn handle_withdrawal_negative() {
        let (mut client, mut archived_transactions) = initial_state();

        let deposit = Transaction::new_withdrawal(thousand().neg());
        execute_transaction(&deposit, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(Decimal::default())
            .assert_available(initial_amount())
            .assert_frozen(false);
    }

    #[test]
    fn handle_valid_dispute() {
        let (mut client, mut archived_transactions) = initial_state();

        let deposit = Transaction::new_dispute(3);
        execute_transaction(&deposit, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(thousand())
            .assert_available(initial_amount().sub(thousand()))
            .assert_frozen(false);
    }

    #[test]
    fn ignore_duplicate_valid_dispute() {
        let (mut client, mut archived_transactions) = initial_state();

        let dispute = Transaction::new_dispute(3);
        execute_transaction(&dispute, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(thousand())
            .assert_available(initial_amount().sub(thousand()))
            .assert_frozen(false);

        let dispute = Transaction::new_dispute(3);
        execute_transaction(&dispute, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(thousand())
            .assert_available(initial_amount().sub(thousand()))
            .assert_frozen(false);
    }

    #[test]
    fn handle_invalid_dispute() {
        let (mut client, mut archived_transactions) = initial_state();

        let deposit = Transaction::new_dispute(4);
        execute_transaction(&deposit, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(Decimal::default())
            .assert_available(initial_amount())
            .assert_frozen(false);
    }

    #[test]
    fn handle_valid_resolve() {
        let (mut client, mut archived_transactions) = initial_state();

        let deposit = Transaction::new_dispute(3);
        execute_transaction(&deposit, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(thousand())
            .assert_available(initial_amount().sub(thousand()))
            .assert_frozen(false);

        let resolve = Transaction::new_resolve(3);
        execute_transaction(&resolve, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(Decimal::default())
            .assert_available(initial_amount())
            .assert_frozen(false);
    }

    #[test]
    fn handle_nonexistant_tx_resolve() {
        let (mut client, mut archived_transactions) = initial_state();

        let resolve = Transaction::new_resolve(5);
        execute_transaction(&resolve, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(Decimal::default())
            .assert_available(initial_amount())
            .assert_frozen(false);
    }

    #[test]
    fn handle_undisputed_tx_resolve() {
        let (mut client, mut archived_transactions) = initial_state();

        let resolve = Transaction::new_resolve(3);
        execute_transaction(&resolve, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(Decimal::default())
            .assert_available(initial_amount())
            .assert_frozen(false);
    }

    #[test]
    fn handle_valid_chargeback() {
        let (mut client, mut archived_transactions) = initial_state();

        let deposit = Transaction::new_dispute(3);
        execute_transaction(&deposit, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(thousand())
            .assert_available(initial_amount().sub(thousand()))
            .assert_frozen(false);

        let resolve = Transaction::new_chargeback(3);
        execute_transaction(&resolve, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount().sub(thousand()))
            .assert_held(Decimal::default())
            .assert_available(initial_amount().sub(thousand()))
            .assert_frozen(true);

        archived_transactions.assert_removed(3);
    }

    #[test]
    fn handle_nonexistant_tx_chargeback() {
        let (mut client, mut archived_transactions) = initial_state();

        let resolve = Transaction::new_chargeback(5);
        execute_transaction(&resolve, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(Decimal::default())
            .assert_available(initial_amount())
            .assert_frozen(false);
    }

    #[test]
    fn handle_undisputed_tx_chargeback() {
        let (mut client, mut archived_transactions) = initial_state();

        let resolve = Transaction::new_chargeback(3);
        execute_transaction(&resolve, &mut client, &mut archived_transactions);

        client
            .assert_total(initial_amount())
            .assert_held(Decimal::default())
            .assert_available(initial_amount())
            .assert_frozen(false);

        archived_transactions.assert_disputed(3, false);
    }
}
