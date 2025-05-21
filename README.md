# Moved to https://codeberg.org/icewind/prometheus-edge-detector

# Prometheus edge detector

Find the most recent rising or dropping edge from a prometheus query

## Usage

```rust
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
```

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
