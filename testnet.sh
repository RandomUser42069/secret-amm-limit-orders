#!/bin/bash

order_factory_contract_address="secret1m6lhg9t54n55snj6xq85r3pyupe65l8xfp9s2g"

my_address="secret1uwdn876f5cal4dskdzny3szml8tgdlgfedtnxy"
amm_pair_address="secret1cwu4vpydy609uthyrddn2c4q0crt4ldccd983k"
amm_pair_hash="3300be84224a4021e96a4940609179b82264e6c981164557cc88aebae0c66c10"
token1_address="secret1s7c6xp9wltthk5r6mmavql4xld5me3g37guhsx"
token1_hash="CD400FB73F5C99EDBC6AAB22C2593332B8C9F2EA806BF9B42E3A523F3AD06F62"
token2_address="secret1ttg5cn3mv5n9qv8r53stt6cjx8qft8ut9d66ed"
token2_hash="2DA545EBC441BE05C9FA6338F3353F35AC02EC4B02454BC49B1A66F4B9866AED"
token2_vk="api_key_Kp5R1zMjYEdxEgtRn/TuSsUQPuxhMdBaDVCKyeT9vDQ="

order_vk="b/SDvg53Ff0e2YH5/nTSu5r/0dWqZujrQS8Arg9x2j4="

#Check Hashes
#secretcli query compute tx 

#Create VK
#secretcli tx compute execute $order_factory_contract_address '{"create_viewing_key": { "entropy": "123"}}' --from a -y --gas 1500000 -b block 

#Check Balance
#secretcli q compute query $token2_address '{"balance": {"address":"'$my_address'", "key":"'$token2_vk'"}}'

#Check AMM pair info
#secretcli q compute query secret1sv2wmfs5z68atl9sgqqm3kpykh92u0qa48h5p2 '{"pairs": {}}'

#Create Pair on order factory address
#secretcli tx compute execute $order_factory_contract_address '{"new_secret_order_book_instanciate": { "amm_pair_address": "'$amm_pair_address'", "amm_pair_hash": "'$amm_pair_hash'"}}' --from a -y --gas 1500000 -b block 

#Get address of Orderbook
orderbook_address=$(secretcli q compute query $order_factory_contract_address '{"secret_order_books": {"contract_address": "'$amm_pair_address'"}}' | jq -r .secret_order_books.secret_order_book.contract_addr)

#Create Limit Order
#msg=$(base64 -w 0 <<<'{"create_limit_order": {"is_bid": true, "price": "123"}}')
#secretcli tx compute execute $token2_address '{"send":{"recipient": "'$orderbook_address'", "amount": "1000", "msg": "'"$msg"'"}}' --from a -y --gas 1500000 -b block

#Get Limit Order
#secretcli q compute query $orderbook_address '{"get_limit_order": {"user_address":"'$my_address'", "user_viewkey":"'$order_vk'"}}'

#Widthdraw Limit Order
#secretcli tx compute execute $orderbook_address '{"withdraw_limit_order": {}}' --from a -y --gas 1500000 -b block

#Check if there are limit orders to trigger
#secretcli q compute query $orderbook_address '{"check_order_book_trigger":{}}'

secretcli q compute query $amm_pair_address '{"simulation":{"offer_asset":{"info": {"token": {"contract_addr": "'$token1_address'", "token_code_hash": "'$token1_hash'", "viewing_key": ""}},"amount":"100000000"}}}'

# total swap amount => return_amount
# slippage % => spread_amount * 100 / return_amount
# swap amount + slippage => return_amount + spread_amount

#      1 Token1 <=> X Token2
# amount Token1 <=> response token2

# cur price => (response.return_amount + spread_amount)/amount

#NOTE: SAME DECIMAL PLACES FOR THEM SO SSCRT NEEDS TO ADD 12 ZEROES

# For a 0 slippage order we would need to have the order trigger only when price is <= (response.return_amount + spread_amount)/amount

#PROBLEM: HOW TO ORGANIZE THE ORDER BOOK IF WE ONLY KNOW THE REAL PRICE AFTER A SIMULATE DEPENDING ON THE AMOUNT TO SWAP