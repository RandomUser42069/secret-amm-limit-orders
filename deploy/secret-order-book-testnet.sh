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

factory_contract_address="secret1d6g7y7hh6jwm0wudkzx8dx000q602qwuja8ssh"

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

# Deploy SSCRT/ETH
amm_pair_address="secret148jpzfh6lvencwtxa6czsk8mxm7kuecncz0g0y"
amm_pair_hash="f86b5c3ca0381ce7edfffa534789501ae17cf6b21515213693baf980765729c2"
sSCRT_fees=1000000 # 1 sSCRT => GAS FEE TESTNET: 1 SCRT
sETH_fees=1500000000000000 # 0.0015 sETH
secretcli tx compute execute $factory_contract_address '{"new_secret_order_book_instanciate": { "amm_pair_address": "'$amm_pair_address'", "amm_pair_hash": "'$amm_pair_hash'", "token1_fee": "'${sSCRT_fees}'", "token2_fee": "'${sETH_fees}'"}}' --from a -y --gas 1500000 -b block 

# Deploy SSCRT/OCEAN
amm_pair_address="secret1nv90j233x88teghhwdz9l0hj4vzrrcwjl4q6fg"
amm_pair_hash="f86b5c3ca0381ce7edfffa534789501ae17cf6b21515213693baf980765729c2"
sSCRT_fees=1000000 # 1 sSCRT => GAS FEE TESTNET: 1 SCRT
sOCEAN_fees=2000000000000000000 # 2 sOCEAN
secretcli tx compute execute $factory_contract_address '{"new_secret_order_book_instanciate": { "amm_pair_address": "'$amm_pair_address'", "amm_pair_hash": "'$amm_pair_hash'", "token1_fee": "'${sSCRT_fees}'", "token2_fee": "'${sOCEAN_fees}'"}}' --from a -y --gas 1500000 -b block 

#STORE_TX_HASH=$(
#  secretcli tx compute execute secret19qepenq0p7rz5nc5ak7pvxr8fax2yswfagwzc3 '{"new_secret_order_book_instanciate": {"token1_info": {"is_native_token": true, "native_token":{"denom": "uscrt"}}, "token2_info": {"is_native_token": false, "token":{"contract_addr":"secret1j50u6hvume8pkq2c7lcmktrzd7lrymxhujrnax","token_code_hash": "78bdf9cdd7538fc96dfb18372635a20162243e49ca9bcd4bd2aff6300d2bc5e2"}}}}' --from a -y --gas 1500000 -b block |
#  jq -r .txhash
#)
#wait_for_tx "$STORE_TX_HASH" "Waiting for instantiate to finish on-chain..."
#echo $(secretcli query compute tx $STORE_TX_HASH)

#secretcli q compute query secret19qepenq0p7rz5nc5ak7pvxr8fax2yswfagwzc3 '{"secret_order_books": {}}'
#secretcli q compute query secret19qepenq0p7rz5nc5ak7pvxr8fax2yswfagwzc3 '{"secret_order_books": {"token_address": "secret1j50u6hvume8pkq2c7lcmktrzd7lrymxhujrnax"}}'