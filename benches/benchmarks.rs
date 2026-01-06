use std::hint::black_box;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use wad_rs::WadIndex;
use wad_rs::graphics::PaletteColorMapMapper;
use wad_rs::index::LumpNode;
use wad_rs::lump::LumpRef;

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

fn bench_converting_sprites(b: &mut Criterion) {
    let wad_data = WAD_DATA;
    let wad = WadIndex::from_bytes("freedoom2.wad".to_string(), &wad_data).unwrap();
    let sprite_index = match wad.get_lump(Vec::new(), "S_START").unwrap() {
        LumpNode::Namespace { children, .. } => children,
        _ => panic!("S_START is not a namespace"),
    };

    let palette_lump = wad.get_lump(Vec::new(), "PLAYPAL").unwrap();
    let lump_ref = match palette_lump {
        LumpNode::Lump { lump, .. } => lump,
        _ => panic!("PLAYPAL is not a lump"),
    };
    let palette_data = lump_ref.data();
    let palette = wad_rs::graphics::Palette::try_from(palette_data).unwrap();

    let colormap_lump = wad.get_lump(Vec::new(), "COLORMAP").unwrap();
    let lump_ref = match colormap_lump {
        LumpNode::Lump { lump, .. } => lump,
        _ => panic!("COLORMAP is not a lump"),
    };
    let colormap_data = lump_ref.data();
    let colormap = wad_rs::graphics::ColorMap::from_bytes(colormap_data).unwrap();

    let remapper = PaletteColorMapMapper::new(&palette, &colormap, 15).unwrap();

    let total_pixels: u64 = sprite_index
        .iter()
        .filter_map(|(_, lump_node)| match lump_node {
            LumpNode::Lump { lump, .. } => Some(lump),
            _ => None,
        })
        .map(|lump_ref| {
            let sprite = wad_rs::sprite::Sprite::new(lump_ref.data()).unwrap();
            (sprite.width() * sprite.height()) as u64
        })
        .sum();

    let mut group = b.benchmark_group("Wad sprite conversion");
    group.throughput(Throughput::Elements(total_pixels));
    group.sample_size(50);
    group.bench_function("convert_sprites", |b| {
        b.iter(|| {
            let mut bytes_out = 0usize;
            for (_, lump_node) in sprite_index.iter() {
                let lump_ref = match lump_node {
                    LumpNode::Lump { lump, .. } => lump,
                    _ => continue,
                };
                let buf = wad_rs::sprite::Sprite::new(lump_ref.data())
                    .unwrap()
                    .rgba_pixel_buffer(&remapper)
                    .unwrap();
                bytes_out = bytes_out.wrapping_add(buf.len());
                black_box(buf);
            }
            black_box(bytes_out);
        })
    });

    group.finish();
}

fn bench_converting_audio(b: &mut Criterion) {
    let wad = WadIndex::from_bytes("freedoom1.wad".to_string(), WAD_DATA).unwrap();
    let filtered_lumps: Vec<&LumpRef> = wad
        .get_lump_index()
        .iter()
        .filter(|(name, _)| name.starts_with("DS"))
        .map(|(_, lump_node)| match lump_node {
            LumpNode::Lump { lump, .. } => lump,
            _ => panic!("not a lump"),
        })
        .collect();

    let mut group = b.benchmark_group("Wad audio conversion");
    group.throughput(Throughput::Elements(filtered_lumps.len() as u64));
    group.sample_size(100);
    group.bench_function("convert_sounds", |b| {
        b.iter(|| {
            for lump_ref in &filtered_lumps {
                let data = lump_ref.data();
                let _ = wad_rs::audio::SoundSample::try_from(data).unwrap();
            }
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_wad_from_bytes,
    bench_converting_sprites,
    bench_converting_audio,
);
criterion_main!(benches);
