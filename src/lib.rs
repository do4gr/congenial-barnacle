mod archived_deposit;
mod client;
mod execute_transaction;
mod transaction;
use crate::archived_deposit::ArchivedDeposit;
use crate::client::Client;
use crate::client::ClientOutput;
use crate::execute_transaction::execute_transaction;
use crate::transaction::Transaction;
use csv_async::AsyncReaderBuilder;
use rustc_hash::FxHashMap;
use std::error::Error;
use std::io;

// FxHashmap since we won't have any key collisions and want faster lookup on int keys
// we don't need client ordering, otherwise a BTreeMap would give ordering
type ClientsDataStructure = FxHashMap<u16, Client>;
type TransactionsDataStructure = FxHashMap<u32, ArchivedDeposit>;

pub async fn core_logic(input_file_path: &str) -> Result<(), Box<dyn Error>> {
    let mut clients = ClientsDataStructure::default();
    let mut transactions = TransactionsDataStructure::default();
    let mut wtr = csv::Writer::from_writer(io::stdout());

    let file = tokio::fs::File::open(input_file_path).await.unwrap();
    let mut rdr = AsyncReaderBuilder::new()
        .trim(csv_async::Trim::All)
        .flexible(true)
        .create_deserializer(file);

    let mut raw_record = csv_async::ByteRecord::new();
    let headers = rdr.byte_headers().await?.clone();

    while rdr.read_byte_record(&mut raw_record).await? {
        let transaction: Transaction = raw_record.deserialize(Some(&headers))?;

        match clients.get_mut(&transaction.client_id) {
            Some(client) if !client.locked => {
                execute_transaction(&transaction, client, &mut transactions)
            }
            None => {
                let mut client = Client::new();
                execute_transaction(&transaction, &mut client, &mut transactions);
                clients.insert(transaction.client_id, client);
            }
            _ => (),
        }
    }

    for (id, client) in clients {
        wtr.serialize(ClientOutput::from_client(client, id))?;
    }

    wtr.flush()?;
    Ok(())
}
