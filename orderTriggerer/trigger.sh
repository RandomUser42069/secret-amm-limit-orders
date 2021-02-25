#!/bin/bash

secretcli config chain-id holodeck-2
secretcli config output json
secretcli config indent true
secretcli config node http://bootstrap.secrettestnet.io:26657
secretcli config trust-node true

trigger_block=$(secretcli status | jq -r .sync_info.latest_block_height)

order_factory_contract_address="secret1lqsqe8kjeuk22vwhkkw3k787ykvcn4kk649mus"
amm_pair_address="secret148jpzfh6lvencwtxa6czsk8mxm7kuecncz0g0y"

orderbook_address=$(secretcli q compute query $order_factory_contract_address '{"secret_order_books": {"contract_address": "'$amm_pair_address'"}}' | jq -r .secret_order_books.secret_order_book.contract_addr)

while :
do
	last_block=$(secretcli status | jq -r .sync_info.latest_block_height)
    if [ $trigger_block -ne $last_block ] ;then
        result=$(secretcli q compute query $orderbook_address '{"check_order_book_trigger":{}}')
        echo Query Trigger Result: $result
        if [ $result = true ] ;then
            secretcli tx compute execute $orderbook_address '{"trigger_limit_orders": {}}' --from a -y --gas 3000000 -b block
        fi
        trigger_block=$last_block
    fi
	sleep 0.5
done
