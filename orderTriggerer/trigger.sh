#!/bin/bash

secretcli config chain-id holodeck-2
secretcli config output json
secretcli config indent true
secretcli config node http://bootstrap.secrettestnet.io:26657
secretcli config trust-node true

trigger_block=$(secretcli status | jq -r .sync_info.latest_block_height)

while :
do
	last_block=$(secretcli status | jq -r .sync_info.latest_block_height)
    if [ $trigger_block -ne $last_block ] ;then
        echo Trigger Check
        #query trigger
        #trigger 
        trigger_block=$last_block
    fi
	sleep 0.5
done
