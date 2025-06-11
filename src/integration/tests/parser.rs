use generated::health::{AppHealthRequest, AppHealthResponse};
use generated::parser::ParseRequest;
use integration::TestArgs;

// XXX: if you're iterating on these tests and the underlying code, make sure you run `cargo build --all`.
// Otherwise, Rust will not recompile the app binaries used here.
// You can also use `make test`, which takes care of recompiling the binaries before running the tests.

#[tokio::test]
async fn parser_e2e() {
    async fn test(test_args: TestArgs) {
        let parse_request = ParseRequest {
            unsigned_payload: "unsignedpayload".to_string(),
        };

        let parse_response = test_args
            .parser_client
            .unwrap()
            .parse(tonic::Request::new(parse_request))
            .await
            .unwrap()
            .into_inner();

        let parsed_transaction = parse_response.parsed_transaction.unwrap().payload.unwrap();
        assert_eq!(
            parsed_transaction.signable_payload,
            "fill in parsed signable payload"
        );
    }

    integration::Builder::new().execute(test).await
}

#[tokio::test]
async fn parser_health_check() {
    async fn test(test_args: TestArgs) {
        let request = tonic::Request::new(AppHealthRequest {});
        let response = test_args
            .health_check_client
            .unwrap()
            .app_health(request)
            .await;
        assert_eq!(
            response.unwrap().into_inner(),
            AppHealthResponse { code: 200 }
        );
    }

    integration::Builder::new().execute(test).await
}

#[tokio::test]
async fn parser_k8_health() {
    async fn test(test_args: TestArgs) {
        integration::k8_health(test_args).await;
    }

    integration::Builder::new().execute(test).await
}
