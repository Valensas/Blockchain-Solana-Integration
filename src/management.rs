use crate::{errors::{ResponseError, Code}, models::ArcRwLockPrometheus};
use rocket::{State, serde::json::Json};
use prometheus::{Encoder, TextEncoder};

#[get("/metrics")]
pub fn metrics(
    prometheus_metrics: &State<ArcRwLockPrometheus>
) -> Result<String, ResponseError> {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    encoder
        .encode(&prometheus_metrics.rw_lock.read().unwrap().registry().gather(), &mut buffer)
        .map_err(|err|{
            log::error!("Error while getting the metric data: {}", err);
            ResponseError::PrometheusError(Json(Code{code: "GET_METRICS_ERROR".to_string()}))
        })?;
    let body = String::from_utf8(buffer.clone()).unwrap();
    Ok(body)
}