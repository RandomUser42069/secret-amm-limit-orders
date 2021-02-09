const { fromUtf8 } = require("@iov/encoding");

// eslint-disable-next-line import/no-anonymous-default-export
export default async (client, pair, factoryAddress) => {
    let handleMsg = { 
        new_secret_order_book_instanciate: {
            //token1_info: pair.asset_infos[0],
            //token2_info: pair.asset_infos[1],
            amm_pair_address: pair.contract_addr,
            amm_pair_hash: pair.contract_hash
        } 
    };

    const response = await client.execute(factoryAddress, handleMsg);
    const apiKey = JSON.parse(fromUtf8(response.data))
    if (apiKey.create_viewing_key) {
      return apiKey.create_viewing_key.key
    } else if (apiKey.viewing_key) {
      return apiKey.viewing_key.key
    }
  }