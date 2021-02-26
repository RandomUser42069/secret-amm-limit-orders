import React, {useState,useEffect} from 'react';
import {Card, Button, Spinner, Modal, DropdownButton, Dropdown, Table} from 'react-bootstrap'

const PAGINATION_LIMIT = 10;
const PAGINATION_OFFSET = 0;

// eslint-disable-next-line import/no-anonymous-default-export
export default ({
    ORDERS_FACTORY_ADDRESS,
    remountMyLimitOrders,
    tokensData,
    client,
    viewKey
}: MyLimitOrdersProps) => {
    const [myLimitOrders, setMyLimitOrders] = useState<any>(null)

    useEffect(() => {
        async function init() {
            setMyLimitOrders(await client.execute.queryContractSmart(ORDERS_FACTORY_ADDRESS, { 
                user_secret_order_books: {
                    address: client.accountData.address,
                    viewing_key: viewKey
                }
              }))
          }
        init()
    }, [])

    return (
        <div>
            <Table striped bordered hover>
                <thead>
                    <tr>
                        <th>Creation Date</th>
                        <th>Pair</th>
                        <th>Type</th>
                        <th>Status</th>
                        <th>Limit Order</th>
                        <th>Triggered Price</th>
                        <th>Current Price</th>
                        <th>Withdraw</th>
                    </tr>
                </thead>
                <tbody>
                {!myLimitOrders && <Spinner animation="border"/>}
                {
                    myLimitOrders && myLimitOrders.user_secret_order_books.user_secret_order_book &&
                        myLimitOrders.user_secret_order_books.user_secret_order_book.map((orderBookAddress: string) => 
                            <MyLimitOrder 
                                orderBookAddress={orderBookAddress}
                                remountMyLimitOrders={remountMyLimitOrders}
                                tokensData={tokensData}
                                client={client}
                                viewKey={viewKey}
                                myLimitOrders={myLimitOrders}
                                setMyLimitOrders={setMyLimitOrders}
                            />)
                }
                </tbody>
            </Table>
        </div>
        
    )
}


