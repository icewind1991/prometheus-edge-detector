use main_error::MainError;
use prometheus_edge_detector::EdgeDetector;
use tokio::time::Duration;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let edge_detector = EdgeDetector::new("http://example.com");
    let edge = edge_detector
        .get_last_edge("prometheus_value", 1, 0, Duration::from_secs(60 * 60))
        .await?;

    if let Some(edge_time) = edge {
        print!("Last dropping edge: {}", edge_time);
    } else {
        println!("Query doesn't end with dropping edge");
    }

    Ok(())
}
