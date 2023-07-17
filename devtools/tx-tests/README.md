# Configure private key

Create file `devtools/tx-tests/src/config/type_ids.toml`
```
touch devtools/tx-tests/src/config/type_ids.toml
```

Then fill the content as follows.
```
seeder_privkey = "0x111111b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e61"

staker_privkeys = [
    "0x222222b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60",
    "0x333333b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e62",
    "0x444444b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e63",
]

delegator_privkeys = [
    "0x222222b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60",
    "0x333333b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e62",
    "0x444444b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e63",
]
```

# Run
```
cd spark/devtools/tx-tests

cargo run -- -f  // faucet (only used on the dev chain)

cargo run -- -i  // init tx

cargo run -- -m  // mint tx

// stake tx
cargo run -- -s first   // first stake
cargo run -- -s add     // add stake
cargo run -- -s redeem  // redeem stake

// delegate tx
cargo run -- -s first   // first delegate
cargo run -- -s add     // add delegate
cargo run -- -s redeem  // redeem delegate

cargo run -- -t  // stake smt tx

cargo run -- -e  // delegate smt tx

cargo run -- -w  // withdraw tx
```