const MyLimitOrder = ({
    orderBookAddress,
    remountMyLimitOrders,
    tokensData,
    client,
    viewKey,
    myLimitOrders,
    setMyLimitOrders
}: any) => {
    const [limitOrdersData, setLimitOrdersData] = useState<any>(null)
    const [orderBookTokensData, setOrderBookTokensData] = useState<any>(null)
    const [ammPriceData, setAmmPriceData] = useState<any>(null)

    useEffect(() => {
        async function init() {
            const limitOrderPromise = client.execute.queryContractSmart(orderBookAddress, { 
                get_limit_orders: {
                    user_address: client.accountData.address,
                    user_viewkey: viewKey,
                    limit: PAGINATION_LIMIT,
                    offset: PAGINATION_OFFSET
                }
              })

            const orderBookTokenDataPromise = client.execute.queryContractSmart(orderBookAddress, { 
                order_book_pair_info: {}
              })

            const [limitOrders, orderBookTokenData] = await Promise.all([limitOrderPromise, orderBookTokenDataPromise]);

            setLimitOrdersData(limitOrders)
            setOrderBookTokensData(orderBookTokenData)

            setAmmPriceData(await getAmmPrice(orderBookTokenData))

            setInterval(async () => {
                setLimitOrdersData(await client.execute.queryContractSmart(orderBookAddress, { 
                    get_limit_orders: {
                        user_address: client.accountData.address,
                        user_viewkey: viewKey,
                        limit: PAGINATION_LIMIT,
                        offset: PAGINATION_OFFSET
                    }
                  }));
                setAmmPriceData(await getAmmPrice(orderBookTokenData))
            },12000)
          }
        init()
    }, [])


    const getAmmPrice = async (orderBookTokenData: any) => {
        return client.execute.queryContractSmart(orderBookTokenData.amm_pair_address, { 
            simulation: {
                offer_asset: {
                    info: {
                        token: {
                            ...orderBookTokenData.assets_info[0].token,
                            viewing_key: ""
                        }
                    },
                    amount: "" + Math.pow(10, tokensData.find((data: any) => data.dst_address === orderBookTokenData.assets_info[0].token.contract_addr).decimals)
                }
            }
          })
    }

    const displayBalance = (index: number, limitOrderData:any, is_withdrew: boolean | null = null) => {
        const tokenData = tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[index].token.contract_addr);
        if (!is_withdrew) {
            return Math.round(limitOrderData.balances[index]/Math.pow(10,orderBookTokensData.assets_info[index].decimal_places) * 100000) / 100000 + " " + tokenData.display_props.symbol
        } else {
            return Math.round(limitOrderData.withdrew_balance[index]/Math.pow(10,orderBookTokensData.assets_info[index].decimal_places) * 100000) / 100000 + " " + tokenData.display_props.symbol
        }
    }

    const displayPrice = (type: string, limitOrderData: any) => {
        const token1Data = tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[0].token.contract_addr);
        const token2Data = tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[1].token.contract_addr);
        if (type === "order") {
            return Math.round(limitOrderData.price/Math.pow(10,orderBookTokensData.assets_info[1].decimal_places) * 100000) / 100000 + " " + token2Data.display_props.symbol + " per " + token1Data.display_props.symbol 
        } else if (type === "amm") {
            return Math.round(ammPriceData.return_amount/Math.pow(10,orderBookTokensData.assets_info[1].decimal_places) * 100000) / 100000 + " " + token2Data.display_props.symbol + " per " + token1Data.display_props.symbol 
        } else if (type === "triggered") {
            if (limitOrderData.is_bid) {
                return (Math.round(limitOrderData.deposit_amount/Math.pow(10,orderBookTokensData.assets_info[1].decimal_places) * 100000) / 100000) / (Math.round(limitOrderData.balances[0]/Math.pow(10,orderBookTokensData.assets_info[0].decimal_places) * 100000) / 100000) + " " +  token2Data.display_props.symbol + " per " + token1Data.display_props.symbol
            } else {
                return (Math.round(limitOrderData.balances[1]/Math.pow(10,orderBookTokensData.assets_info[1].decimal_places) * 100000) / 100000) / (Math.round(limitOrderData.deposit_amount/Math.pow(10,orderBookTokensData.assets_info[0].decimal_places) * 100000) / 100000) + " " +  token2Data.display_props.symbol + " per " + token1Data.display_props.symbol
            }
        }
    }

    const pairDisplay = () => {
        const token1Data = tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[0].token.contract_addr);
        const token2Data = tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[1].token.contract_addr);

        return token1Data.display_props.symbol + " / " + token2Data.display_props.symbol 
    }

    const findTokenData = (index: number) => 
        tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[index].token.contract_addr);

    const getDepositedAmount = (limitOrderData: any) => {
        const index = limitOrderData.is_bid ? 1 : 0;
        const amount = Math.round(limitOrderData.deposit_amount/Math.pow(10,orderBookTokensData.assets_info[index].decimal_places) * 100000) / 100000;

        return "Deposited: " + amount + " " + findTokenData(index).display_props.symbol
    }

    const getExpectedAmount = (limitOrderData: any) => {
        const index = limitOrderData.is_bid ? 0 : 1;
        const amount = Math.round(limitOrderData.expected_amount/Math.pow(10,orderBookTokensData.assets_info[index].decimal_places) * 100000) / 100000;

        return "Expected (~): " + amount + " " + findTokenData(index).display_props.symbol
    }

    const rowStyle = limitOrdersData && limitOrdersData.active_order ? {
        backgroundColor: limitOrdersData.active_order.status === "Filled" ? "#Cfffbc" : "#Fff4ad"
    } : undefined
    
    return (
        <React.Fragment>
            {
                // active order
                limitOrdersData && limitOrdersData.active_order &&
                    <tr key={orderBookAddress} style={rowStyle}>
                        <td>{new Date(limitOrdersData.active_order.timestamp*1000).toLocaleString()}</td>
                        {orderBookTokensData && tokensData && <td>{pairDisplay()}</td>}
                        <td>{limitOrdersData.active_order.is_bid ? "Buy" : "Sell"}</td>
                        <td>{limitOrdersData.active_order.status}</td>
                        {orderBookTokensData && 
                            <div>
                                {displayPrice("order", limitOrdersData.active_order)} <br/><br/>
                                {getDepositedAmount(limitOrdersData.active_order)}<br/>
                                {getExpectedAmount(limitOrdersData.active_order)}
                            </div>}
                        {<td>{
                            <div>
                                {
                                    ammPriceData && 
                                    limitOrdersData.active_order.status === "Filled" ? 
                                    displayPrice("triggered", limitOrdersData.active_order)
                                    : " - "}
                            </div>
                        }</td>}
                        {<td>{ammPriceData ? displayPrice("amm", limitOrdersData.active_order) : " - "}</td>}
                        {limitOrdersData && <td>{<Button onClick={ async () => {
                            try{
                                await client.execute.execute(orderBookAddress, { 
                                    withdraw_limit_order: {}
                                })
                                let update = {...myLimitOrders}
                                let arr = update.user_secret_order_books.user_secret_order_book.filter((address: string) => address !== orderBookAddress)
                                update.user_secret_order_books.user_secret_order_book = arr
                                setMyLimitOrders(update)
                                remountMyLimitOrders()
                            } catch (e) {
                                alert("error on widthdraw: " + e)
                            }
                        }}>
                            Widthdraw <br/>
                            {limitOrdersData && orderBookTokensData && displayBalance(0, limitOrdersData.active_order) + "  and  " + displayBalance(1, limitOrdersData.active_order)}
                        </Button>}</td>}
                    </tr>
            }
            {
                // active order
                limitOrdersData && limitOrdersData.history_orders.length > 0 && 
                    limitOrdersData.history_orders.map((history_order:any) => 
                        <tr key={history_order.timestamp}>
                            <td>{new Date(history_order.timestamp*1000).toLocaleString()}</td>
                            {orderBookTokensData && tokensData && <td>{pairDisplay()}</td>}
                            <td>{history_order.is_bid ? "Buy" : "Sell"}</td>
                            <td>{history_order.status}</td>
                            {orderBookTokensData && 
                                <div>
                                    {displayPrice("order", history_order)} <br/><br/>
                                    {getDepositedAmount(history_order)}<br/>
                                    {getExpectedAmount(history_order)}
                                </div>}
                            {<td>{
                                <div>
                                    {
                                        ammPriceData && 
                                        history_order.status === "Filled" ? 
                                        displayPrice("triggered", history_order)
                                        : " - "}
                                </div>
                            }</td>}
                            {<td> - </td>}
                            {<td>Withdrew:  {history_order.withdrew_balance && orderBookTokensData && displayBalance(0, history_order, true) + "  and  " + displayBalance(1, history_order, true)}</td>}
                        </tr>
                    )
            }
        </React.Fragment>
    )
}

type MyLimitOrdersProps = {
    ORDERS_FACTORY_ADDRESS: string,
    remountMyLimitOrders: any,
    client: any,
    tokensData: any,
    viewKey: string | null
}
