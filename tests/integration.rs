// use axum::{Router, routing::get};
// use hyper::Request;
// use loop_sense::{self, AppState, SensorData, get_data};
// use tower::ServiceExt;
//
// #[tokio::test]
// async fn test_get_data() {
//     let state = AppState::default(); // your test state
//
//     let app = Router::new()
//         .route("/data", get(get_data))
//         .with_state(state);
//
//     let response = app
//         .oneshot(Request::builder().uri("/data").body(Body::empty()).unwrap())
//         .await
//         .unwrap();
//
//     assert_eq!(response.status(), 200);
//
//     let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
//     let json: SensorData = serde_json::from_slice(&body).unwrap();
//
//     assert_eq!(json.experiment_time, 0.0); // or whatever you expect
// }
