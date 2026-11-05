use std::alloc::{GlobalAlloc, Layout, System};
use std::hint::black_box;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use markers::{ConversionOptions, convert_with_options};

struct CountingAllocator;

static ALLOC_CALLS: AtomicUsize = AtomicUsize::new(0);
static ALLOCATED_BYTES: AtomicUsize = AtomicUsize::new(0);
static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
static PEAK_LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            record_alloc(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
        LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_pointer = unsafe { System.realloc(pointer, layout, new_size) };
        if !new_pointer.is_null() {
            ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
            ALLOCATED_BYTES.fetch_add(new_size, Ordering::Relaxed);
            if new_size >= layout.size() {
                let growth = new_size - layout.size();
                let live = LIVE_BYTES.fetch_add(growth, Ordering::Relaxed) + growth;
                update_peak(live);
            } else {
                LIVE_BYTES.fetch_sub(layout.size() - new_size, Ordering::Relaxed);
            }
        }
        new_pointer
    }
}

fn record_alloc(size: usize) {
    ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
    ALLOCATED_BYTES.fetch_add(size, Ordering::Relaxed);
    let live = LIVE_BYTES.fetch_add(size, Ordering::Relaxed) + size;
    update_peak(live);
}

fn update_peak(live: usize) {
    let mut peak = PEAK_LIVE_BYTES.load(Ordering::Relaxed);
    while live > peak {
        match PEAK_LIVE_BYTES.compare_exchange_weak(
            peak,
            live,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => break,
            Err(previous) => peak = previous,
        }
    }
}

fn reset_stats() {
    ALLOC_CALLS.store(0, Ordering::Relaxed);
    ALLOCATED_BYTES.store(0, Ordering::Relaxed);
    LIVE_BYTES.store(0, Ordering::Relaxed);
    PEAK_LIVE_BYTES.store(0, Ordering::Relaxed);
}

struct Stats {
    allocations: usize,
    allocated_bytes: usize,
    peak_live_bytes: usize,
}

fn stats() -> Stats {
    Stats {
        allocations: ALLOC_CALLS.load(Ordering::Relaxed),
        allocated_bytes: ALLOCATED_BYTES.load(Ordering::Relaxed),
        peak_live_bytes: PEAK_LIVE_BYTES.load(Ordering::Relaxed),
    }
}

struct Case {
    name: &'static str,
    html: String,
    options: ConversionOptions,
}

fn cases() -> Vec<Case> {
    let selector_options =
        ConversionOptions::new().exclude_selectors(vec![".navigation".to_string()]);
    let reference_options = ConversionOptions::new().link_style(markers::LinkStyle::Referenced);

    vec![
        Case {
            name: "blog",
            html: include_str!("../tests/fixtures/blog_post.html").to_string(),
            options: ConversionOptions::default(),
        },
        Case {
            name: "documentation",
            html: include_str!("../tests/fixtures/documentation.html").to_string(),
            options: selector_options,
        },
        Case {
            name: "tables",
            html: include_str!("../tests/fixtures/tables.html").to_string(),
            options: ConversionOptions::default(),
        },
        Case {
            name: "code",
            html: include_str!("../tests/fixtures/code_heavy.html").to_string(),
            options: ConversionOptions::default(),
        },
        Case {
            name: "references",
            html: include_str!("../tests/fixtures/blog_post.html").to_string(),
            options: reference_options,
        },
        Case {
            name: "large",
            html: format!(
                "<article>{}</article>",
                include_str!("../tests/fixtures/blog_post.html").repeat(32)
            ),
            options: ConversionOptions::default(),
        },
    ]
}

fn run(case: &Case, iterations: usize) -> (Duration, Stats) {
    for _ in 0..10 {
        black_box(convert_with_options(black_box(&case.html), &case.options).unwrap());
    }

    reset_stats();
    let started = Instant::now();
    for _ in 0..iterations {
        let output = convert_with_options(black_box(&case.html), &case.options).unwrap();
        black_box(output);
    }
    (started.elapsed(), stats())
}

fn main() {
    let iterations = std::env::var("MARKERS_BENCH_ITERATIONS")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(100);

    println!(
        "case,input_bytes,output_bytes,iterations,total_ms,us_per_conversion,\
         allocations_per_conversion,allocated_bytes_per_conversion,peak_live_bytes"
    );
    for case in cases() {
        let output = convert_with_options(&case.html, &case.options).unwrap();
        let (elapsed, stats) = run(&case, iterations);
        let micros = elapsed.as_secs_f64() * 1_000_000.0 / iterations as f64;
        println!(
            "{},{},{},{},{:.3},{:.3},{:.1},{:.1},{}",
            case.name,
            case.html.len(),
            output.len(),
            iterations,
            elapsed.as_secs_f64() * 1_000.0,
            micros,
            stats.allocations as f64 / iterations as f64,
            stats.allocated_bytes as f64 / iterations as f64,
            stats.peak_live_bytes,
        );
    }
}
