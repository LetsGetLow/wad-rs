use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::sync::Arc;
use std::time::Duration;
use wad_rs::WadIndex;

const WAD_DATA: &[u8] = include_bytes!("../assets/wad/freedoom1.wad").as_slice();

fn bench_wad_from_bytes(b: &mut Criterion) {
    let wad_data: Arc<[u8]> = Arc::from(WAD_DATA);

    let wad = WadIndex::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_data)).unwrap();

    let mut group = b.benchmark_group("Wad from_bytes");
    group.throughput(Throughput::Bytes(wad_data.len() as u64));
    group.sample_size(100);

    group.bench_function("index_lumps", |b| {
        b.iter(|| {
            WadIndex::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_data)).unwrap();
        })
    });
    group.finish();
}

criterion_group!(benches, bench_wad_from_bytes);
criterion_main!(benches);
