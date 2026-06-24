use std::alloc::Layout;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};

use camino::Utf8PathBuf;
use image::{codecs::png::PngEncoder, ColorType, GrayImage, ImageBuffer, ImageEncoder, Luma, Rgba, RgbaImage};

use engage::app::chapterdata::IChapterDataMethods;
use engage::app::facethumbnail::FaceThumbnail;
use engage::app::gameicon::GameIcon;
use engage::app::gmapmapinfocontent::{GmapMapInfoContent, IGmapMapInfoContent};
use engage::app::gmapspot::{GmapSpot, IGmapSpotMethods};
use engage::app::goddata::{GodData, IGodDataMethods};
use engage::app::godcolorrefineemblem::GodColorRefineEmblem;
use engage::app::jobdata::{IJobDataMethods, JobData};
use engage::app::mainsequence::MainSequence;
use engage::app::persondata::{IPersonDataMethods, PersonData};
use engage::app::ringdata::{IRingDataMethods, RingData};
use engage::app::spriteatlasmanager_2::{ISpriteAtlasManager_2, SpriteAtlasManager_2 as SpriteAtlasManager};
use engage::app::structdata_1::IStructData_1Methods as IStructDataMethods;
use engage::app::uniticon::{IUnitIcon, IUnitIconMethods, UnitIcon};
use engage::root::mapuigauge::{IMapUIGauge, MapUIGauge};

use engage::unity_engine::sprite::Sprite;
use engage::unity_engine::texture2d::{ITexture2DMethods, Texture2D};
use engage::unity_engine::texture::ITextureMethods;
use engage::unity_engine::imageconversion::ImageConversion;
use engage::unity_engine::ui::graphic::IGraphicMethods;
use engage::unity_engine::ui::image::IImageMethods;
use engage::unity_engine::u2d::spriteatlas::ISpriteAtlasMethods;
use engage::unity_engine::color::Color;
use engage::unity_engine::rendering::colorutils::ColorUtils;
use engage::unity_engine::filtermode::FilterMode;
use engage::unity_engine::rect::Rect;
use engage::unity_engine::spritemeshtype::SpriteMeshType;
use engage::unity_engine::vector2::Vector2;
use engage::unity_engine::experimental::rendering::graphicsformat::GraphicsFormat;
use engage::unity_engine::experimental::rendering::texturecreationflags::TextureCreationFlags;
use engage::system::collections::generic::dictionary_2::IDictionary_2Methods as IDictionaryMethods;
use unity::prelude::*;
use engage::{List_1Ext, Dictionary_2Ext};

use crate::api::events::SystemEvent;

#[repr(i32)]
#[derive(Debug, PartialEq)]
pub enum PngResult {
    Ok = 0,
    Error,
    OutOfMemory,
}

#[repr(C)]
#[derive(Debug)]
pub struct Dimension {
    width: i32,
    height: i32,
}

#[repr(C)]
pub struct PngDecoder {
    _unk: [u8; 0x8C],
}

impl PngDecoder {
    extern "C" fn malloc_cb(size: usize, _user_data: *const u8) -> *mut u8 {
        match Layout::from_size_align(size, 16) {
            Ok(layout) => unsafe { std::alloc::alloc(layout) },
            Err(_) => std::ptr::null_mut(),
        }
    }

    extern "C" fn free_cb(ptr: *mut u8, _user_data: *const u8) {
        unsafe { std::alloc::dealloc(ptr, Layout::new::<()>()) }
    }

    pub fn new() -> Self {
        let this = Self { _unk: [0u8; 0x8C] };
        unsafe { png_ctor(&this) };
        this
    }

    pub fn initialize(&mut self) {
        unsafe {
            png_initialize(
                self,
                Self::malloc_cb,
                std::ptr::null(),
                Self::free_cb,
                std::ptr::null(),
            )
        }
    }

