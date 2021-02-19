#!/bin/bash

order_factory_contract_address="secret1legkt2lx2myet6es3vlct22w49f80ukqz88k3q"

my_address="secret1uwdn876f5cal4dskdzny3szml8tgdlgfedtnxy"
amm_pair_address="secret148jpzfh6lvencwtxa6czsk8mxm7kuecncz0g0y"
amm_pair_hash="f86b5c3ca0381ce7edfffa534789501ae17cf6b21515213693baf980765729c2"
token1_address="secret1s7c6xp9wltthk5r6mmavql4xld5me3g37guhsx"
token1_hash="CD400FB73F5C99EDBC6AAB22C2593332B8C9F2EA806BF9B42E3A523F3AD06F62"
token1_my_vk="api_key_ZA1M96VGOLex1WycZwXQObNCdAcqNCQDQIcIGjCSPO8="
token2_address="secret1ttg5cn3mv5n9qv8r53stt6cjx8qft8ut9d66ed"
token2_hash="2DA545EBC441BE05C9FA6338F3353F35AC02EC4B02454BC49B1A66F4B9866AED"
token2_vk="api_key_Kp5R1zMjYEdxEgtRn/TuSsUQPuxhMdBaDVCKyeT9vDQ="

order_vk="x5EvcmoYP/Jq1hRpysQCElToK4T7IcSNl3DCdp7xJ6o="

#Check Hashes
#secretcli query compute tx 3A9BCE55AE71A483B45C5C4D9A17641829275B99CD2C4B2535ABC442215A5686

#Create VK
#secretcli tx compute execute $order_factory_contract_address '{"create_viewing_key": { "entropy": "123"}}' --from a -y --gas 1500000 -b block 

#Check Balance
#secretcli q compute query $token2_address '{"balance": {"address":"'$my_address'", "key":"'$token2_vk'"}}'

#Check AMM pair info
#secretcli q compute query secret1ypfxpp4ev2sd9vj9ygmsmfxul25xt9cfadrxxy '{"pairs": {}}'

#Create Pair on order factory address
#secretcli tx compute execute $order_factory_contract_address '{"new_secret_order_book_instanciate": { "amm_pair_address": "'$amm_pair_address'", "amm_pair_hash": "'$amm_pair_hash'"}}' --from a -y --gas 1500000 -b block 

#Get address of Orderbook
orderbook_address=$(secretcli q compute query $order_factory_contract_address '{"secret_order_books": {"contract_address": "'$amm_pair_address'"}}' | jq -r .secret_order_books.secret_order_book.contract_addr)

#Query Pair Info
#secretcli q compute query $orderbook_address '{"order_book_pair_info":{}}'

#Create Limit Order
#msg=$(base64 -w 0 <<<'{"create_limit_order": {"is_bid": true, "price": "4041160934948558462"}}')
#secretcli tx compute execute $token1_address '{"send":{"recipient": "'$orderbook_address'", "amount": "1000000", "msg": "'"$msg"'"}}' --from a -y --gas 1500000 -b block

#Get User OrderBooks
secretcli q compute query $order_factory_contract_address '{"user_secret_order_books": {"address":"'$my_address'", "viewing_key":"'$order_vk'"}}'

#Get Limit Order
#secretcli q compute query $orderbook_address '{"get_limit_order": {"user_address":"'$my_address'", "user_viewkey":"'$order_vk'"}}'

#Widthdraw Limit Order
#secretcli tx compute execute $orderbook_address '{"withdraw_limit_order": {}}' --from a -y --gas 1500000 -b block

#Check if there are limit orders to trigger
#secretcli q compute query $orderbook_address '{"check_order_book_trigger":{}}'

#Trigger Limit Orders
#secretcli tx compute execute $orderbook_address '{"trigger_limit_orders": {}}' --from a -y --gas 1500000 -b block


#secretcli q compute query $amm_pair_address '{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"secret1s7c6xp9wltthk5r6mmavql4xld5me3g37guhsx","token_code_hash":"CD400FB73F5C99EDBC6AAB22C2593332B8C9F2EA806BF9B42E3A523F3AD06F62","viewing_key":""}},"amount":"1000000"}}}' 
#secretcli q compute query $amm_pair_address '{"simulation":{"offer_asset":{"info":{"token":{"contract_addr":"secret1ttg5cn3mv5n9qv8r53stt6cjx8qft8ut9d66ed","token_code_hash":"2DA545EBC441BE05C9FA6338F3353F35AC02EC4B02454BC49B1A66F4B9866AED","viewing_key":""}},"amount":"1"}}}'

# total swap amount => return_amount 
# slippage % => spread_amount * 100 / return_amount
# swap amount + slippage => return_amount + spread_amount

#      1 Token1 <=> X Token2
# amount Token1 <=> response token2

# cur price => (response.return_amount + spread_amount)/amount
