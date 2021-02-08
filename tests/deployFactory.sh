#!/bin/bash

echo Build new contracts to deploy? [yn]
read toBuild

if [ "$toBuild" != "${toBuild#[Yy]}" ] ;then
    cd contracts/factory
    #cargo clean
    #RUST_BACKTRACE=1 cargo unit-test
    #rm -f ./contract.wasm ./contract.wasm.gz
    #cargo wasm
    #cargo schema
    #docker run --rm -v $PWD:/contract \
    #--mount type=volume,source=factory_cache,target=/code/target \
    #--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    #enigmampc/secret-contract-optimizer

    cd ../secret-order-book
    #cargo clean
    #RUST_BACKTRACE=1 cargo unit-test
    #rm -f ./contract.wasm ./contract.wasm.gz
    #cargo wasm
    #cargo schema
    #docker run --rm -v $PWD:/contract \
    #--mount type=volume,source=factory_cache,target=/code/target \
    #--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    #enigmampc/secret-contract-optimizer
fi

docker_name=secretdev

function secretcli() {
  docker exec "$docker_name" secretcli "$@";
}

function wait_for_tx() {
  until (secretcli q tx "$1"); do
      sleep 5
  done
}

export SGX_MODE=SW

################################################################
## Get current contracts onchain
################################################################

deployer_name_a=a
deployer_name_b=b
deployer_name_c=c

deployer_address_a=$(secretcli keys show -a $deployer_name_a)
echo "Deployer address a: '$deployer_address_a'"

deployer_address_b=$(secretcli keys show -a $deployer_name_b)
echo "Deployer address b: '$deployer_address_b'"

deployer_address_c=$(secretcli keys show -a $deployer_name_c)
echo "Deployer address c: '$deployer_address_c'"

token1_contract_address=$(docker exec -it $docker_name secretcli query compute list-contract-by-code 1 | jq '.[-1].address')
token1_hash="$(secretcli query compute contract-hash $(echo "$token1_contract_address" | tr -d '"'))"
echo "token1 contract address: '$token1_contract_address'"

token2_contract_address=$(docker exec -it $docker_name secretcli query compute list-contract-by-code 1 | jq '.[-2].address')
token2_hash="$(secretcli query compute contract-hash $(echo "$token2_contract_address" | tr -d '"'))"
echo "token2 contract address: '$token2_contract_address'"

################################################################
## Deploy Factory and Secret Order Book
################################################################

deployed=$(docker exec -it "$docker_name" secretcli tx compute store "/root/code/contracts/factory/contract.wasm.gz" --from a --gas 2000000 -b block -y)
factory_code_id=$(secretcli query compute list-code | jq '.[-1]."id"')
factory_code_hash=$(secretcli query compute list-code | jq '.[-1]."data_hash"')
echo "Stored contract: '$factory_code_id', '$factory_code_hash'"

deployed=$(docker exec -it "$docker_name" secretcli tx compute store "/root/code/contracts/secret-order-book/contract.wasm.gz" --from a --gas 2000000 -b block -y)
secret_order_book_code_id=$(secretcli query compute list-code | jq '.[-1]."id"')
secret_order_book_code_hash=$(secretcli query compute list-code | jq '.[-1]."data_hash"')
echo "Stored contract: '$secret_order_book_code_id', '$secret_order_book_code_hash'"

label=$(date +"%T")

################################################################
## Instanciate Factory
################################################################
STORE_TX_HASH=$( 
  secretcli tx compute instantiate $factory_code_id '{"entropy": "'$RANDOM'", "secret_order_book_code_id": '$secret_order_book_code_id', "secret_order_book_code_hash": '$secret_order_book_code_hash'}' --from $deployer_name_a --gas 1500000 --label $label -b block -y |
  jq -r .txhash
)
wait_for_tx "$STORE_TX_HASH" "Waiting for instantiate to finish on-chain..."

factory_contract_address=$(docker exec -it $docker_name secretcli query compute list-contract-by-code $factory_code_id | jq '.[-1].address')
echo "factory_contract_address: '$factory_contract_address'"

################################################################
## Factory Handle Instanciate Secret Order Book - Token1 and Token2
################################################################
STORE_TX_HASH=$(
  secretcli tx compute execute $(echo "$factory_contract_address" | tr -d '"') '{"new_secret_order_book_instanciate": {"token1_info": {"is_native_token": false, "token":{"contract_addr":'$token1_contract_address',"token_code_hash": "'${token1_hash:2}'"}}, "token2_info": {"is_native_token": false, "token":{"contract_addr":'$token2_contract_address',"token_code_hash": "'${token2_hash:2}'"}}}}' --from $deployer_name_a -y --gas 1500000 -b block |
  jq -r .txhash
)
wait_for_tx "$STORE_TX_HASH" "Waiting for instantiate to finish on-chain..."
echo $(docker exec $docker_name secretcli query compute tx $STORE_TX_HASH)

################################################################
## Factory Query Secret Order Books - 1
################################################################
secretcli q compute query $(echo "$factory_contract_address" | tr -d '"') '{"secret_order_books": {"token_address": '$token1_contract_address'}}'
secretcli q compute query $(echo "$factory_contract_address" | tr -d '"') '{"secret_order_books": {"token_address": '$token2_contract_address'}}'
secret_order_book_address1=$(docker exec $docker_name secretcli q compute query $(echo "$factory_contract_address" | tr -d '"') '{"secret_order_books": {"token_address": '$token1_contract_address'}}' | jq -r .secret_order_books.secret_order_books[0])
echo $secret_order_book_address1