    pub fn set_image_data(&mut self, png: &[u8]) {
        unsafe { png_set_image_data(self, png.as_ptr(), png.len()) }
    }

    pub fn analyze(&mut self) -> PngResult {
        unsafe { png_analyze(self) }
    }

    pub fn dimension(&self) -> Dimension {
        unsafe { png_get_dimension(self) }
    }
}

impl Drop for PngDecoder {
    fn drop(&mut self) {
        unsafe { png_dtor(self) }
    }
}

extern "C" {
    #[link_name = "_ZN2nn5image10PngDecoderC1Ev"]
    fn png_ctor(this: &PngDecoder);

    #[link_name = "_ZN2nn5image10PngDecoderD1Ev"]
    fn png_dtor(this: &PngDecoder);

    #[link_name = "_ZN2nn5image10PngDecoder10InitializeEPFPvmS2_ES2_PFvS2_S2_ES2_"]
    fn png_initialize(
        this: &PngDecoder,
        alloc_fn: extern "C" fn(size: usize, data: *const u8) -> *mut u8,
        allocate_data: *const u8,
        free_fn: extern "C" fn(ptr: *mut u8, data: *const u8),
        free_data: *const u8,
    );

    #[link_name = "_ZN2nn5image10PngDecoder12SetImageDataEPKvm"]
    fn png_set_image_data(this: &mut PngDecoder, data: *const u8, size: usize);

    #[link_name = "_ZN2nn5image10PngDecoder7AnalyzeEv"]
    fn png_analyze(this: &mut PngDecoder) -> PngResult;

    #[link_name = "_ZNK2nn5image10PngDecoder12GetDimensionEv"]
    fn png_get_dimension(this: &PngDecoder) -> Dimension;
}

// (width, height) == (0, 0) means use whatever the file reports, any other value is
// validated against the PNG's real dimensions
fn load_sprite(name: Il2CppString, filepath: &str, mut width: i32, mut height: i32, filter_mode: FilterMode) -> Option<Sprite> {
    if name.is_null() {
        return None;
    }

    let path = Utf8PathBuf::from(filepath).join(name.to_rust_string()).with_extension("png");
    let file = mods::read(&path).ok()?;

    let mut decoder = PngDecoder::new();
    decoder.initialize();
    decoder.set_image_data(&file);
    if decoder.analyze() == PngResult::Ok {
        let dim = decoder.dimension();
        if (width, height) != (dim.width, dim.height) {
            if (width, height) == (0, 0) {
                (width, height) = (dim.width, dim.height);
            } else {
                panic!(
                    "Malformed sprite file\nLocation: {}\nDimensions: {}x{}\nExpected: {}x{}\n\nResize the image to avoid stretching",
                    path, dim.width, dim.height, width, height
                );
            }
        }
    }

    let array = Array::<u8>::from_slice(&file)?;
    let texture = Texture2D::new_9(width, height);
    if !ImageConversion::load_image_2(texture, array) {
        panic!(
            "Could not load icon at `{}`.\n\nMake sure it is a PNG file with a dimension of {}x{} pixels",
            path, width, height
        );
    }

    texture.set_filter_mode(filter_mode);
    let rect = Rect { m_x_min: 0.0, m_y_min: 0.0, m_width: width as f32, m_height: height as f32 };
    let pivot = Vector2 { x: 0.5, y: 0.5 };
    Some(Sprite::create_5(texture, rect, pivot, 100.0, 1u32, SpriteMeshType::tight()))
}

struct QuantizedIcon {
    palette: RgbaImage,
    indices: GrayImage,
}

