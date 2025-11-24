//! Statistics and performance tracking for conversion operations

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Performance statistics for conversion operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionStatistics {
    /// Input JSON size in bytes
    pub input_size_bytes: u64,
    /// Output TOON size in bytes
    pub output_size_bytes: u64,
    /// Token reduction percentage
    pub token_reduction_percent: f32,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Peak memory usage in bytes
    pub memory_peak_bytes: usize,
    /// Number of files processed
    pub file_count: usize,
    /// Number of conversion operations
    pub operation_count: usize,
    /// Average time per operation
    pub avg_time_per_operation_ms: f32,
    /// Throughput (bytes processed per second)
    pub throughput_bytes_per_sec: f32,
    /// Timestamp of when statistics were collected
    pub collected_at: chrono::DateTime<chrono::Utc>,
}

impl Default for ConversionStatistics {
    fn default() -> Self {
        Self {
            input_size_bytes: 0,
            output_size_bytes: 0,
            token_reduction_percent: 0.0,
            processing_time_ms: 0,
            memory_peak_bytes: 0,
            file_count: 0,
            operation_count: 0,
            avg_time_per_operation_ms: 0.0,
            throughput_bytes_per_sec: 0.0,
            collected_at: chrono::Utc::now(),
        }
    }
}

impl ConversionStatistics {
    /// Create new empty statistics
    pub fn new() -> Self {
        Self::default()
    }

    /// Create statistics for a single conversion
    pub fn for_conversion(
        input_size: u64,
        output_size: u64,
        processing_time: Duration,
        memory_peak: usize,
    ) -> Self {
        let processing_time_ms = processing_time.as_millis() as u64;
        let token_reduction_percent = if input_size > 0 {
            ((input_size as f32 - output_size as f32) / input_size as f32) * 100.0
        } else {
            0.0
        };

        let throughput_bytes_per_sec = if processing_time.as_secs_f64() > 0.0 {
            input_size as f64 / processing_time.as_secs_f64()
        } else {
            0.0
        } as f32;

        Self {
            input_size_bytes: input_size,
            output_size_bytes: output_size,
            token_reduction_percent: token_reduction_percent.max(0.0),
            processing_time_ms,
            memory_peak_bytes: memory_peak,
            file_count: 1,
            operation_count: 1,
            avg_time_per_operation_ms: processing_time_ms as f32,
            throughput_bytes_per_sec,
            collected_at: chrono::Utc::now(),
        }
    }

    /// Combine statistics from multiple operations
    pub fn combine(&mut self, other: &Self) {
        self.input_size_bytes += other.input_size_bytes;
        self.output_size_bytes += other.output_size_bytes;
        self.memory_peak_bytes = self.memory_peak_bytes.max(other.memory_peak_bytes);
        self.file_count += other.file_count;
        self.operation_count += other.operation_count;
        self.processing_time_ms += other.processing_time_ms;

        // Recalculate derived metrics
        self.token_reduction_percent = if self.input_size_bytes > 0 {
            ((self.input_size_bytes as f32 - self.output_size_bytes as f32) 
                / self.input_size_bytes as f32) * 100.0
        } else {
            0.0
        }.max(0.0);

        self.avg_time_per_operation_ms = if self.operation_count > 0 {
            self.processing_time_ms as f32 / self.operation_count as f32
        } else {
            0.0
        };

        self.throughput_bytes_per_sec = if self.processing_time_ms > 0 {
            self.input_size_bytes as f32 / (self.processing_time_ms as f32 / 1000.0)
        } else {
            0.0
        };

        self.collected_at = chrono::Utc::now();
    }

    /// Get the efficiency score (0-100, higher is better)
    pub fn efficiency_score(&self) -> f32 {
        let mut score = 0.0;

        // Token reduction contributes up to 40 points
        score += (self.token_reduction_percent / 100.0).min(0.4) * 100.0;

        // Speed contributes up to 30 points (faster = better)
        if self.avg_time_per_operation_ms > 0.0 {
            let speed_score = (1000.0 / self.avg_time_per_operation_ms).min(3.0) / 3.0;
            score += speed_score * 30.0;
        }

        // Memory efficiency contributes up to 30 points
        if self.memory_peak_bytes > 0 {
            let memory_score = (1_000_000.0 / self.memory_peak_bytes as f32).min(1.0);
            score += memory_score * 30.0;
        }

        score.min(100.0)
    }

