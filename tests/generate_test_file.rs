use rand::distributions::WeightedIndex;
use rand::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[ignore]
#[test]
/// quick and dirty sample file generator, will produce invalid transactions
fn generate_test_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();

    let tpes = &["deposit", "withdrawal", "dispute", "resolve", "chargeback"];
    let weights = [16, 14, 7, 2, 1];
    let dist = WeightedIndex::new(&weights).unwrap();
    let mut client_transactions: HashMap<u16, Vec<u32>> = HashMap::new();
    let mut clients = vec![];

    let mut csv_content = String::new();
    csv_content.push_str("type,client,tx,amount\n");
    for tx_id in 1..u32::MAX / 1000 {
        let chosen_type = tpes[dist.sample(&mut rng)];

        let (client_id, amount_or_nothing, tx_id) =
            if matches!(chosen_type, "deposit" | "withdrawal") {
                let client_id: u16 = rng.gen();
                let client_entry = client_transactions.entry(client_id).or_insert(vec![]);
                client_entry.push(tx_id);
                clients.push(client_id);
                (
                    client_id,
                    format!(",{}", Decimal::from(rng.gen::<u16>())),
                    tx_id,
                )
            } else {
                let client_id = clients.choose(&mut rand::thread_rng()).unwrap();
                let tx_id = if let Some(tx_ids) = client_transactions.get(client_id) {
                    *tx_ids.choose(&mut rand::thread_rng()).unwrap()
                } else {
                    123
                };

                (*client_id, "".into(), tx_id)
            };
        csv_content.push_str(&format!(
            "{},{},{}{}\n",
            chosen_type, client_id, tx_id, amount_or_nothing
        ));
    }

    std::fs::write("./large_test_file.csv", csv_content)?;

    Ok(())
}
