import React, {useState, useEffect} from 'react';
import logo from './logo.svg';
import './App.css';
import { SigningCosmWasmClient } from 'secretjs';
import ViewKeyButton from "./Containers/ViewKeyButton"
import MyActiveLimitOrders from "./Containers/MyActiveLimitOrders"
import MyHistoryLimitOrders from "./Containers/MyHistoryLimitOrders"
import 'bootstrap/dist/css/bootstrap.min.css';
import CreateNewLimitOrder from "./Containers/CreateNewLimitOrder";
import axios from 'axios';

const AMM_FACTORY_ADDRESS="secret1ypfxpp4ev2sd9vj9ygmsmfxul25xt9cfadrxxy"
const ORDERS_FACTORY_ADDRESS="secret10dls0d070dmjksypfp3d4q3xe4c6jnknyy522g" 
const SSCRT_CONTRACT_ADDRESS="secret1s7c6xp9wltthk5r6mmavql4xld5me3g37guhsx"

function App() {
  const [client, setClient] = useState({
    ready: false,
    execute: null,
    accountData: {
      address: ""
    }
  });

  const [viewKey, setViewKey] = useState({
    ready: false,
    value: null
  });

  const [tokensData, setTokensData] = useState<any>(null);
  const [remountMyLimitOrdersCount, setRemountMyLimitOrdersCount] = useState<number>(0);

  useEffect(() => {
    async function init() {
      setupKeplr(setClient);
      try {
        const response = await axios.get("https://scrt-bridge-api.azurewebsites.net/tokens/?page=0&size=1000");
          setTokensData([...response.data.tokens,{
            dst_address: SSCRT_CONTRACT_ADDRESS,
            decimals: 6,
            display_props: {
              symbol: "sSCRT"
            }
          },{
            dst_address: "secret1ha79qdkjsq7nyy8hagsggfq6zzlwshfmgfv3k0",
            decimals: 18,
            display_props: {
              symbol: "sTST"
            }
          },
        ]);
      } catch (e) {
        setTokensData([{
          dst_address: "secret1ttg5cn3mv5n9qv8r53stt6cjx8qft8ut9d66ed",
          decimals: 18,
          display_props: {
            symbol: "sETH"
          }
        },{
          dst_address: "secret10zr3azpmr42vatq3pey2aaxurug0c668km6rzl",
          decimals: 18,
          display_props: {
            symbol: "sOCEAN"
          }
        },{
          dst_address: SSCRT_CONTRACT_ADDRESS,
          decimals: 6,
          display_props: {
            symbol: "sSCRT"
          }
        },{
          dst_address: "secret1ha79qdkjsq7nyy8hagsggfq6zzlwshfmgfv3k0",
          decimals: 18,
          display_props: {
            symbol: "sTST"
          }
        },
      ]);
      }
    }
    init();
  }, [])

  const remountMyLimitOrders = () => setRemountMyLimitOrdersCount(remountMyLimitOrdersCount+1)

  if(!client.ready) {
    return <div>Loading...</div>
  } else {
    return (
      <div className="App">
          <ViewKeyButton 
            ORDERS_FACTORY_ADDRESS={ORDERS_FACTORY_ADDRESS}
            client={client}
            viewKey={viewKey}
            setViewKey={setViewKey}
          />
          {
            viewKey.value &&  
              <div>
                  <CreateNewLimitOrder 
                    ORDERS_FACTORY_ADDRESS={ORDERS_FACTORY_ADDRESS}
                    AMM_FACTORY_ADDRESS={AMM_FACTORY_ADDRESS}
                    tokensData={tokensData}
                    client={client}
                    viewKey={viewKey.value}
                    remountMyLimitOrders={remountMyLimitOrders}
                  /> 
                  {
                    <MyActiveLimitOrders 
                      key={remountMyLimitOrdersCount} // Used to force remount this component
                      remountMyLimitOrders={remountMyLimitOrders}
                      ORDERS_FACTORY_ADDRESS={ORDERS_FACTORY_ADDRESS}
                      tokensData={tokensData}
                      client={client}
                      viewKey={viewKey.value}
                    />
                  }
                  <br/><br/><br/>
                  {
                    <MyHistoryLimitOrders 
                      key={remountMyLimitOrdersCount} // Used to force remount this component
                      remountMyLimitOrders={remountMyLimitOrders}
                      ORDERS_FACTORY_ADDRESS={ORDERS_FACTORY_ADDRESS}
                      tokensData={tokensData}
                      client={client}
                      viewKey={viewKey.value}
                    />
                  }
              </div>
          }
          
          {
            /*
              <PairsAvailable 
                AMM_FACTORY_ADDRESS={AMM_FACTORY_ADDRESS}
                ORDERS_FACTORY_ADDRESS={ORDERS_FACTORY_ADDRESS}
                client={client}
                viewKey={viewKey.value}
              />
            */
          }
          
      </div>
    );
  }
}

