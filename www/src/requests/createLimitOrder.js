const { fromUtf8 } = require("@iov/encoding");

// eslint-disable-next-line import/no-anonymous-default-export
export default async (client, pair) => {
    //check if what i want to send is the native token or snip20

    const tokenAddress = pair.asset_infos[1].token.contract_addr
    const orderBookPairAddress = pair.order_book_data.contract_addr
    
    const amount = (parseFloat(0.0005)*parseFloat("1e" + pair.asset_infos[1].token_info.decimals)).toString()
    const is_bid = true
    const price = "100"

    //snip20 => Send a transfer with a message to the snip20 contract
    let handleMsg = { 
        send: {
            recipient: orderBookPairAddress,
            amount,
            msg: btoa(JSON.stringify({
                create_limit_order: {
                    is_bid,
                    price
                }
            }))
        } 
    };

    console.log(handleMsg)

    const response = await client.execute(tokenAddress, handleMsg);
    const apiKey = JSON.parse(fromUtf8(response.data))
    if (apiKey.create_viewing_key) {
      return apiKey.create_viewing_key.key
    } else if (apiKey.viewing_key) {
      return apiKey.viewing_key.key
    }
  }