fn quantize_icon(icon: &RgbaImage) -> QuantizedIcon {
    let (w, h) = icon.dimensions();

    let mut palette = ImageBuffer::new(512, 1);
    let mut indices = ImageBuffer::new(w, h);
    let mut lookup: HashMap<[u8; 4], u8> = HashMap::with_capacity(256);

    let transparent: Rgba<u8> = Rgba([0, 0, 0, 0]);
    palette.put_pixel(0, 0, transparent);
    palette.put_pixel(1, 0, transparent);
    lookup.insert(transparent.0, 0);
    let mut next_index: u8 = 1;

    for (x, y, pixel) in icon.enumerate_pixels() {
        let index = match lookup.get(&pixel.0) {
            Some(&i) => i,
            None => {
                if next_index as usize >= 256 {
                    panic!("Palette has too many colors.");
                }
                let slot = next_index as u32 * 2;
                palette.put_pixel(slot, 0, *pixel);
                palette.put_pixel(slot + 1, 0, *pixel);
                lookup.insert(pixel.0, next_index);
                let assigned = next_index;
                next_index += 1;
                assigned
            }
        };
        indices.put_pixel(x, y, Luma([index]));
    }

    QuantizedIcon { palette, indices }
}

fn generate_palette_sprite(palette: &RgbaImage) -> Sprite {
    let texture = Texture2D::new_4(
        512,
        1,
        GraphicsFormat::r8g8b8a8_srgb(),
        1, // single mip level
        TextureCreationFlags::none(),
    );
    texture.set_filter_mode(FilterMode::point());
    texture.set_aniso_level(1);

    let mut png_buf = Vec::<u8>::with_capacity(4096);
    PngEncoder::new(&mut png_buf)
        .write_image(palette.as_raw(), 512, 1, ColorType::Rgba8)
        .expect("palette png encode");
    let array = Array::<u8>::from_slice(&png_buf).expect("palette array");
    ImageConversion::load_image_2(texture, array);

    let rect = Rect { m_x_min: 0.0, m_y_min: 0.0, m_width: 512.0, m_height: 1.0 };
    let pivot = Vector2 { x: 0.0, y: 0.0 };
    Sprite::create_5(texture, rect, pivot, 100.0, 1u32, SpriteMeshType::full_rect())
}

fn generate_unit_sprite(indices: &GrayImage) -> Sprite {
    let flipped = image::imageops::flip_vertical(indices);
    let (w, h) = flipped.dimensions();

    let texture = Texture2D::new_4(
        w as i32,
        h as i32,
        GraphicsFormat::r8_u_norm(),
        1, // single mip level
        TextureCreationFlags::none(),
    );

    let raw = flipped.as_raw();
    let array = Array::<u8>::from_slice(raw).expect("index array");
    texture.set_pixel_data_impl_array(array, 0, 1, raw.len() as i32, 0);
    texture.set_filter_mode(FilterMode::point());
    texture.set_aniso_level(1);
    texture.apply_2(false);

    let rect = Rect { m_x_min: 0.0, m_y_min: 0.0, m_width: w as f32, m_height: h as f32 };
    let pivot = Vector2 { x: 0.5, y: 0.5 };
    Sprite::create_5(texture, rect, pivot, 100.0, 1u32, SpriteMeshType::tight())
}

fn build_unit_icon_sprites(file: &[u8], sprite_path: &Utf8PathBuf) -> (Sprite, Sprite) {
    let icon = image::load_from_memory_with_format(file, image::ImageFormat::Png)
        .expect("decode character icon PNG")
        .to_rgba8();

    if icon.dimensions() != (48, 48) {
        panic!(
            "Malformed sprite file\nLocation: {}\nDimensions: {}x{}\nExpected: 48x48\n\nResize the image to avoid stretching",
            sprite_path,
            icon.width(),
            icon.height()
        );
    }

    let quantized = quantize_icon(&icon);
    (
        generate_palette_sprite(&quantized.palette),
        generate_unit_sprite(&quantized.indices),
    )
}

