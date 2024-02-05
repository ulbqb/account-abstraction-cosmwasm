#!/bin/bash
RPC_URL=http://host.docker.internal:26657
FAUCE_URL=http://host.docker.internal:8081
MNEMONIC="test test test test test test test test test test test junk"
COMMON_OPT="--from bundler --gas-prices 0.025cony --gas auto --gas-adjustment 1.5 --node $RPC_URL --keyring-backend test --chain-id simd-testing -b block --log_level none -y"

fnsad keys add bundler --account 0 --recover --keyring-backend test <<< $MNEMONIC &> /dev/null
fnsad keys add user --account 1 --recover --keyring-backend test <<< $MNEMONIC &> /dev/null
curl --header "Content-Type: application/json" \
     --request POST \
     --data '{"denom":"cony","address":"'$(fnsad keys show bundler --keyring-backend test -a)'"}' \
     $FAUCE_URL/credit &> /dev/null

# store and instantiate account contract
PUB_KEY=$(base64-to-u8s/target/debug/base64-to-u8s $(fnsad keys show user --keyring-backend test -p | jq .key -r))
RESULT=$(fnsad tx wasm store-instantiate cosmwasm/artifacts/account.wasm "{\"type_url\":\"/cosmos.crypto.secp256k1.PubKey\",\"key\":$PUB_KEY}" --label "Account Contract" --amount 1000cony $COMMON_OPT)
CONTRACT=$(echo $RESULT | jq '.logs[0].events[] | select (.type == "instantiate").attributes[] | select (.key == "_contract_address").value' -r)

# generate user op tx
PRIVATE_KEY=$(fnsad keys export user --keyring-backend test --unsafe --unarmored-hex <<< y)
USER_OP=$(wallet/target/debug/wallet $CONTRACT 0 $PRIVATE_KEY)

# send user op by bundler
fnsad tx wasm execute $CONTRACT "{\"send_tx\":{\"tx\":$USER_OP}}" $COMMON_OPT | jq .
