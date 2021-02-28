import React, {useState,useEffect} from 'react';
import {Spinner, Button} from "react-bootstrap"

// eslint-disable-next-line import/no-anonymous-default-export
export default ({
    ORDERS_FACTORY_ADDRESS,
    client,
    viewKey,
    setViewKey
}) => {
    const [loading, setLoading] = useState(false)

    useEffect(() => {
        if (client.ready && !viewKey.ready) {
          const viewKeys = localStorage.getItem("vk");
          if (viewKeys && JSON.parse(viewKeys)[ORDERS_FACTORY_ADDRESS] && JSON.parse(viewKeys)[ORDERS_FACTORY_ADDRESS][client.accountData.address]) {
            setViewKey({
              ready: true,
              value: JSON.parse(viewKeys)[ORDERS_FACTORY_ADDRESS][client.accountData.address]
            })
          } else {
            setViewKey({
              ready: true,
              value: null
            })
          }
        }
      }, [client, setViewKey, viewKey.ready])
      
    if (viewKey.ready && !viewKey.value) {
        return (
            <Button variant="primary" onClick={async() => {
                setLoading(true)
                try {
                    const response = await getViewKey(client.execute, ORDERS_FACTORY_ADDRESS)
                    localStorage.setItem("vk",JSON.stringify({[ORDERS_FACTORY_ADDRESS]: {[client.accountData.address]: response}}))
                    setViewKey({
                      ready: true,
                      value: response
                    })
                } catch {}
                setLoading(false)
              }}>
                  {
                    loading ? <Spinner animation="border" /> : "Create View Key"
                  }
            </Button>
        )
    } else {
        return null
    }
}

const { fromUtf8 } = require("@iov/encoding");

// eslint-disable-next-line import/no-anonymous-default-export
const getViewKey = async (client, tokenAddress) => {
    let handleMsg = { create_viewing_key: {entropy: "1321313123"} };
    const response = await client.execute(tokenAddress, handleMsg);
    const apiKey = JSON.parse(fromUtf8(response.data))
    if (apiKey.create_viewing_key) {
      return apiKey.create_viewing_key.key
    } else if (apiKey.viewing_key) {
      return apiKey.viewing_key.key
    }
  }