// Fall back to `Unit_Unit.png` when the exact `Unit_Job_Item.png` is missing
fn find_unit_icon_file(index_name: Il2CppString) -> Option<(Vec<u8>, Utf8PathBuf)> {
    let name_str = index_name.to_rust_string();
    let primary = Utf8PathBuf::from("patches/icon/job").join(&name_str).with_extension("png");

    if let Ok(file) = mods::read(&primary) {
        return Some((file, primary));
    }

    let unit = name_str.split('_').next()?;
    let fallback = Utf8PathBuf::from("patches/icon/job")
        .join(format!("{unit}_{unit}"))
        .with_extension("png");
    mods::read(&fallback).ok().map(|f| (f, fallback))
}

#[unity::hook("App", "UnitIcon", "UpdateIcon")]
pub fn uniticon_update_icon(this: UnitIcon, method_info: OptionalMethod) {
    let icon_name = this.m_icon_name();
    if !icon_name.is_null() {
        if let Some((file, path)) = find_unit_icon_file(icon_name) {
            let (palette_sprite, unit_sprite) = build_unit_icon_sprites(&file, &path);
            this.set_pallete_sprite(palette_sprite);
            this.set_sprite(unit_sprite);
            this.set_all_dirty();
            return;
        }
    }
    call_original!(this, method_info);
}

#[unity::hook("App", "UnitIcon", "TrySet")]
pub fn uniticon_try_set(
    this: UnitIcon,
    index_name: Il2CppString,
    palette_name: Il2CppString,
    method_info: OptionalMethod,
) -> bool {
    if !index_name.is_null() {
        if let Some((file, path)) = find_unit_icon_file(index_name) {
            let (palette_sprite, unit_sprite) = build_unit_icon_sprites(&file, &path);
            this.set_m_icon_name(index_name);
            this.set_pallete_sprite(palette_sprite);
            this.set_sprite(unit_sprite);
            this.set_vertices_dirty();
            return true;
        }

        if !GameIcon::tyr_get_unit_icon_index(index_name).is_null() {
            return call_original!(this, index_name, palette_name, method_info);
        }
    }

    if !palette_name.is_null() {
        let fallback = Il2CppString::new(format!("{0}_{0}", palette_name.to_rust_string()));
        call_original!(this, fallback, palette_name, method_info)
    } else {
        call_original!(this, index_name, palette_name, method_info)
    }
}

#[unity::hook("App", "BondsRingFacePicture", "Get")]
pub fn bondsringfacepicture_get(ring: RingData, method_info: OptionalMethod) -> Sprite {
    load_sprite(ring.get_icon(), "patches/icon/face", 300, 300, FilterMode::point())
        .unwrap_or_else(|| call_original!(ring, method_info))
}

#[unity::hook("App", "GodFacePicture", "Get")]
pub fn godfacepicture_getsprite(god: GodData, method_info: OptionalMethod) -> Sprite {
    load_sprite(god.get_ascii_name(), "patches/icon/emblem/godface", 388, 388, FilterMode::point())
        .unwrap_or_else(|| call_original!(god, method_info))
}

#[unity::hook("App", "GodColorRefineEmblem", "GetColor")]
pub fn godcolorrefineemblem_getcolor(
    this: GodColorRefineEmblem,
    god: GodData,
    method_info: OptionalMethod,
) -> Color {
    let ascii_name = god.get_ascii_name();
    if ascii_name.is_null() {
        return call_original!(this, god, method_info);
    }

    let path = Utf8PathBuf::from("patches/icon/emblem/godsymbol")
        .join(ascii_name.to_rust_string())
        .with_extension("txt");

    match mods::read(&path) {
        Ok(bytes) => match std::str::from_utf8(&bytes) {
            Ok(hex) => hex_to_rgba(hex),
            Err(_) => call_original!(this, god, method_info),
        },
        Err(_) => call_original!(this, god, method_info),
    }
}

