// #[query(hidden = true)]
// fn http_request(request: HttpRequest) -> HttpResponse {
//     fn get_logs_impl(since: Option<TimestampMillis>) -> HttpResponse {
//         encode_logs(canister_logger::export_logs(), since.unwrap_or(0))
//     }

//     fn get_traces_impl(since: Option<TimestampMillis>) -> HttpResponse {
//         encode_logs(canister_logger::export_traces(), since.unwrap_or(0))
//     }

//     fn get_metrics_impl(state: &RuntimeState) -> HttpResponse {
//         build_json_response(&state.metrics())
//     }

//     match extract_route(&request.url) {
//         Route::Logs(since) => get_logs_impl(since),
//         Route::Traces(since) => get_traces_impl(since),
//         Route::Metrics => read_state(get_metrics_impl),
//         _ => HttpResponse::not_found(),
//     }
// }
