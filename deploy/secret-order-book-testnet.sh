#!/bin/bash

cd ./contracts/secret-order-book

echo Build new contracts to deploy? [yn]
read toBuild

function wait_for_tx() {
  until (secretcli q tx "$1"); do
      sleep 5
  done
}

if [ "$toBuild" != "${toBuild#[Yy]}" ] ;then
    #cargo clean
    RUST_BACKTRACE=1 cargo unit-test
    rm -f ./contract.wasm ./contract.wasm.gz
    cargo wasm
    cargo schema
    docker run --rm -v $PWD:/contract \
        --mount type=volume,source=factory_cache,target=/code/target \
        --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
        enigmampc/secret-contract-optimizer
fi

#echo factory address?
#read factory_contract_address

factory_contract_address="secret1p25nlkahppha9nqxa23pvussk942ps6xlauhmk"
token1_contract_address=""
token1_code_hash=""
token2_contract_address=""
token2_code_hash=""

secretcli q account $(secretcli keys show -a a)

deployed=$(secretcli tx compute store "contract.wasm.gz" --from a --gas 2000000 -b block -y)
secret_order_book_code_id=$(secretcli query compute list-code | jq '.[-1]."id"')
secret_order_book_code_hash=$(secretcli query compute list-code | jq '.[-1]."data_hash"')
echo "Stored contract: '$secret_order_book_code_id', '$secret_order_book_code_hash'"

deployer_name_a=a

STORE_TX_HASH=$(
  secretcli tx compute execute $(echo "$factory_contract_address" | tr -d '"') '{"change_secret_order_book_contract_code_id": {"code_id": '$secret_order_book_code_id', "code_hash":'${secret_order_book_code_hash}'}}' --from $deployer_name_a -y --gas 1500000 -b block |
  jq -r .txhash
)
wait_for_tx "$STORE_TX_HASH" "Waiting for instantiate to finish on-chain..."
echo $(secretcli query compute tx $STORE_TX_HASH)

#STORE_TX_HASH=$(
#  secretcli tx compute execute $(echo "$factory_contract_address" | tr -d '"') '{"new_secret_order_book_instanciate": {"token1_code_address": '$token1_contract_address', "token1_code_hash":"'${token1_hash:2}'", "token2_code_address": '$token2_contract_address', "token2_code_hash":"'${token2_hash:2}'"}}' --from $deployer_name_a -y --gas 1500000 -b block |
#  jq -r .txhash
#)
#wait_for_tx "$STORE_TX_HASH" "Waiting for instantiate to finish on-chain..."
#echo $(secretcli query compute tx $STORE_TX_HASH)