# Private order book - Limit order contract for secretAMM

[Issue Description](https://github.com/enigmampc/SecretNetwork/issues/699)

TODO Sequence:
* Factory Instanciate
    * "entropy": "'$RANDOM'"
    * "secret_order_book_code_id": 1
    * "secret_order_book_code_hash": "'12131'"
* Factory Create User View Key
    * "entropy": "'$RANDOM'"
* Factory Admin Change Code ID
    * codeid
    * codehash
* Factory Instanciate Secret Order Book
    * snip20 token1
    * snip20 token2