fn hex_to_rgba(hex: &str) -> Color {
    let hex = hex.trim().trim_start_matches('#');
    let white = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };

    if hex.chars().any(|c| !c.is_ascii_hexdigit()) {
        return white;
    }

    match hex.len() {
        6 => {
            // ColorUtils expects 0xAARRGGBB
            let packed = u32::from_str_radix(hex, 16).unwrap_or(0xFFFFFF) | 0xFF00_0000;
            ColorUtils::to_rgba(packed)
        }
        8 => {
            // Input 0xRRGGBBAA, shuffle to 0xAARRGGBB
            let rgba = u32::from_str_radix(hex, 16).unwrap_or(0xFFFFFFFF);
            let packed = (rgba >> 8) | (rgba << 24);
            ColorUtils::to_rgba(packed)
        }
        _ => white,
    }
}

#[skyline::hook(offset = 0x201f7d0)]
pub fn mapuigauge_getspritebyindex(this: MapUIGauge, index: i32, _method_info: OptionalMethod) -> Sprite {
    let sprites = this.m_sprites();
    if sprites.is_null() || (index as usize) >= sprites.len() {
        return Sprite::instantiate().expect("Sprite::instantiate");
    }
    let sprite = sprites.get(index as usize);
    if sprite.is_null() {
        Sprite::instantiate().expect("Sprite::instantiate")
    } else {
        sprite
    }
}

#[skyline::hook(offset = 0x2026630)]
pub fn mapuigauge_cctor(method_info: OptionalMethod) {
    call_original!(method_info);

    let existing_names: HashSet<String> = MapUIGauge::icon_names()
        .into_iter()
        .filter(|s| !s.is_null())
        .map(|s| s.to_rust_string())
        .collect();

    let new_names: Vec<Il2CppString> = mods::list("patches/icon/mapstatus")
        .filter(|p| p.extension() == Some("png"))
        .filter_map(|p| p.file_stem())
        .filter(|stem| !existing_names.contains(*stem))
        .map(Il2CppString::new)
        .collect();

    if new_names.is_empty() {
        return;
    }

    let mut combined: Vec<Il2CppString> = MapUIGauge::icon_names().into_iter().collect();
    combined.extend(new_names);
    if let Some(arr) = Array::<Il2CppString>::from_slice(&combined) {
        MapUIGauge::set_icon_names(arr);
    }
}

#[unity::hook("", "MapUIGauge", "OnEnable")]
pub fn mapuigauge_onenable(this: MapUIGauge, _method_info: OptionalMethod) {
    if !this.m_sprites().is_null() {
        return;
    }

    let atlas = this.m_sprite_atlas();
    let dictionary = this.m_dictionary();
    let names = MapUIGauge::icon_names();
    let len = names.len();
    let mut temp: Vec<Sprite> = Vec::with_capacity(len);

    for (idx, name) in names.into_iter().enumerate() {
        let sprite = if idx < 0xA7 {
            atlas.get_sprite(name)
        } else {
            load_sprite(name, "patches/icon/mapstatus", 0, 0, FilterMode::bilinear())
                .unwrap_or_else(|| Sprite::instantiate().expect("Sprite::instantiate"))
        };
        temp.push(sprite);
        dictionary.add(name, sprite);
    }

    if let Some(arr) = Array::<Sprite>::from_slice(&temp) {
        this.set_m_sprites(arr);
    }
}

#[unity::hook("App", "GmapMapInfoContent", "SetMapInfo")]
pub fn gmapinfocontent_setmapinfo_hook(this: GmapMapInfoContent, gmap_spot: GmapSpot, method_info: OptionalMethod) {
    call_original!(this, gmap_spot, method_info);

    let chapter = gmap_spot.get_chapter();
    if chapter.is_null() {
        return;
    }
    let prefixless = chapter.get_prefixless_cid();
    if let Some(thumb) = load_sprite(prefixless, "patches/ui/gmap/infothumb", 468, 256, FilterMode::trilinear()) {
        this.set_m_map_info_sprite(thumb);
        this.m_map_info_image().set_sprite(thumb);
    }
}

