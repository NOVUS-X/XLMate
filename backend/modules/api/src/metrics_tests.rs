#[cfg(test)]
mod tests {
    use crate::metrics::{Metrics, init_metrics};
    use prometheus::Encoder;

    #[test]
    fn test_metrics_initialization() {
        // Initialize metrics
        let metrics = init_metrics();
        
        // Verify metrics instance is created
        assert!(metrics.active_games.get() >= 0.0);
        assert!(metrics.ws_connections.get() >= 0.0);
    }

    #[test]
    fn test_active_games_increment_decrement() {
        let metrics = init_metrics();
        
        let initial_value = metrics.active_games.get();
        
        // Increment
        metrics.active_games.inc();
        assert_eq!(metrics.active_games.get(), initial_value + 1.0);
        
        // Decrement
        metrics.active_games.dec();
        assert_eq!(metrics.active_games.get(), initial_value);
    }

    #[test]
    fn test_ws_connections_increment_decrement() {
        let metrics = init_metrics();
        
        let initial_value = metrics.ws_connections.get();
        
        // Increment
        metrics.ws_connections.inc();
        assert_eq!(metrics.ws_connections.get(), initial_value + 1.0);
        
        // Decrement
        metrics.ws_connections.dec();
        assert_eq!(metrics.ws_connections.get(), initial_value);
    }

    #[test]
    fn test_ws_connections_set() {
        let metrics = init_metrics();
        
        // Set to specific value
        metrics.ws_connections.set(42.0);
        assert_eq!(metrics.ws_connections.get(), 42.0);
        
        // Set to another value
        metrics.ws_connections.set(10.0);
        assert_eq!(metrics.ws_connections.get(), 10.0);
    }

    #[test]
    fn test_db_query_duration_histogram() {
        let metrics = init_metrics();
        
        // Observe some durations
        metrics.db_query_duration.observe(0.05);
        metrics.db_query_duration.observe(0.1);
        metrics.db_query_duration.observe(0.15);
        
        // Verify the histogram has samples
        let metric_protos = metrics.db_query_duration.collect();
        assert!(!metric_protos.is_empty());
    }

    #[test]
    fn test_counters_increment() {
        let metrics = init_metrics();
        
        let initial_ai = metrics.ai_requests_total.get();
        let initial_auth = metrics.auth_events_total.get();
        let initial_game = metrics.game_events_total.get();
        
        // Increment counters
        metrics.ai_requests_total.inc();
        metrics.auth_events_total.inc();
        metrics.game_events_total.inc();
        
        assert_eq!(metrics.ai_requests_total.get(), initial_ai + 1.0);
        assert_eq!(metrics.auth_events_total.get(), initial_auth + 1.0);
        assert_eq!(metrics.game_events_total.get(), initial_game + 1.0);
    }

    #[test]
    fn test_matchmaking_queue_operations() {
        let metrics = init_metrics();
        
        let initial_value = metrics.matchmaking_queue_size.get();
        
        // Increment queue
        metrics.matchmaking_queue_size.inc();
        assert_eq!(metrics.matchmaking_queue_size.get(), initial_value + 1.0);
        
        // Set to specific value
        metrics.matchmaking_queue_size.set(15.0);
        assert_eq!(metrics.matchmaking_queue_size.get(), 15.0);
        
        // Decrement queue
        metrics.matchmaking_queue_size.dec();
        assert_eq!(metrics.matchmaking_queue_size.get(), 14.0);
    }

    #[test]
    fn test_metrics_registry_gather() {
        let metrics = init_metrics();
        
        // Gather metrics from registry
        let registry = Metrics::registry();
        let metric_families = registry.gather();
        
        // Verify we have metrics registered
        assert!(!metric_families.is_empty());
        
        // Should have at least our 7 custom metrics
        assert!(metric_families.len() >= 7);
    }

    #[test]
    fn test_metrics_encoding() {
        let metrics = init_metrics();
        
        // Perform some operations
        metrics.active_games.inc();
        metrics.ws_connections.set(5.0);
        metrics.ai_requests_total.inc();
        
        // Encode metrics
        let encoder = prometheus::TextEncoder::new();
        let registry = Metrics::registry();
        let metric_families = registry.gather();
        let mut buffer = Vec::new();
        
        let result = encoder.encode(&metric_families, &mut buffer);
        
        // Encoding should succeed
        assert!(result.is_ok());
        
        // Buffer should contain metric data
        assert!(!buffer.is_empty());
        
        // Convert to string and verify format
        let output = String::from_utf8(buffer).unwrap();
        assert!(output.contains("xlmate_active_games"));
        assert!(output.contains("xlmate_ws_connections"));
        assert!(output.contains("xlmate_ai_requests_total"));
    }

    #[test]
    fn test_concurrent_metric_updates() {
        use std::thread;
        use std::sync::Arc;
        
        let metrics = init_metrics();
        let metrics_arc = Arc::new(metrics);
        
        let mut handles = vec![];
        
        // Spawn multiple threads to increment metrics
        for _ in 0..10 {
            let metrics_clone = metrics_arc.clone();
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    metrics_clone.active_games.inc();
                    metrics_clone.ai_requests_total.inc();
                }
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Verify final values
        assert_eq!(metrics_arc.active_games.get(), 1000.0); // 10 threads * 100 increments
        assert_eq!(metrics_arc.ai_requests_total.get(), 1000.0);
    }
}
