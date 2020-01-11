use err_derive::Error;
use reqwest::Client;
use serde::Deserialize;
use std::time::SystemTime;
use tokio::time::Duration;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display = "Network error: {}", _0)]
    Network(reqwest::Error),
    #[error(display = "Malformed json response: {}", _0)]
    MalformedResponse(reqwest::Error),
    #[error(display = "Data point is not an interger: {}", _0)]
    NonNumericDataPoint(String),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryResultStatus {
    Success,
    Error,
}

#[derive(Debug, Clone, Deserialize)]
struct QueryResult {
    status: QueryResultStatus,
    data: QueryResultData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum QueryResultDataType {
    Matrix,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueryResultData {
    result_type: QueryResultDataType,
    result: Vec<QueryResultDataResult>,
}

#[derive(Debug, Clone, Deserialize)]
struct QueryResultDataResult {
    metric: QueryResultDataResultMetric,
    values: Vec<QueryResultDataResultValue>,
}

#[derive(Debug, Clone, Deserialize)]
struct QueryResultDataResultMetric {
    #[serde(rename = "__name__")]
    id: String,
    instance: String,
    job: String,
}

#[derive(Debug, Clone, Deserialize)]
struct QueryResultDataResultValue(u64, String);

async fn query_prometheus(
    client: &Client,
    base_url: &str,
    query: &str,
    start: u64,
    end: u64,
    step: usize,
) -> Result<QueryResult, Error> {
    client
        .get(base_url)
        .query(&[
            ("query", query),
            ("start", &format!("{}", start)),
            ("end", &format!("{}", end)),
            ("step", &format!("{}", step)),
        ])
        .send()
        .await
        .map_err(|err| Error::Network(err))?
        .json()
        .await
        .map_err(|err| Error::MalformedResponse(err))
}

#[derive(Debug, Clone)]
pub struct EdgeDetector {
    client: Client,
    base_url: String,
}

impl EdgeDetector {
    pub fn new(prometheus_url: impl std::fmt::Display) -> Self {
        EdgeDetector {
            client: Client::new(),
            base_url: format!("{}/api/v1/query_range", prometheus_url),
        }
    }

    pub async fn get_last_edge(
        &self,
        query: &str,
        from: u64,
        to: u64,
        max_age: Duration,
    ) -> Result<Option<u64>, Error> {
        let end_time = SystemTime::now();
        let start_time = end_time - max_age;

        let end_time = end_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let start_time = start_time
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.get_edge_between(query, from, to, start_time, end_time)
            .await
    }

    pub async fn get_edge_between(
        &self,
        query: &str,
        from: u64,
        to: u64,
        start_time: u64,
        end_time: u64,
    ) -> Result<Option<u64>, Error> {
        let rising = from < to;

        let data = query_prometheus(
            &self.client,
            &self.base_url,
            query,
            start_time,
            end_time,
            60,
        )
        .await?
        .data
        .result;

        let first_data = match data.into_iter().next() {
            Some(result) => result,
            None => return Ok(None),
        };

        let mut last_from_time = 0;
        let mut last_to_time = 0;

        for point in first_data.values {
            let QueryResultDataResultValue(time, value_str) = point;
            let value: u64 = value_str
                .parse()
                .map_err(|_| Error::NonNumericDataPoint(value_str))?;

            let is_from_value = if rising { value <= from } else { value >= from };

            let is_to_value = if rising { value >= from } else { value <= from };

            if is_from_value {
                last_from_time = time;
            } else if is_to_value {
                last_to_time = time;
            }
        }

        if last_from_time > 0 && last_to_time > last_from_time {
            Ok(Some(last_from_time))
        } else {
            Ok(None)
        }
    }
}