#[unity::hook("App", "MainSequence", "PostLoadResource")]
pub fn add_custom_sprites_icons(this: MainSequence, method_info: OptionalMethod) {
    call_original!(this, method_info);
    println!("[MainSequence] PostLoadResource");

    let mut counters = AtlasCounters::default();

    let face_thumbs = FaceThumbnail::s_face_thumb();
    let filter_tri = FilterMode::trilinear();
    for dir in [
        "patches/icon/facethumb/unit",
        "patches/icon/facethumb/emblem",
        "patches/icon/facethumb/bond",
    ] {
        inject_atlas_from_dir(face_thumbs, dir, 188, 74, filter_tri, &mut counters);
    }

    let skill = GameIcon::s_skill();
    let item = GameIcon::s_item();
    let god_ring = GameIcon::s_god_ring();
    let god_symbol = GameIcon::s_god_symbol();
    let system = GameIcon::s_system();

    for (atlas, dir, w, h) in [
        (skill, "patches/icon/skill", 56, 56),
        (item, "patches/icon/item", 64, 64),
        (god_ring, "patches/icon/emblem/godring", 74, 74),
        (god_symbol, "patches/icon/emblem/godsymbol", 72, 72),
        (system, "patches/icon/system", 48, 48),
    ] {
        inject_atlas_from_dir(atlas, dir, w, h, FilterMode::trilinear(), &mut counters);
    }

    add_default_unit_sprites();
    println!(
        "Changed/Added Sprite/Facethumb to Dictionaries: {}/{}",
        counters.changed, counters.added
    );
}

#[derive(Default)]
struct AtlasCounters {
    added: u32,
    changed: u32,
}

fn inject_atlas_from_dir(
    atlas: SpriteAtlasManager,
    dir: &str,
    width: i32,
    height: i32,
    filter_mode: FilterMode,
    counters: &mut AtlasCounters,
) {
    let cache = atlas.m_cache_table();

    for path in mods::list(dir) {
        if path.extension() != Some("png") {
            continue;
        }
        let Some(stem) = path.file_stem() else { continue };
        let filename = Il2CppString::new(stem);
        let Some(sprite) = load_sprite(filename, dir, width, height, filter_mode) else {
            continue;
        };

        if cache.contains_key(filename) {
            cache.remove(filename);
            counters.changed += 1;
            println!("Changed {} in {}", filename, dir);
        } else {
            counters.added += 1;
            println!("Added {} from {}", filename, dir);
        }
        cache.add(filename, sprite);
    }
}

