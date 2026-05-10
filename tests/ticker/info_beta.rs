use httpmock::{Method::GET, MockServer};
use rust_decimal::Decimal;
use url::Url;
use yfinance_rs::{ApiPreference, Ticker, YfClient};

/// Asserts that `Info.beta` is populated from the `defaultKeyStatistics` module.
#[tokio::test]
async fn info_beta_from_key_statistics() {
    let server = MockServer::start();
    let sym = "AAPL";
    let crumb = "test-crumb";

    // quote
    server.mock(|when, then| {
        when.method(GET)
            .path("/v7/finance/quote")
            .query_param("symbols", sym);
        then.status(200)
            .header("content-type", "application/json")
            .body(crate::common::fixture("quote_v7", sym, "json"));
    });

    // profile
    server.mock(|when, then| {
        when.method(GET)
            .path(format!("/v10/finance/quoteSummary/{sym}"))
            .query_param("modules", "assetProfile,quoteType,fundProfile");
        then.status(200)
            .header("content-type", "application/json")
            .body(crate::common::fixture(
                "profile_api_assetProfile-quoteType-fundProfile",
                sym,
                "json",
            ));
    });

    // price_target — no AAPL-specific fixture; MSFT fixture is structurally identical
    server.mock(|when, then| {
        when.method(GET)
            .path(format!("/v10/finance/quoteSummary/{sym}"))
            .query_param("modules", "financialData");
        then.status(200)
            .header("content-type", "application/json")
            .body(crate::common::fixture("analysis_api_financialData", "MSFT", "json"));
    });

    // recommendations_summary — no AAPL-specific fixture; MSFT fixture is structurally identical
    server.mock(|when, then| {
        when.method(GET)
            .path(format!("/v10/finance/quoteSummary/{sym}"))
            .query_param("modules", "recommendationTrend,financialData");
        then.status(200)
            .header("content-type", "application/json")
            .body(crate::common::fixture(
                "analysis_api_recommendationTrend-financialData",
                "MSFT",
                "json",
            ));
    });

    // esg — no AAPL-specific fixture; MSFT fixture is structurally identical
    server.mock(|when, then| {
        when.method(GET)
            .path(format!("/v10/finance/quoteSummary/{sym}"))
            .query_param("modules", "esgScores");
        then.status(200)
            .header("content-type", "application/json")
            .body(crate::common::fixture("esg_api_esgScores", "MSFT", "json"));
    });

    // defaultKeyStatistics — the fixture under test
    let key_stats_mock = server.mock(|when, then| {
        when.method(GET)
            .path(format!("/v10/finance/quoteSummary/{sym}"))
            .query_param("modules", "defaultKeyStatistics");
        then.status(200)
            .header("content-type", "application/json")
            .body(crate::common::fixture(
                "key_statistics_api_defaultKeyStatistics",
                sym,
                "json",
            ));
    });

    let client = YfClient::builder()
        .base_quote_v7(Url::parse(&format!("{}/v7/finance/quote", server.base_url())).unwrap())
        .base_quote_api(
            Url::parse(&format!("{}/v10/finance/quoteSummary/", server.base_url())).unwrap(),
        )
        ._api_preference(ApiPreference::ApiOnly)
        ._preauth("cookie", crumb)
        .build()
        .unwrap();

    let ticker = Ticker::new(&client, sym);
    let info = ticker.info().await.unwrap();

    key_stats_mock.assert();
    assert_eq!(
        info.beta,
        Decimal::try_from(1.2_f64).ok(),
        "Info.beta should be populated from defaultKeyStatistics"
    );
}