################################################################
## Factory Handle Instanciate Secret Order Book - Native Token + Token2
################################################################
STORE_TX_HASH=$(
  secretcli tx compute execute $(echo "$factory_contract_address" | tr -d '"') '{"new_secret_order_book_instanciate": {"token1_info": {"is_native_token": true, "native_token":{"denom": "uscrt"}}, "token2_info": {"is_native_token": false, "token":{"contract_addr":'$token2_contract_address',"token_code_hash": "'${token2_hash:2}'"}}}}' --from $deployer_name_a -y --gas 1500000 -b block |
  jq -r .txhash
)
wait_for_tx "$STORE_TX_HASH" "Waiting for instantiate to finish on-chain..."
echo $(docker exec $docker_name secretcli query compute tx $STORE_TX_HASH)

################################################################
## Factory Query Secret Order Books - 2
################################################################
secretcli q compute query $(echo "$factory_contract_address" | tr -d '"') '{"secret_order_books": {"token_address": '$token1_contract_address'}}'
secretcli q compute query $(echo "$factory_contract_address" | tr -d '"') '{"secret_order_books": {"token_address": '$token2_contract_address'}}'
secret_order_book_address2=$(docker exec $docker_name secretcli q compute query $(echo "$factory_contract_address" | tr -d '"') '{"secret_order_books": {"token_address": '$token2_contract_address'}}' | jq -r .secret_order_books.secret_order_books[0])
echo $secret_order_book_address2

################################################################
## Factory Create User B VK
################################################################
STORE_TX_HASH=$(
  secretcli tx compute execute $(echo "$factory_contract_address" | tr -d '"') '{"create_viewing_key": {"entropy": "'$RANDOM'"}}' --from $deployer_name_b -y --gas 1500000 -b block |
  jq -r .txhash
)
wait_for_tx "$STORE_TX_HASH" "Waiting for instantiate to finish on-chain..."
user_factory_vk_b=$(docker exec $docker_name secretcli query compute tx $STORE_TX_HASH | jq '.output_data_as_string | fromjson.viewing_key.key')

################################################################
## Secret Order Book - Create Limit Order
################################################################
echo "Create Limit Order"
STORE_TX_HASH=$(
  secretcli tx compute execute $(echo "$token1_contract_address" | tr -d '"') '{"send":{"recipient": "'$secret_order_book_address1'", "amount": "1000000", "msg": "'"$(base64 -w 0 <<<'{"create_limit_order": {"is_bid": true, "price": "123"}}')"'"}}' --from $deployer_name_b -y --gas 1500000 -b block |
  jq -r .txhash
)
wait_for_tx "$STORE_TX_HASH" "Waiting for instantiate to finish on-chain..."
echo $(docker exec $docker_name secretcli query compute tx $STORE_TX_HASH)

################################################################
## Secret Order Book - Create Limit Order
################################################################
echo "Create Limit Order"
STORE_TX_HASH=$(
  secretcli tx compute execute $secret_order_book_address2 '{"receive_native_token": {"is_bid": true, "price": "123"}}' --amount 1000000uscrt --from $deployer_name_b -y --gas 1500000  -b block |
  jq -r .txhash
)
wait_for_tx "$STORE_TX_HASH" "Waiting for instantiate to finish on-chain..."
echo $(docker exec $docker_name secretcli query compute tx $STORE_TX_HASH)

################################################################
## Secret Order Book - Query Limit Order
################################################################
secretcli q compute query $(echo "$secret_order_book_address1" | tr -d '"') '{"get_limit_order": {"user_address": "'$deployer_address_b'", "user_viewkey": '$user_factory_vk_b'}}'
secretcli q compute query $(echo "$secret_order_book_address2" | tr -d '"') '{"get_limit_order": {"user_address": "'$deployer_address_b'", "user_viewkey": '$user_factory_vk_b'}}'

################################################################
## Secret Order Book - Widthdraw Limit Order 1
################################################################
STORE_TX_HASH=$(
  secretcli tx compute execute $(echo "$secret_order_book_address1" | tr -d '"') '{"withdraw_limit_order": {}}' --from $deployer_name_b -y --gas 1500000 -b block |
  jq -r .txhash
)
wait_for_tx "$STORE_TX_HASH" "Waiting for instantiate to finish on-chain..."
echo $(docker exec $docker_name secretcli query compute tx $STORE_TX_HASH)

################################################################
## Secret Order Book - Widthdraw Limit Order 2
################################################################
STORE_TX_HASH=$(
  secretcli tx compute execute $(echo "$secret_order_book_address2" | tr -d '"') '{"withdraw_limit_order": {}}' --from $deployer_name_b -y --gas 1500000 -b block |
  jq -r .txhash
)
wait_for_tx "$STORE_TX_HASH" "Waiting for instantiate to finish on-chain..."
echo $(docker exec $docker_name secretcli query compute tx $STORE_TX_HASH)

################################################################
## Secret Order Book - Query Check Order Book Trigger
################################################################
#secretcli q compute query $(echo "$secret_order_book_address" | tr -d '"') '{"check_order_book_trigger": {"user_address": "'$deployer_address_b'", "user_viewkey": '$user_factory_vk_b'}}'
