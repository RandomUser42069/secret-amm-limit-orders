# Private order book - Limit order contract for secretAMM

[Issue Description](https://github.com/enigmampc/SecretNetwork/issues/699)

Example:

https://github.com/dgtony/orderbook-rs/blob/master/src

TODO:
* Create Factory method to be called by secret order contracts only that checks a triggerer and the VK auth
* Resolve the problem with the floating point on the secretdev 
* Integrations tests: Create Limit Order, Query Limit Order, Withdraw Limit Order and Query Order Book Peek