export default App;

const setupKeplr = async (setClient: any) => {
  // Define sleep
  const CHAIN_ID = "holodeck-2";
  
  const sleep = (ms: number) => new Promise((accept) => setTimeout(accept, ms));

  // Wait for Keplr to be injected to the page
  while (
    !window.keplr &&
    !window.getOfflineSigner &&
    !window.getEnigmaUtils
  ) {
    await sleep(10);
  }

  // Use a custom chain with Keplr.
  // On mainnet we don't need this (`experimentalSuggestChain`).
  // This works well with `enigmampc/secret-network-sw-dev`:
  //     - https://hub.docker.com/r/enigmampc/secret-network-sw-dev
  //     - Run a local chain: `docker run -it --rm -p 26657:26657 -p 26656:26656 -p 1337:1337 -v $(shell pwd):/root/code --name secretdev enigmampc/secret-network-sw-dev`
  //     - `alias secretcli='docker exec -it secretdev secretcli'`
  //     - Store a contract: `docker exec -it secretdev secretcli tx compute store /root/code/contract.wasm.gz --from a --gas 10000000 -b block -y`
  // On holodeck, set:
  //     1. CHAIN_ID = "holodeck-2"
  //     2. rpc = "ttp://bootstrap.secrettestnet.io:26657"
  //     3. rest = "https://bootstrap.secrettestnet.io"
  //     4. chainName = Whatever you like
  // For more examples, go to: https://github.com/chainapsis/keplr-example/blob/master/src/main.js
  await window.keplr.experimentalSuggestChain({
    chainId: CHAIN_ID,
    chainName: "Local Secret Chain",
    rpc: "http://bootstrap.secrettestnet.io:26657",
    rest: "https://bootstrap.secrettestnet.io",
    bip44: {
      coinType: 529,
    },
    coinType: 529,
    stakeCurrency: {
      coinDenom: "SCRT",
      coinMinimalDenom: "uscrt",
      coinDecimals: 6,
    },
    bech32Config: {
      bech32PrefixAccAddr: "secret",
      bech32PrefixAccPub: "secretpub",
      bech32PrefixValAddr: "secretvaloper",
      bech32PrefixValPub: "secretvaloperpub",
      bech32PrefixConsAddr: "secretvalcons",
      bech32PrefixConsPub: "secretvalconspub",
    },
    currencies: [
      {
        coinDenom: "SCRT",
        coinMinimalDenom: "uscrt",
        coinDecimals: 6,
      },
    ],
    feeCurrencies: [
      {
        coinDenom: "SCRT",
        coinMinimalDenom: "uscrt",
        coinDecimals: 6,
      },
    ],
    gasPriceStep: {
      low: 0.3,
      average: 0.45,
      high: 0.6,
    },
    features: ["secretwasm"],
  });

  // Enable Keplr.
  // This pops-up a window for the user to allow keplr access to the webpage.
  await window.keplr.enable(CHAIN_ID);

  // Setup SecrtJS with Keplr's OfflineSigner
  // This pops-up a window for the user to sign on each tx we sent
  const keplrOfflineSigner = window.getOfflineSigner(CHAIN_ID);
  const accounts = await keplrOfflineSigner.getAccounts();

  const execute = await new SigningCosmWasmClient(
    "https://bootstrap.secrettestnet.io", // holodeck - https://bootstrap.secrettestnet.io; mainnet - user your LCD/REST provider
    accounts[0].address,
    window.getOfflineSigner(CHAIN_ID),
    window.getEnigmaUtils(CHAIN_ID),
    {
      // 300k - Max gas units we're willing to use for init
      init: {
        amount: [{ amount: "500000", denom: "uscrt" }],
        gas: "500000",
      },
      // 300k - Max gas units we're willing to use for exec
      exec: {
        amount: [{ amount: "500000", denom: "uscrt" }],
        gas: "500000",
      },
    }
  )

  const accountData = await execute.getAccount(accounts[0].address);
  
  setClient({
    ready: true,
    execute,
    accountData
  })
}

declare global {
  interface Window { keplr: any, getOfflineSigner:any, getEnigmaUtils:any }
}


