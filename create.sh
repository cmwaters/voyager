# !/bin/bash

ARGS=`echo '{"config": {"name": "voyager", "symbol": "GENESIS", "decimals": 24, "purpose": "test", "bond": "1000000000000000000000000", "metadata": ""}, "policy": '$COUNCIL'}' | base64`

near call $CONTRACT_ID create "{\"name\": \"voyager\", \"args\": \"$ARGS\"}"  --accountId $CONTRACT_ID --amount 5 --gas 150000000000000