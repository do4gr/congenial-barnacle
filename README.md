# Transaction Engine Notes

## Assumptions

### Negative Amounts in Transaction
Withdrawals and deposits with negative amounts are ignored. They cause no errors, but are also not persisted, so cannot be disputed / resolved /charged back.

### Locked Clients
For locked clients all further transactions will be rejected.

### Chargebacks
Only deposits can be disputed /resolved /charged back. 
Chargebacks are final and are removed from the transactions working set. If they needed to be kept, an additional flag would be needed on the data structure.

## Performance
FxHashMaps are used since the given keys are unique and we do not need to care about collision resistance. 
The effect of different data-structures on the performance was evaluated for the sync version of the code with `cargo bench`

## Testing
Unit tests cover the core logic of executing transactions. Integration tests handle reading of differently formatted files and the output as well as some more complicated scenarios.
