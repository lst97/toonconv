//! Memory optimization for large nested structures
//!
//! Provides memory-efficient processing strategies for large JSON
//! structures to prevent excessive memory usage and improve performance.

use crate::error::{FormattingError, FormattingResult};
use serde_json::Value;
use std::io::Write;

/// Memory-optimized formatter for large structures
pub struct MemoryOptimizer {
    /// Current memory usage estimate (bytes)
    current_memory: usize,

    /// Maximum allowed memory usage (bytes)
    max_memory: usize,

    /// Enable streaming mode for very large structures
    streaming_enabled: bool,
}

impl MemoryOptimizer {
    /// Create a new memory optimizer
    pub fn new(max_memory: usize) -> Self {
        Self {
            current_memory: 0,
            max_memory,
            streaming_enabled: false,
        }
    }

    /// Enable streaming mode for large structures
    pub fn enable_streaming(&mut self) {
        self.streaming_enabled = true;
    }

    /// Check if current operation would exceed memory limit
    pub fn check_memory(&self, additional_bytes: usize) -> FormattingResult<()> {
        if self.current_memory + additional_bytes > self.max_memory {
            return Err(FormattingError::invalid_structure(format!(
                "Memory limit exceeded: {} + {} > {} bytes",
                self.current_memory, additional_bytes, self.max_memory
            )));
        }
        Ok(())
    }

    /// Track memory allocation
    pub fn allocate(&mut self, bytes: usize) -> FormattingResult<()> {
        self.check_memory(bytes)?;
        self.current_memory += bytes;
        Ok(())
    }

    /// Track memory deallocation
    pub fn deallocate(&mut self, bytes: usize) {
        self.current_memory = self.current_memory.saturating_sub(bytes);
    }

    /// Estimate memory requirement for a JSON value
    pub fn estimate_size(&self, value: &Value) -> usize {
        match value {
            Value::Null => 4, // "null"
            Value::Bool(b) => {
                if *b {
                    4
                } else {
                    5
                }
            } // "true" or "false"
            Value::Number(n) => n.to_string().len(),
            Value::String(s) => s.len() + 2, // quotes
            Value::Array(arr) => {
                let mut size = 2; // []
                for item in arr {
                    size += self.estimate_size(item) + 1; // + comma
                }
                size
            }
            Value::Object(obj) => {
                let mut size = 2; // {}
                for (key, val) in obj {
                    size += key.len() + 2; // key + ": "
                    size += self.estimate_size(val) + 1; // value + comma
                }
                size
            }
        }
    }

    /// Process large structure with memory optimization
    pub fn process_large_structure<F>(
        &mut self,
        value: &Value,
        processor: F,
    ) -> FormattingResult<String>
    where
        F: Fn(&Value, &mut Self) -> FormattingResult<String>,
    {
        // Estimate required memory
        let estimated_size = self.estimate_size(value);

        // Check if we should use streaming
        if estimated_size > self.max_memory / 2 {
            self.enable_streaming();
        }

        // Pre-allocate output buffer with estimated size
        let mut output = String::with_capacity(estimated_size.min(self.max_memory));

        // Track allocation
        self.allocate(estimated_size)?;

        // Process the value
        let result = processor(value, self)?;
        output.push_str(&result);

        // Deallocate
        self.deallocate(estimated_size);

        Ok(output)
    }

    /// Get current memory usage
    pub fn current_usage(&self) -> usize {
        self.current_memory
    }

    /// Get memory limit
    pub fn memory_limit(&self) -> usize {
        self.max_memory
    }

    /// Check if memory usage is critical (>80% of limit)
    pub fn is_critical(&self) -> bool {
        self.current_memory * 100 / self.max_memory > 80
    }

    /// Reset memory tracking
    pub fn reset(&mut self) {
        self.current_memory = 0;
        self.streaming_enabled = false;
    }
}

/// Streaming writer for very large structures
pub struct StreamingWriter<W: Write> {
    writer: W,
    buffer: Vec<u8>,
    buffer_size: usize,
}

impl<W: Write> StreamingWriter<W> {
    /// Create a new streaming writer
    pub fn new(writer: W, buffer_size: usize) -> Self {
        Self {
            writer,
            buffer: Vec::with_capacity(buffer_size),
            buffer_size,
        }
    }

    /// Write data to the stream
    pub fn write(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.buffer.extend_from_slice(data);

        if self.buffer.len() >= self.buffer_size {
            self.flush()?;
        }

        Ok(())
    }

    /// Flush the buffer to the underlying writer
    pub fn flush(&mut self) -> std::io::Result<()> {
        if !self.buffer.is_empty() {
            self.writer.write_all(&self.buffer)?;
            self.buffer.clear();
        }
        self.writer.flush()
    }

    /// Write a string
    pub fn write_str(&mut self, s: &str) -> std::io::Result<()> {
        self.write(s.as_bytes())
    }
}

impl<W: Write> Drop for StreamingWriter<W> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

/// Chunk processor for processing arrays in chunks
pub struct ChunkProcessor {
    chunk_size: usize,
}

impl ChunkProcessor {
    /// Create a new chunk processor
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    /// Process an array in chunks
    pub fn process_array<F>(
        &self,
        array: &[Value],
        mut processor: F,
    ) -> FormattingResult<Vec<String>>
    where
        F: FnMut(&[Value]) -> FormattingResult<String>,
    {
        let mut results = Vec::new();

        for chunk in array.chunks(self.chunk_size) {
            let result = processor(chunk)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Get chunk size
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }
}

/// Memory pool for reusing allocations
pub struct MemoryPool {
    /// Pool of reusable string buffers
    string_buffers: Vec<String>,