    /// Check if performance meets targets
    pub fn meets_targets(&self, targets: &PerformanceTargets) -> PerformanceCheck {
        let mut passed = vec![];
        let mut failed = vec![];

        // Check processing time
        if self.avg_time_per_operation_ms <= targets.max_avg_time_ms {
            passed.push(format!("Processing time: {:.1}ms", self.avg_time_per_operation_ms));
        } else {
            failed.push(format!(
                "Processing time: {:.1}ms (target: {:.1}ms)",
                self.avg_time_per_operation_ms, targets.max_avg_time_ms
            ));
        }

        // Check memory usage
        if self.memory_peak_bytes <= targets.max_memory_bytes {
            passed.push(format!("Memory usage: {} bytes", self.memory_peak_bytes));
        } else {
            failed.push(format!(
                "Memory usage: {} bytes (target: {} bytes)",
                self.memory_peak_bytes, targets.max_memory_bytes
            ));
        }

        // Check token reduction
        if self.token_reduction_percent >= targets.min_token_reduction {
            passed.push(format!("Token reduction: {:.1}%", self.token_reduction_percent));
        } else {
            failed.push(format!(
                "Token reduction: {:.1}% (target: {:.1}%)",
                self.token_reduction_percent, targets.min_token_reduction
            ));
        }

        PerformanceCheck { passed, failed }
    }

    /// Get a formatted summary
    pub fn summary(&self) -> String {
        format!(
            "Processed {} files in {:.1}s - {:.1}% token reduction, {:.1}MB/s throughput",
            self.file_count,
            self.processing_time_ms as f32 / 1000.0,
            self.token_reduction_percent,
            self.throughput_bytes_per_sec / (1024.0 * 1024.0)
        )
    }

    /// Export to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Performance targets for comparison
#[derive(Debug, Clone)]
pub struct PerformanceTargets {
    pub max_avg_time_ms: f32,
    pub max_memory_bytes: usize,
    pub min_token_reduction: f32,
    pub min_throughput_mbps: f32,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            max_avg_time_ms: 1000.0, // 1 second per operation
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            min_token_reduction: 20.0, // 20% token reduction
            min_throughput_mbps: 1.0, // 1MB/s minimum
        }
    }
}

/// Result of performance target checking
#[derive(Debug, Clone)]
pub struct PerformanceCheck {
    pub passed: Vec<String>,
    pub failed: Vec<String>,
}

impl PerformanceCheck {
    pub fn is_success(&self) -> bool {
        self.failed.is_empty()
    }

    pub fn summary(&self) -> String {
        if self.is_success() {
            "All performance targets met".to_string()
        } else {
            format!("{} targets met, {} failed", self.passed.len(), self.failed.len())
        }
    }
}

/// Performance tracker for conversion operations
pub struct PerformanceTracker {
    start_time: Instant,
    memory_start: usize,
    stats: ConversionStatistics,
}

impl PerformanceTracker {
    /// Start tracking a new conversion operation
    pub fn start() -> Self {
        Self {
            start_time: Instant::now(),
            memory_start: Self::get_current_memory(),
            stats: ConversionStatistics::new(),
        }
    }

    /// Complete tracking and return statistics
    pub fn finish(
        mut self,
        input_size: u64,
        output_size: u64,
        memory_peak: usize,
    ) -> ConversionStatistics {
        let processing_time = self.start_time.elapsed();
        
        let operation_stats = ConversionStatistics::for_conversion(
            input_size,
            output_size,
            processing_time,
            memory_peak,
        );

        self.stats.combine(&operation_stats);
        self.stats
    }

    /// Get current memory usage (simplified - in real implementation would be more sophisticated)
    fn get_current_memory() -> usize {
        // This is a placeholder - in a real implementation,
        // you would use platform-specific APIs to get actual memory usage
        0
    }
}

/// Benchmark results for performance testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub test_name: String,
    pub input_size: u64,
    pub output_size: u64,
    pub iterations: usize,
    pub total_time_ms: u64,
    pub avg_time_ms: f32,
    pub min_time_ms: u64,
    pub max_time_ms: u64,
    pub std_deviation_ms: f32,
    pub memory_peak: usize,
    pub throughput_mbps: f32,
}

