use binance_spot_connector_rust::{http::Credentials, ureq::BinanceHttpClient};

pub fn new_http_apiclient() -> BinanceHttpClient {
    let credentials = Credentials::from_hmac(
        "Rw6WLIV7EZBRcSqAfoGFCkj7tUKboDNPa0P6wQ77QQz5BM8aMzK0FEDmrHJlIVPX".to_owned(),
        "  secret_key: L2IWScYwhH2I6hGwzo9JkTnEuCnxTihmzRAg7Fb0jZ6YbYp9eo8bRc3b1kj4HLax
    "
        .to_owned(),
    );
    BinanceHttpClient::with_url("https://testnet.binance.vision").credentials(credentials)
}

#[cfg(test)]
mod tests {
    use super::*;
    use binance_spot_connector_rust::{
        http::{request::RequestBuilder, Credentials, Method},
        ureq::{BinanceHttpClient, Error},
    };
    use futures_util::future::ok;

    #[test]
    fn test_new_http_apiclient() -> Result<(), Box<Error>> {
        let client = new_http_apiclient();
        let request = RequestBuilder::new(Method::Post, "/api/v3/order")
            .params(vec![
                ("symbol", "BNBUSDT"),
                ("side", "SELL"),
                ("type", "LIMIT"),
                ("quantity", "0.1"),
                ("price", "320.2"),
            ])
            .sign();
        let data = client.send(request)?.into_body_str()?;
        log::info!("{}", data);
        Ok(())
    }
}