    /// Maximum number of buffers to keep
    max_buffers: usize,
}

impl MemoryPool {
    /// Create a new memory pool
    pub fn new(max_buffers: usize) -> Self {
        Self {
            string_buffers: Vec::with_capacity(max_buffers),
            max_buffers,
        }
    }

    /// Acquire a string buffer from the pool
    pub fn acquire_string(&mut self, capacity: usize) -> String {
        if let Some(mut buffer) = self.string_buffers.pop() {
            buffer.clear();
            if buffer.capacity() < capacity {
                buffer.reserve(capacity - buffer.capacity());
            }
            buffer
        } else {
            String::with_capacity(capacity)
        }
    }

    /// Return a string buffer to the pool
    pub fn release_string(&mut self, buffer: String) {
        if self.string_buffers.len() < self.max_buffers {
            self.string_buffers.push(buffer);
        }
    }

    /// Clear the pool
    pub fn clear(&mut self) {
        self.string_buffers.clear();
    }

    /// Get pool size
    pub fn pool_size(&self) -> usize {
        self.string_buffers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_memory_optimizer_basic() {
        let mut optimizer = MemoryOptimizer::new(1024);

        assert_eq!(optimizer.current_usage(), 0);
        assert!(optimizer.allocate(512).is_ok());
        assert_eq!(optimizer.current_usage(), 512);

        optimizer.deallocate(256);
        assert_eq!(optimizer.current_usage(), 256);
    }

    #[test]
    fn test_memory_limit_exceeded() {
        let mut optimizer = MemoryOptimizer::new(100);

        optimizer.allocate(50).unwrap();
        let result = optimizer.allocate(100);

        assert!(result.is_err());
    }

    #[test]
    fn test_size_estimation() {
        let optimizer = MemoryOptimizer::new(1024);

        assert_eq!(optimizer.estimate_size(&json!(null)), 4);
        assert_eq!(optimizer.estimate_size(&json!(true)), 4);
        assert_eq!(optimizer.estimate_size(&json!(false)), 5);

        let size = optimizer.estimate_size(&json!({"key": "value"}));
        assert!(size > 0);
    }

    #[test]
    fn test_critical_memory_check() {
        let mut optimizer = MemoryOptimizer::new(100);

        assert!(!optimizer.is_critical());

        optimizer.allocate(85).unwrap();
        assert!(optimizer.is_critical());
    }

    #[test]
    fn test_memory_reset() {
        let mut optimizer = MemoryOptimizer::new(1024);

        optimizer.allocate(512).unwrap();
        assert_eq!(optimizer.current_usage(), 512);

        optimizer.reset();
        assert_eq!(optimizer.current_usage(), 0);
    }

    #[test]
    fn test_streaming_writer() {
        let mut buffer = Vec::new();
        {
            let mut writer = StreamingWriter::new(&mut buffer, 16);

            writer.write_str("Hello").unwrap();
            writer.write_str(" ").unwrap();
            writer.write_str("World").unwrap();
            writer.flush().unwrap();
        } // writer dropped here

        assert_eq!(String::from_utf8(buffer).unwrap(), "Hello World");
    }

    #[test]
    fn test_streaming_writer_auto_flush() {
        let mut buffer = Vec::new();
        {
            let mut writer = StreamingWriter::new(&mut buffer, 5);

            writer.write_str("12345").unwrap(); // Should auto-flush at buffer size
            writer.write_str("67890").unwrap();
            writer.flush().unwrap();
        } // writer dropped here

        assert_eq!(String::from_utf8(buffer).unwrap(), "1234567890");
    }

    #[test]
    fn test_chunk_processor() {
        let processor = ChunkProcessor::new(3);
        let array = vec![json!(1), json!(2), json!(3), json!(4), json!(5)];

        let results = processor
            .process_array(&array, |chunk| Ok(format!("[{}]", chunk.len())))
            .unwrap();

        assert_eq!(results, vec!["[3]", "[2]"]);
    }

    #[test]
    fn test_memory_pool() {
        let mut pool = MemoryPool::new(5);

        let buf1 = pool.acquire_string(100);
        assert_eq!(buf1.capacity(), 100);

        pool.release_string(buf1);
        assert_eq!(pool.pool_size(), 1);

        let buf2 = pool.acquire_string(50);
        assert_eq!(pool.pool_size(), 0);
        assert!(buf2.capacity() >= 100); // Reused buffer
    }

    #[test]
    fn test_memory_pool_max_size() {
        let mut pool = MemoryPool::new(2);

        for _ in 0..5 {
            let buf = String::with_capacity(100);
            pool.release_string(buf);
        }

        assert_eq!(pool.pool_size(), 2); // Max 2 buffers
    }

    #[test]
    fn test_memory_pool_clear() {
        let mut pool = MemoryPool::new(5);

        pool.release_string(String::new());
        pool.release_string(String::new());
        assert_eq!(pool.pool_size(), 2);

        pool.clear();
        assert_eq!(pool.pool_size(), 0);
    }

    #[test]
    fn test_large_structure_processing() {
        let mut optimizer = MemoryOptimizer::new(10000);

        let json = json!({
            "data": (0..100).map(|i| json!({"id": i})).collect::<Vec<_>>()
        });

        let result = optimizer.process_large_structure(&json, |val, _opt| Ok(val.to_string()));

        assert!(result.is_ok());
    }
}
