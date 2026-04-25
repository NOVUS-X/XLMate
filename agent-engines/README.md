# agent-engines

GPU-oriented engine infrastructure for the XLMate chess platform. The module provides an asyncio-based worker pool that wraps UCI-compatible engines, with first-class support for Leela Chess Zero (`lc0`) and CPU fallback support for Stockfish.

## Overview

The GPU worker subsystem is designed for long-running analysis services where neural-network inference should stay close to a dedicated GPU while requests are dispatched through a pool abstraction.

```text
Client/API
   |
   v
AnalysisRequest
   |
   v
WorkerPool ------------------> ResourceMonitor
   |
   +--> GPUAnalysisWorker #1 --> AsyncUciBridge --> lc0 / stockfish
   |
   +--> GPUAnalysisWorker #2 --> AsyncUciBridge --> lc0 / stockfish
   |
   +--> BatchAnalyzer (optional request coalescing)
```

## Package layout

- `gpu_worker/config.py`: worker and GPU configuration models.
- `gpu_worker/models.py`: request, result, and status models.
- `gpu_worker/anomaly.py`: passive bot-farm anomaly detection over request telemetry.
- `gpu_worker/uci_bridge.py`: async UCI subprocess bridge and protocol parsing.
- `gpu_worker/worker.py`: single-worker lifecycle and analysis orchestration.
- `gpu_worker/pool.py`: least-loaded worker dispatch.
- `gpu_worker/batch.py`: time- and size-based batching layer.
- `gpu_worker/resource_monitor.py`: CPU/GPU monitoring with graceful fallback.

## Requirements

- Python 3.11+
- A UCI-compliant engine binary
- For GPU acceleration with `lc0`:
  - NVIDIA drivers and CUDA/cuDNN or another supported backend
  - Optional `pynvml` installation for detailed GPU metrics
  - Leela network weights (`.pb.gz` / `.onnx`) when required by the selected engine build

## Installation

```bash
cd agent-engines
python3 -m venv .venv
source .venv/bin/activate
pip install -e .[dev]
```

For NVIDIA monitoring:

```bash
pip install -e .[gpu,dev]
```

## Configuration

`WorkerConfig` controls engine selection, defaults, and GPU tuning.

```python
from gpu_worker.config import EngineBackend, GPUConfig, WorkerConfig

config = WorkerConfig(
    engine_backend=EngineBackend.LC0,
    engine_path="/usr/local/bin/lc0",
    gpu=GPUConfig(device_id=0, max_batch_size=32, memory_limit_mb=2048, backend="cudnn"),
    default_depth=22,
    default_time_limit_ms=3000,
    threads=2,
    hash_size_mb=512,
    network_weights_path="/models/bt4.pb.gz",
)
```

Key options:

- `engine_backend`: `lc0` or `stockfish`
- `engine_path`: path to the engine binary
- `gpu.device_id`: GPU index to bind the worker to
- `gpu.backend`: GPU backend such as `cudnn`, `cuda`, or `opencl`
- `max_concurrent_analyses`: queued work allowed per worker
- `default_depth` / `default_time_limit_ms`: search defaults when requests omit limits
- `threads`: engine thread count
- `hash_size_mb`: cache/hash allocation
- `network_weights_path`: lc0 weights file path

## Usage

### Start a worker pool

```bash
python main.py
```

### Submit a single request

```python
import asyncio

from gpu_worker.config import WorkerConfig
from gpu_worker.models import AnalysisRequest
from gpu_worker.pool import WorkerPool


async def run() -> None:
    pool = WorkerPool([WorkerConfig()])
    await pool.start_all()
    try:
        result = await pool.submit(
            AnalysisRequest(
                fen="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                depth=18,
            )
        )
        print(result.model_dump())
    finally:
        await pool.shutdown_all()


asyncio.run(run())
```

### Batch analysis

```python
import asyncio

from gpu_worker.batch import BatchAnalyzer
from gpu_worker.config import WorkerConfig
from gpu_worker.models import AnalysisRequest
from gpu_worker.pool import WorkerPool


async def run() -> None:
    pool = WorkerPool([WorkerConfig(), WorkerConfig(gpu={"device_id": 1})])
    await pool.start_all()
    analyzer = BatchAnalyzer(pool, batch_size=8, flush_interval_ms=50)
    try:
        results = await analyzer.submit_batch(
            [
                AnalysisRequest(
                    fen="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                    priority=10,
                )
            ]
        )
        print([result.best_move for result in results])
    finally:
        await analyzer.shutdown()
        await pool.shutdown_all()


asyncio.run(run())
```

### Bot-farm anomaly detection

`BotFarmAnomalyDetector` passively scores request telemetry before or after dispatch; it never blocks `WorkerPool.submit()`.

```python
from gpu_worker.anomaly import BotFarmAnomalyDetector
from gpu_worker.models import AnalysisRequest


detector = BotFarmAnomalyDetector()
report = detector.record_request(
    AnalysisRequest(
        fen="rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        depth=18,
        actor_id="user-42",
        session_id="session-abc",
        ip_hash="sha256:ip-hash",
        device_hash="sha256:device-hash",
    )
)
print(report.risk_level, report.score)
```

## Testing

The test suite uses mocked UCI processes and does not require `lc0` or Stockfish to be installed.

```bash
pytest
```

## Notes

- The UCI bridge works with any UCI-compatible engine and only applies `setoption` calls for engine options reported during the `uci` handshake.
- GPU monitoring gracefully degrades to empty metrics when NVML or `nvidia-smi` are unavailable.
- `BatchAnalyzer` improves throughput for bursty workloads such as review pipelines or offline game processing.
