use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use std::rc::Rc;
use wad_rs::index::index_tokens;
use wad_rs::lump::LumpRef;
use wad_rs::tokenizer::LumpToken;
use wad_rs::WadIndex;

const WAD_DATA: &[u8] = include_bytes!("../assets/wad/freedoom1.wad").as_slice();

fn bench_wad_from_bytes(b: &mut Criterion) {
    let wad_data: Rc<[u8]>= Rc::from(WAD_DATA);

    let mut group = b.benchmark_group("Wad from_bytes");
    group.throughput(Throughput::Bytes(wad_data.len() as u64));
    group.sample_size(100);

    group.bench_function("index_lumps", |b| {
        b.iter(|| {
            WadIndex::from_bytes("freedoom1.wad".to_string(), Rc::clone(&wad_data)).unwrap();
        })
    });
    group.finish();
}


fn bench_indexing_lumps(b: &mut Criterion) {
    let tokens = vec![
        LumpToken::MarkerStart("S_START".to_string()),
        LumpToken::Lump("TEST1".to_string(), LumpRef::new(0, 0, 0)),
        LumpToken::Lump("TEST2".to_string(), LumpRef::new(0, 0, 0)),
        LumpToken::MarkerEnd("S_END".to_string()),
        LumpToken::Lump("TEST3".to_string(), LumpRef::new(0, 0, 0)),
    ];


    let mut group = b.benchmark_group("Wad indexing lumps");
    group.throughput(Throughput::Elements(tokens.len() as u64));
    group.sample_size(100);

    group.bench_function("index_lumps", |b| {
        b.iter(|| {
            index_tokens(&tokens).unwrap();
        })
    });

    group.finish();
}

criterion_group!(benches, bench_wad_from_bytes, bench_indexing_lumps);
criterion_main!(benches);
