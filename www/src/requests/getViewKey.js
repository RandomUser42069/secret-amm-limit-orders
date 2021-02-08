const { fromUtf8 } = require("@iov/encoding");

// eslint-disable-next-line import/no-anonymous-default-export
export default async (client, tokenAddress) => {
    let handleMsg = { create_viewing_key: {entropy: "1321313123"} };
    const response = await client.execute(tokenAddress, handleMsg);
    const apiKey = JSON.parse(fromUtf8(response.data))
    if (apiKey.create_viewing_key) {
      return apiKey.create_viewing_key.key
    } else if (apiKey.viewing_key) {
      return apiKey.viewing_key.key
    }
  }