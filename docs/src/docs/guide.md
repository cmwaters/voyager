# Guide

This section provides a simple walkthough designed to display some of the [features](./features.md) that are part of the Voyager DAO. Before we get started, you will need to install the [Near CLI](https://docs.near.org/docs/tools/near-cli#installation). Then clone the [repository](https://github.com/cmwaters/voyager) and `cd` into the directory.

1. Login to your account. If you haven't set up one you can create a [testnet account](https://wallet.testnet.near.org/). For this guide you may want to create a couple if you want to experiment with adding and removing members.
```
near login
```

2. Deploy and initialize the voyager factory. We are going to use `CONTRACT_ID` to represent the account holder. You can set this up, for example, by running `CONTRACT_ID=voyager.testnet`.
```
near deploy $CONTRACT_ID --wasmFile=factory/res/voyager_factory.wasm
```
```
near call $CONTRACT_ID new --accountId $CONTRACT_ID
```

3. Create a DAO. Let's first define the council as consisting only of the `CONTRACT_ID` account.
```
ARGS=`echo '{"config": {"name": "genesis", "symbol": "GENESIS", "decimals": 24, "purpose": "test", "bond": "1000000000000000000000000", "metadata": ""}, "policy": '["$CONTRACT_ID"]'}' | base64`
```
Now use the factory to initiate the dao
```
near call $CONTRACT_ID create "{\"name\": \"genesis\", \"args\": \"$ARGS\"}"  --accountId $CONTRACT_ID --amount 5 --gas 150000000000000
```
The commands should return `true` at the bottom if it passed. You can further validate it by running `near view genesis.$CONTRACT_ID get_policy` or to check all the DAO's the factory has built `near view $CONTRACT_ID get_dao_list`.

4. Create a proposal. 