impl BenchmarkResults {
    /// Create benchmark results from raw data
    pub fn new(
        test_name: String,
        input_size: u64,
        output_size: u64,
        iterations: usize,
        times_ms: Vec<u64>,
        memory_peak: usize,
    ) -> Self {
        let total_time_ms = times_ms.iter().sum();
        let avg_time_ms = total_time_ms as f32 / iterations as f32;
        
        let min_time_ms = times_ms.iter().min().copied().unwrap_or(0);
        let max_time_ms = times_ms.iter().max().copied().unwrap_or(0);
        
        // Calculate standard deviation
        let variance = times_ms.iter()
            .map(|&t| {
                let diff = t as f32 - avg_time_ms;
                diff * diff
            })
            .sum::<f32>() / iterations as f32;
        let std_deviation_ms = variance.sqrt();

        let total_time_sec = total_time_ms as f32 / 1000.0;
        let throughput_mbps = if total_time_sec > 0.0 {
            (input_size as f64 / (1024.0 * 1024.0)) / total_time_sec as f64
        } else {
            0.0
        } as f32;

        Self {
            test_name,
            input_size,
            output_size,
            iterations,
            total_time_ms,
            avg_time_ms,
            min_time_ms,
            max_time_ms,
            std_deviation_ms,
            memory_peak,
            throughput_mbps,
        }
    }

    /// Get formatted benchmark report
    pub fn report(&self) -> String {
        format!(
            "Benchmark: {}\n\
             Input: {} bytes, Output: {} bytes\n\
             Iterations: {}\n\
             Timing: {:.2}ms ± {:.2}ms (min: {}ms, max: {}ms)\n\
             Throughput: {:.2} MB/s\n\
             Memory peak: {} bytes",
            self.test_name,
            self.input_size,
            self.output_size,
            self.iterations,
            self.avg_time_ms,
            self.std_deviation_ms,
            self.min_time_ms,
            self.max_time_ms,
            self.throughput_mbps,
            self.memory_peak
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_statistics_creation() {
        let stats = ConversionStatistics::for_conversion(
            1000,
            600,
            Duration::from_millis(100),
            1024 * 1024,
        );

        assert_eq!(stats.input_size_bytes, 1000);
        assert_eq!(stats.output_size_bytes, 600);
        assert_eq!(stats.token_reduction_percent, 40.0);
        assert_eq!(stats.processing_time_ms, 100);
        assert_eq!(stats.file_count, 1);
        assert_eq!(stats.operation_count, 1);
    }

    #[test]
    fn test_statistics_combination() {
        let mut stats1 = ConversionStatistics::for_conversion(1000, 600, Duration::from_millis(100), 1024 * 1024);
        let stats2 = ConversionStatistics::for_conversion(2000, 1200, Duration::from_millis(200), 2 * 1024 * 1024);

        stats1.combine(&stats2);

        assert_eq!(stats1.input_size_bytes, 3000);
        assert_eq!(stats1.output_size_bytes, 1800);
        assert_eq!(stats1.file_count, 2);
        assert_eq!(stats1.operation_count, 2);
        assert_eq!(stats1.processing_time_ms, 300);
    }

    #[test]
    fn test_efficiency_score() {
        let stats = ConversionStatistics::for_conversion(
            1000,
            500,
            Duration::from_millis(50),
            512 * 1024,
        );

        let score = stats.efficiency_score();
        assert!(score > 0.0);
        assert!(score <= 100.0);
    }

    #[test]
    fn test_performance_check() {
        let targets = PerformanceTargets::default();
        let stats = ConversionStatistics::for_conversion(
            1000,
            600,
            Duration::from_millis(100),
            1024 * 1024,
        );

        let check = stats.meets_targets(&targets);
        assert!(!check.passed.is_empty());
    }

    #[test]
    fn test_benchmark_results() {
        let times = vec![100, 110, 95, 105, 120];
        let results = BenchmarkResults::new(
            "Test Benchmark".to_string(),
            10000,
            6000,
            5,
            times,
            1024 * 1024,
        );

        assert_eq!(results.iterations, 5);
        assert!(results.avg_time_ms > 0.0);
        assert!(results.std_deviation_ms >= 0.0);
    }

    #[test]
    fn test_performance_tracker() {
        let tracker = PerformanceTracker::start();
        
        // Simulate some work
        thread::sleep(Duration::from_millis(10));
        
        let stats = tracker.finish(1000, 600, 1024 * 1024);
        
        assert!(stats.processing_time_ms >= 10);
        assert_eq!(stats.input_size_bytes, 1000);
        assert_eq!(stats.output_size_bytes, 600);
    }
}
