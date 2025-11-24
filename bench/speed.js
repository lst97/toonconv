#!/usr/bin/env node
/**
 * JavaScript Speed Benchmark
 *
 * Compare with Rust: cargo bench --bench speed_comparison
 *
 * This script requires the official TOON library:
 *   npm install @toon-format/toon
 *
 * Run: node bench/speed.js
 */

// Check if @toon-format/toon is installed
let encode;
try {
  const toon = require('@toon-format/toon');
  encode = toon.encode;
} catch (e) {
  console.error('Error: @toon-format/toon not installed.');
  console.error('Install with: npm install @toon-format/toon');
  console.error('');
  console.error('Alternative: Clone the official repo:');
  console.error('  git clone https://github.com/toon-format/toon.git');
  console.error('  cd toon && npm install && npm run bench');
  process.exit(1);
}

// Data generators matching Rust benchmarks
function generateEmployeeRecords(count) {
  const departments = ['Engineering', 'Sales', 'Marketing', 'HR'];
  const employees = [];
  for (let i = 0; i < count; i++) {
    employees.push({
      id: i + 1,
      name: `Employee ${i + 1}`,
      email: `emp${i + 1}@example.com`,
      department: departments[i % 4],
      salary: 50000 + (i * 1000),
      yearsExperience: (i % 20) + 1,
      active: i % 3 !== 0
    });
  }
  return { employees };
}

function generateEcommerceOrders(count) {
  const orders = [];
  for (let i = 0; i < count; i++) {
    orders.push({
      order_id: `ORD-${String(i).padStart(5, '0')}`,
      customer: {
        id: `CUST-${String(i).padStart(4, '0')}`,
        name: `Customer ${i}`,
        email: `customer${i}@example.com`,
        address: {
          street: `${i} Main St`,
          city: 'New York',
          zip: '10001',
          country: 'USA'
        }
      },
      items: [
        { product_id: 'PROD-1', name: 'Widget A', quantity: 2, price: 19.99 },
        { product_id: 'PROD-2', name: 'Widget B', quantity: 1, price: 29.99 }
      ],
      status: 'shipped',
      created_at: '2023-10-27T10:00:00Z',
      total: 69.97
    });
  }
  return { orders };
}

function generateTimeSeries(days) {
  const metrics = [];
  for (let i = 0; i < days; i++) {
    metrics.push({
      date: `2025-01-${String((i % 28) + 1).padStart(2, '0')}`,
      views: 5000 + (i * 100),
      clicks: 200 + (i * 10),
      conversions: 20 + (i % 10),
      revenue: 7000.0 + (i * 50.0),
      bounceRate: 0.3 + (i * 0.01)
    });
  }
  return { metrics };
}

function generateNestedConfig() {
  return {
    app: {
      server: {
        host: '0.0.0.0',
        port: 8080,
        options: {
          timeout: 30,
          keepalive: true,
          retry: { attempts: 3, backoff: 'exponential', max_delay: 1000 }
        }
      },
      database: {
        primary: {
          host: 'db-primary',
          port: 5432,
          credentials: { username: 'admin', password_file: '/run/secrets/db_pass' },
          pool: { min: 5, max: 20, idle_timeout: 60 }
        },
        replicas: [
          { host: 'db-replica-1', port: 5432, readonly: true },
          { host: 'db-replica-2', port: 5432, readonly: true }
        ]
      },
      logging: {
        level: 'debug',
        format: 'json',
        outputs: ['stdout', 'file'],
        file: { path: '/var/log/app.log', rotation: { max_size: '100MB', max_files: 5 } }
      }
    }
  };
}

// Benchmark function
function benchmark(name, data, iterations = 1000) {
  // Warmup
  for (let i = 0; i < 10; i++) {
    encode(data);
  }

  // Measure
  const start = process.hrtime.bigint();
  for (let i = 0; i < iterations; i++) {
    encode(data);
  }
  const end = process.hrtime.bigint();

  const totalMs = Number(end - start) / 1_000_000;
  const avgMs = totalMs / iterations;
  const opsPerSec = Math.round(1000 / avgMs);

  const jsonSize = JSON.stringify(data).length;
  const throughputMBs = (jsonSize * opsPerSec) / (1024 * 1024);

  return {
    name,
    iterations,
    totalMs: totalMs.toFixed(2),
    avgMs: avgMs.toFixed(4),
    opsPerSec,
    jsonSize,
    throughputMBs: throughputMBs.toFixed(2)
  };
}

// Run benchmarks
console.log('');
console.log('╔══════════════════════════════════════════════════════════════════════════════╗');
console.log('║                    TOON JavaScript Speed Benchmark                           ║');
console.log('╠══════════════════════════════════════════════════════════════════════════════╣');
console.log('║  Compare with Rust: cargo bench --bench speed_comparison                     ║');
console.log('╚══════════════════════════════════════════════════════════════════════════════╝');
console.log('');

const results = [];

// Employee records
console.log('Benchmarking employees (100 records)...');
results.push(benchmark('employees_100', generateEmployeeRecords(100)));

console.log('Benchmarking employees (1000 records)...');
results.push(benchmark('employees_1000', generateEmployeeRecords(1000), 100));

console.log('Benchmarking employees (10000 records)...');
results.push(benchmark('employees_10000', generateEmployeeRecords(10000), 10));

// E-commerce orders
console.log('Benchmarking e-commerce orders (100 orders)...');
results.push(benchmark('ecommerce_100', generateEcommerceOrders(100)));

// Time series
console.log('Benchmarking time series (365 days)...');
results.push(benchmark('timeseries_365', generateTimeSeries(365)));

// Nested config
console.log('Benchmarking nested config...');
results.push(benchmark('nested_config', generateNestedConfig(), 5000));

// Print results
console.log('');
console.log('────────────────────────────────────────────────────────────────────────────────');
console.log('Results:');
console.log('────────────────────────────────────────────────────────────────────────────────');
console.log('');
console.log('| Benchmark          | Iterations | Total (ms) | Avg (ms) | Ops/sec | MB/s  |');
console.log('|--------------------|------------|------------|----------|---------|-------|');

for (const r of results) {
  const name = r.name.padEnd(18);
  const iter = String(r.iterations).padStart(10);
  const total = r.totalMs.padStart(10);
  const avg = r.avgMs.padStart(8);
  const ops = String(r.opsPerSec).padStart(7);
  const mbps = r.throughputMBs.padStart(5);
  console.log(`| ${name} | ${iter} | ${total} | ${avg} | ${ops} | ${mbps} |`);
}

console.log('');
console.log('────────────────────────────────────────────────────────────────────────────────');
console.log('To compare with Rust implementation:');
console.log('  cargo bench --bench speed_comparison');
console.log('');
console.log('Expected: Rust is typically 10-50x faster than JavaScript');
console.log('────────────────────────────────────────────────────────────────────────────────');
