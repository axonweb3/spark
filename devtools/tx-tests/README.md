# Config

Create file `devtools/tx-tests/src/config/type_ids.toml`
```
touch devtools/tx-tests/src/config/type_ids.toml
```
This fill will be automatically filled in when running integration tests. You can also fill it by running `cargo run -- tx -n test -i` after claiming CKB from the faucet as described in the next chapter.

`type_ids.toml` stores the type ids of config cells which every Axon-Based chain needs.
For example, every Axon-Based chain needs a selection cell to control the issue of ATs. And the `selection_type_id` is the type id of selection cell.

Create file `devtools/tx-tests/src/config/priv_keys.toml`
```
touch devtools/tx-tests/src/config/priv_keys.toml
```

Then fill `priv_keys.toml` as follows.
```
seeder_privkey = "0x111111b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60"

staker_privkeys = [
    "0x222222b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60",
    "0x333333b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60",
    "0x444444b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60",
    "0x555555b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60",
]

delegator_privkeys = [
    "0x666666b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60",
    "0x777777b054d5dd04013156dced8ba2ce4d8cc5973e10d905a228ea1abc267e60",
]
```

# Faucet

View users' addresses.
```
cargo run -- users -n test --address
```

Then use users' ckb addresses to claim tokens from the [faucet](https://faucet.nervos.org).

# Run

## Run integration tests

The following tests can be found under folder `devtools/tx-tests/src/cases`.

```
cd devtools/tx-tests

cargo run -- cases -n test --all
cargo run -- cases -n test --stake
cargo run -- cases -n test --stake-smt
cargo run -- cases -n test --delegate
cargo run -- cases -n test --delegate-smt
cargo run -- cases -n test --withdraw
cargo run -- cases -n test --reward
cargo run -- cases -n test --metadata
```

## Debug tx

The following commands are commonly used for debugging single tx located in folder `devtools/tx-tests/src/tx`.

```
cd devtools/tx-tests

cargo run -- tx -n dev -f   // faucet (only used on the dev chain)

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
