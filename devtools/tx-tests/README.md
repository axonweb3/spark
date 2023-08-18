# Config

Create file `devtools/tx-tests/src/config/type_ids.toml`
```
touch devtools/tx-tests/src/config/type_ids.toml
```

Create file `devtools/tx-tests/src/config/priv_keys.toml`
```
touch devtools/tx-tests/src/config/priv_keys.toml
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

## Run cases

```
cd spark/devtools/tx-tests

cargo run -- cases -n test --all
cargo run -- cases -n test --delegate
cargo run -- cases -n test --delegate-smt
cargo run -- cases -n test --stake
cargo run -- cases -n test --stake-smt
```

## Run tx
```
cd spark/devtools/tx-tests

cargo run -- tx -n test -f  // faucet (only used on the dev chain)

cargo run -- tx -n test -i  // init tx

cargo run -- tx -n test -m  // mint tx

// stake tx
cargo run -- tx -n test -s first   // first stake
cargo run -- tx -n test -s add     // add stake
cargo run -- tx -n test -s redeem  // redeem stake

// delegate tx
cargo run -- tx -n test -s first   // first delegate
cargo run -- tx -n test -s add     // add delegate
cargo run -- tx -n test -s redeem  // redeem delegate

cargo run -- tx -n test -t  // stake smt tx

cargo run -- tx -n test -e  // delegate smt tx

cargo run -- tx -n test -w  // withdraw tx

cargo run -- tx -n test -a  // metadata tx

cargo run -- tx -n test -r  // reward tx
```