// Ensures every palette has a matching `<palette>_<palette>` unit-icon entry so the
// game falls back gracefully when a specific Unit_Job pair is missing
fn add_default_unit_sprites() {
    println!("Adding default sprites");

    let unit_icon_cache = GameIcon::s_unit_icon_index().m_cache_table();
    let palette_cache = GameIcon::s_unit_icon_pallete().m_cache_table();

    let palettes: Vec<Il2CppString> = palette_cache
        .iter()
        .filter_map(|(k, _)| (!k.is_null()).then_some(k))
        .collect();

    let mut key_sprite_pair: Vec<(String, Sprite)> = unit_icon_cache
        .iter()
        .filter_map(|(k, v)| {
            if k.is_null() {
                return None;
            }
            let key_str = k.to_rust_string();
            key_str.contains("NoWeapon").then_some((key_str, v))
        })
        .collect();

    let mut added_count = 0u32;

    for palette in &palettes {
        let palette_str = palette.to_rust_string();
        let default_key = format!("{palette_str}_{palette_str}");
        if !GameIcon::tyr_get_unit_icon_index(Il2CppString::new(&default_key)).is_null() {
            continue;
        }
        let noweapon_key = format!("{default_key}_NoWeapon");
        let sprite = GameIcon::tyr_get_unit_icon_index(Il2CppString::new(&noweapon_key));
        if !sprite.is_null() {
            unit_icon_cache.add(Il2CppString::new(&default_key), sprite);
            added_count += 1;
        }
    }

    for job in JobData::get_list().iter() {
        for job_icon in [job.get_unit_icon_id_f(), job.get_unit_icon_id_m()] {
            if job_icon.is_null() {
                continue;
            }
            let icon_str = job_icon.to_rust_string();
            let matching: Vec<&(String, Sprite)> = key_sprite_pair
                .iter()
                .filter(|(k, _)| k.contains(&icon_str))
                .collect();
            if matching.is_empty() || matching.len() >= 5 {
                continue;
            }

            for palette in &palettes {
                let palette_str = palette.to_rust_string();
                if !matching.iter().any(|(k, _)| k.contains(&palette_str)) {
                    continue;
                }
                let default_key = format!("{palette_str}_{palette_str}");
                if !GameIcon::tyr_get_unit_icon_index(Il2CppString::new(&default_key)).is_null() {
                    continue;
                }
                if let Some((_, sprite)) = matching.iter().find(|(k, _)| k.contains(&palette_str)) {
                    unit_icon_cache.add(Il2CppString::new(&default_key), *sprite);
                    added_count += 1;
                }
            }
        }
    }

    let palette_to_jobs: HashMap<String, Vec<JobData>> = {
        let mut map: HashMap<String, Vec<JobData>> = HashMap::new();
        for person in PersonData::get_list().iter() {
            let icon_id = person.get_unit_icon_id();
            let job = person.get_job();
            if icon_id.is_null() || job.is_null() {
                continue;
            }
            map.entry(icon_id.to_rust_string()).or_default().push(job);
        }
        map
    };

    for palette in &palettes {
        let palette_str = palette.to_rust_string();
        let default_key = format!("{palette_str}_{palette_str}");
        if !GameIcon::tyr_get_unit_icon_index(Il2CppString::new(&default_key)).is_null() {
            continue;
        }
        let Some(jobs) = palette_to_jobs.get(&palette_str) else { continue };

        'jobs: for job in jobs {
            for job_icon in [job.get_unit_icon_id_f(), job.get_unit_icon_id_m()] {
                if job_icon.is_null() {
                    continue;
                }
                let icon_str = job_icon.to_rust_string();
                if let Some(pos) = key_sprite_pair
                    .iter()
                    .position(|(k, _)| k.contains(&icon_str) && k.contains(&palette_str))
                {
                    let (_, sprite) = key_sprite_pair.remove(pos);
                    unit_icon_cache.add(Il2CppString::new(&default_key), sprite);
                    added_count += 1;
                    break 'jobs;
                }
            }
        }
    }
    // Default Icons that are not 'NoWeapon'
    for key in ["102Louis_630LanceArmor_Lance", "153Chloe_646LancePegasus_Lance", "203Umber_637LanceKnight_Lance", "250Jade_631AxArmor_Ax"]{
        let palette = key.split_once('_').unwrap().0;
        let default_key = format!("{palette}_{palette}_NoWeapon");
        let sprite = GameIcon::tyr_get_unit_icon_index(Il2CppString::new(key));
        if GameIcon::tyr_get_unit_icon_index(Il2CppString::new(&default_key)).is_null() {
            unit_icon_cache.add(Il2CppString::new(&default_key), sprite);
            added_count += 1;
        }
    }
    println!("Added {} Default Sprites", added_count);
}

pub static SPRITES_PATCHED: AtomicBool = AtomicBool::new(false);

pub extern "C" fn add_default_sprites(evt: &crate::api::events::Event<SystemEvent>) {
    use engage::app::procinst::IProcInst;

    let crate::api::events::Event::Args(SystemEvent::ProcInstJump { proc, .. }) = evt else {
        return;
    };

    if proc.m_hash_code() == crate::sequences::titleloop::TITLELOOPSEQUENCE_HASHCODE
        && !SPRITES_PATCHED.load(Ordering::Relaxed)
    {
        add_default_unit_sprites();
        SPRITES_PATCHED.store(true, Ordering::Relaxed);
    }
}
