#!/bin/bash

cd contracts/factory
#cargo clean
rm -f ./contract.wasm ./contract.wasm.gz
cargo wasm
cargo schema
docker run --rm -v $PWD:/contract \
--mount type=volume,source=factory_cache,target=/code/target \
--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
enigmampc/secret-contract-optimizer

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

deployed=$(docker exec -it "$docker_name" secretcli tx compute store "/root/code/contracts/factory/contract.wasm.gz" --from a --gas 2000000 -b block -y)
factory_code_id=$(secretcli query compute list-code | jq '.[-1]."id"')
factory_code_hash=$(secretcli query compute list-code | jq '.[-1]."data_hash"')
echo "Stored contract: '$factory_code_id', '$factory_code_hash'"
