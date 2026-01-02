use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use wad_rs::index::LumpNode;
use wad_rs::lump::LumpRef;
use wad_rs::WadIndex;

const WAD_DATA: &[u8] = include_bytes!("../assets/wad/freedoom1.wad").as_slice();

fn bench_wad_from_bytes(b: &mut Criterion) {
    let wad_data: &[u8] = WAD_DATA;

    let mut group = b.benchmark_group("Wad from_bytes");
    group.throughput(Throughput::Bytes(wad_data.len() as u64));
    group.sample_size(100);

    group.bench_function("index_lumps", |b| {
        b.iter(|| {
            WadIndex::from_bytes("freedoom1.wad".to_string(), &wad_data).unwrap();
        })
    });
    group.finish();
}

// fn bench_indexing_lumps(b: &mut Criterion) {
//     let tokens = vec![
//         LumpToken::MarkerStart("S_START".to_string()),
//         LumpToken::Lump("TEST1".to_string(), LumpRef::new(0, 0, 0)),
//         LumpToken::Lump("TEST2".to_string(), LumpRef::new(0, 0, 0)),
//         LumpToken::MarkerEnd("S_END".to_string()),
//         LumpToken::Lump("TEST3".to_string(), LumpRef::new(0, 0, 0)),
//     ];
//
//     let mut group = b.benchmark_group("Wad indexing lumps");
//     group.throughput(Throughput::Elements(tokens.len() as u64));
//     group.sample_size(100);
//
//     group.bench_function("index_lumps", |b| {
//         b.iter(|| {
//             index_tokens(&tokens).unwrap();
//         })
//     });
//
//     group.finish();
// }

fn bench_converting_audio(b: &mut Criterion) {
    let wad_data = WAD_DATA;
    let wad = WadIndex::from_bytes("freedoom1.wad".to_string(), &wad_data).unwrap();
    let filtered_lumps: Vec<&LumpRef> = wad
        .get_lump_index()
        .iter()
        .filter(|(name, _)| name.starts_with("DS"))
        .map(|(_, lump_node)| match lump_node {
            LumpNode::Namespace { .. } => panic!("Sound lump is a namespace, expected a lump"),
            LumpNode::Lump {lump,  .. } => lump,
        })
        .collect();

    let mut group = b.benchmark_group("Wad audio conversion");
    group.throughput(Throughput::Elements(filtered_lumps.len() as u64));
    group.sample_size(100);
    group.bench_function("convert_sounds", |b| {
        b.iter(|| {
            for lump_ref in &filtered_lumps {
                let data = lump_ref.data();
                let _sample = wad_rs::audio::SoundSample::try_from(data.as_ref()).unwrap();
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_wad_from_bytes,
    // bench_indexing_lumps,
    bench_converting_audio,
);
criterion_main!(benches);
