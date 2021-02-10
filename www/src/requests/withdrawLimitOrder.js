// eslint-disable-next-line import/no-anonymous-default-export
export default async (client, pair) => {
    const orderBookPairAddress = pair.order_book_data.contract_addr

    let handleMsg = { 
        withdraw_limit_order: {}
    };

    await client.execute(orderBookPairAddress, handleMsg);
  }