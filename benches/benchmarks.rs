use criterion::{Criterion, Throughput, black_box, criterion_group, criterion_main};
use std::sync::Arc;
use std::time::Duration;
use wad_rs::WadReader;

const WAD_DATA: &[u8] = include_bytes!("../assets/wad/freedoom1.wad").as_slice();

fn bench_wad_from_bytes(b: &mut Criterion) {
    let wad_data: Arc<[u8]> = Arc::from(WAD_DATA);

    let wad = WadReader::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_data)).unwrap();
    let num_lumps = wad.get_header().num_lumps as usize;

    let mut group = b.benchmark_group("Wad from_bytes");
    group.throughput(Throughput::Elements(num_lumps as u64));
    group.sample_size(100);
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("wad_from_bytes", |b| {
        b.iter(|| {
            let wad =
                WadReader::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_data)).unwrap();
            let mut names = Vec::with_capacity(wad.get_header().num_lumps as usize * 16);
            let dir_iter = wad.get_directory().iter();
            names.extend(dir_iter.map(|e| e.name(Arc::clone(&wad_data))));

            black_box(names);
        })
    });

    group.bench_function("index_lumps", |b| {
        b.iter(|| {
            let mut wad =
                WadReader::from_bytes("freedoom1.wad".to_string(), Arc::clone(&wad_data)).unwrap();
            wad.index_lumps();
            black_box(&wad.maps);
        })
    });
    group.finish();
}

criterion_group!(benches, bench_wad_from_bytes);
criterion_main!(benches);
