use std::{
    alloc::Layout, ops::Deref,
};

use engage::{gamedata::{dispos::ChapterData, item::ItemData, ring::RingData, unit::Unit, GodData}, uniticon::UnitIcon};

use camino::{Utf8Path, Utf8PathBuf};

use image::{codecs::png::PngEncoder, ColorType, GrayImage, ImageBuffer, ImageEncoder, Luma, Pixel, Rgba, RgbaImage};

use unity::{
    engine::{
        ui::{Image, IsImage},
        Color, FilterMode, ImageConversion, Rect, Sprite, SpriteMeshType, Texture2D, Vector2,
    }, il2cpp::object::Array, prelude::*, system::Dictionary
};

#[repr(i32)]
#[derive(Debug, PartialEq)]

pub enum PngResult {
    Ok = 0,
    Error,
    OutOfMemory
}

#[repr(C)]
#[derive(Debug)]
pub struct Dimension {
    width: i32,
    height: i32
}

#[repr(C)]
pub struct PngDecoder {
    unk: [u8; 0x8C],
}

impl PngDecoder {
    extern "C" fn pngdecoder_malloc(size: usize, _user_data: *const u8) -> *mut u8 {
        if let Ok(layout) = Layout::from_size_align(size, 16) {
            unsafe { std::alloc::alloc(layout) }
        } else {
            // I don't think it's wise to panic in a allocator but who knows
            std::ptr::null_mut()
        }
    }

    extern "C" fn pngdecoder_free(ptr: *mut u8, _user_data: *const u8) {
        // Switch's global allocator is malloc/free and therefore do not care about the Layout. I don't really see the point of this API.
        unsafe { std::alloc::dealloc(ptr, Layout::new::<()>()) }
    }

    pub fn new() -> Self {
        let this = Self {
            unk: [0u8;0x8c],
        };

        unsafe { ctor(&this) }

        this
    }

    pub fn initialize(&mut self) {
        unsafe { initialize(self, Self::pngdecoder_malloc, 0 as _, Self::pngdecoder_free, 0 as _) }
    }

    pub fn set_image_data(&mut self, png: impl AsRef<[u8]>) {
        let data = png.as_ref();
        unsafe { set_image_data(self, data.as_ptr(), data.len()) }
    }

    pub fn analyze(&mut self) -> PngResult {
        unsafe { analyze(self) }
    }

    pub fn get_dimension(&self) -> Dimension {
        unsafe { get_dimension(self) }
    }
    
}

impl Drop for PngDecoder {
    fn drop(&mut self) {
        unsafe { dtor(self) }
    }
}

extern "C" {
    #[link_name = "_ZN2nn5image10PngDecoderC1Ev"]
    pub fn ctor(this: &PngDecoder);

    #[link_name = "_ZN2nn5image10PngDecoderD1Ev"]
    pub fn dtor(this: &PngDecoder);
    
    #[link_name = "_ZN2nn5image10PngDecoder10InitializeEPFPvmS2_ES2_PFvS2_S2_ES2_"]
    pub fn initialize(this: &PngDecoder, alloc_fn: extern "C" fn(size: usize, data: *const u8) -> *mut u8, allocate_data: *const u8, free_fn: extern "C" fn(ptr: *mut u8, data: *const u8), free_data: *const u8);

    #[link_name = "_ZN2nn5image10PngDecoder12SetImageDataEPKvm"]
    pub fn set_image_data(this: &mut PngDecoder, data: *const u8, size: usize);

    #[link_name = "_ZN2nn5image10PngDecoder7AnalyzeEv"]
    pub fn analyze(this: &mut PngDecoder) -> PngResult;

    #[link_name = "_ZNK2nn5image10PngDecoder12GetDimensionEv"]
    pub fn get_dimension(this: &PngDecoder) -> Dimension;
}

fn load_sprite(name: Option<&Il2CppString>, filepath: &str, mut width: i32, mut height: i32, filter_mode: FilterMode) -> Option<&'static mut Sprite> {
    if let Some(this) = name {
        let path = Utf8PathBuf::from(filepath)
            .join(this.to_string())
            .with_extension("png");

        if let Ok(file) = mods::manager::Manager::get().get_file(&path) {
            let array = Il2CppArray::from_slice(file).unwrap();

            let mut decoder = PngDecoder::new();

            decoder.initialize();
            decoder.set_image_data(array.fields.deref());

            if PngResult::Ok == decoder.analyze() {
                let dim = decoder.get_dimension();
                if (width, height) != (dim.width, dim.height) {
                    if (width, height) == (0, 0) { // WxH of 0x0 implies that it could be anything
                        (width, height) = (dim.width, dim.height);
                    } else {
                        panic!("Malformed sprite file\nLocation: {}\nDimensions: {}x{} \nExpected: {}x{}\n\nResize the image to avoid stretching", path, dim.width, dim.height, width, height);
                    }
                }
            }

            let new_texture = Texture2D::new(width, height);
            
            if ImageConversion::load_image(new_texture, array) {
                new_texture.set_filter_mode(filter_mode);

                //println!("Before Sprite::Create");
                let rect = Rect::new(0.0, 0.0, width as f32, height as f32);
                let pivot = Vector2::new(0.5, 0.5);

                return Some(Sprite::create2(new_texture, rect, pivot, 100.0, 1, SpriteMeshType::Tight));
            } else {
                panic!("Could not load icon at `{}`.\n\nMake sure it is a PNG file with a dimension of {}x{} pixels", path, width, height);
            }
        }
    }
    None
}

#[unity::from_offset("UnityEngine", "Texture2D", "get_format")]
fn texture2d_get_format(this: &Texture2D, method_info: OptionalMethod) -> i32;

#[unity::hook("App", "UnitIcon", "UpdateIcon")] // What does this even do?
pub fn uniticon_update_icon(this: &mut UnitIcon,  method_info: OptionalMethod) {
    if let Some(index) = this.icon_name {
        let sprite_path = Utf8PathBuf::from("patches/icon/job").join(index.to_string()).with_extension("png");

        if let Ok(file) = mods::manager::Manager::get().get_file(&sprite_path) {
            let unit_icon = image::load_from_memory_with_format(&file, image::ImageFormat::Png).unwrap().to_rgba8();

            // Enforce sprite size
            if (48, 48) != (unit_icon.width(), unit_icon.height()) {
                panic!("Malformed sprite file\nLocation: {}\nDimensions: {}x{} \nExpected: 48x48\n\nResize the image to avoid stretching", sprite_path, unit_icon.width(), unit_icon.height());
            }

            let palette = make_palette(&unit_icon);

            let palette_sprite = generate_palette_sprite(&palette);
            let unit_sprite = generate_unit_sprite(&unit_icon, &palette);

            this.pallete_sprite = Some(palette_sprite);
            this.set_sprite(unit_sprite);

            // Set the UnitIcon as dirty to refresh the material and texture and sprite
            this
                .get_class()
                .get_virtual_method("SetAllDirty")
                .map(|method| {
                    let close_anime_all = unsafe {
                        std::mem::transmute::<_, extern "C" fn(&UnitIcon, &MethodInfo)>(
                            method.method_info.method_ptr,
                        )
                    };
                    close_anime_all(this, method.method_info);
                })
                .unwrap();
        } else {
            call_original!(this, method_info)
        }

    } else {
        call_original!(this, method_info);
    }
}

#[unity::hook("App", "UnitIcon", "TrySet")]
pub fn uniticon_try_set(this: &mut UnitIcon, index_name: Option<&'static Il2CppString>, palette_name: Option<&Il2CppString>, method_info: OptionalMethod) -> bool {
    if let Some(index) = index_name {
        // Check if we have a sprite file with that name before proceeding
        let sprite_path = Utf8PathBuf::from("patches/icon/job").join(index.to_string()).with_extension("png");

        if let Ok(file) = mods::manager::Manager::get().get_file(&sprite_path) {
            let unit_icon = image::load_from_memory_with_format(&file, image::ImageFormat::Png).unwrap().to_rgba8();

            // Enforce sprite size
            if (48, 48) != (unit_icon.width(), unit_icon.height()) {
                panic!("Malformed sprite file\nLocation: {}\nDimensions: {}x{} \nExpected: 48x48\n\nResize the image to avoid stretching", sprite_path, unit_icon.width(), unit_icon.height());
            }
            let palette = make_palette(&unit_icon);

            let palette_sprite = generate_palette_sprite(&palette);
            let unit_sprite = generate_unit_sprite(&unit_icon, &palette);

            this.icon_name = index_name;
            this.pallete_sprite = Some(palette_sprite);
            this.set_sprite(unit_sprite);

            // Set the UnitIcon as dirty to refresh the material and texture and sprite
            this
                .get_class()
                .get_virtual_method("SetVerticesDirty")
                .map(|method| {
                    let close_anime_all = unsafe {
                        std::mem::transmute::<_, extern "C" fn(&UnitIcon, &MethodInfo)>(
                            method.method_info.method_ptr,
                        )
                    };
                    close_anime_all(this, method.method_info);
                })
                .unwrap();

            true
        } else {
            call_original!(this, index_name, palette_name, method_info)
        }
    } else {
        call_original!(this, index_name, palette_name, method_info)
    }
}

pub fn generate_palette_sprite(palette: &RgbaImage) -> &'static mut Sprite {
    let new_texture = Texture2D::instantiate().unwrap();
    unsafe { texture2d_ctor2(new_texture, 512, 1, 48, false, None) };
    new_texture.set_filter_mode(FilterMode::Point);
    unsafe { texture2d_set_anisolevel(new_texture, 1, None) };

    let mut temp = Vec::<u8>::new();
    PngEncoder::new(&mut temp).write_image(&palette, 512, 1, ColorType::Rgba8).unwrap();
    
    let array = Il2CppArray::from_slice(temp).unwrap();
    ImageConversion::load_image(new_texture, array);
    
    let rect = Rect::new(0.0, 0.0, 512 as f32, 1 as f32);
    let pivot = Vector2::new(0.0, 0.0);
    
    Sprite::create2(new_texture, rect, pivot, 100.0, 1, SpriteMeshType::FullRect)
}

pub fn generate_unit_sprite(unit_icon: &RgbaImage, palette: &RgbaImage) -> &'static mut Sprite {
    let r8_icon = make_r8(&unit_icon, &palette);
    let r8_icon = image::imageops::flip_vertical(&r8_icon);

    let new_texture = Texture2D::instantiate().unwrap();
    unsafe { texture2d_ctor2(new_texture, 48, 48, 63, false, None) }

    let test: Vec<u8> = r8_icon.pixels().map(|p| p.channels()[0]).collect();

    let len = test.len();

    let array = Il2CppArray::from_slice(test).unwrap();
    unsafe { texture2d_set_pixel_data_impl_array(new_texture, array, 0, 1, len as i32, 0, None) };
    new_texture.set_filter_mode(FilterMode::Point);
    unsafe { texture2d_set_anisolevel(new_texture, 1, None) };
    unsafe { texture2d_apply(new_texture, false, None) };
    
    let rect = Rect::new(0.0, 0.0, 48 as f32, 48 as f32);
    let pivot = Vector2::new(0.5, 0.5);

    Sprite::create2(new_texture, rect, pivot, 100.0, 1, SpriteMeshType::Tight)
}

#[skyline::from_offset(0x378bb90)]
fn texture2d_ctor2(this: &Texture2D, width: i32, height: i32, format: i32, mip_chain: bool, method_info: OptionalMethod);

#[unity::from_offset("UnityEngine", "Texture2D", "SetPixelDataImplArray")]
fn texture2d_set_pixel_data_impl_array(this: &Texture2D, data: &'static mut Array<u8>, mip_level: i32, element_size: i32, data_array_size: i32, source_data_start_index: i32, method_info: OptionalMethod) -> bool;

#[unity::from_offset("UnityEngine", "Texture2D", "Apply")]
fn texture2d_apply(this: &Texture2D, update_mipmaps: bool, method_info: OptionalMethod);

#[unity::from_offset("UnityEngine", "Texture", "set_anisoLevel")]
fn texture2d_set_anisolevel(this: &Texture2D, level: i32, method_info: OptionalMethod);

#[unity::from_offset("UnityEngine", "Texture2D", "GetRawTextureData")]
fn texture2d_get_raw_texture_data(this: &Texture2D, method_info: OptionalMethod) -> &'static mut Array<u8>;

#[skyline::from_offset(0x2f979a0)]
fn sprite_get_rect(this: &Sprite, method_info: OptionalMethod) -> Rect;

#[skyline::from_offset(0x2f97c40)]
fn sprite_get_uv(this: &Sprite, method_info: OptionalMethod) -> &'static mut Array<Vector2<f32>>;

#[unity::from_offset("UnityEngine", "ImageConversion", "EncodeToPNG")]
fn imageconversion_encode_to_png(tex: &Texture2D, method_info: OptionalMethod) -> &'static mut Array<u8>;

pub fn make_palette(uniticon: &RgbaImage) -> RgbaImage {
    let mut palette = ImageBuffer::new(512, 1);
    let mut index = 0;

    let transparency_slice: &[u8; 4] = &[0, 0, 0, 0];
    let transparency = Rgba::from_slice(transparency_slice);


    palette.put_pixel(index*2, 0, transparency.to_owned());
    palette.put_pixel(index*2+1, 0, transparency.to_owned());
    
    index += 1;

    for w in 0..uniticon.width() {
        for h in 0..uniticon.height() {
            let pixel = uniticon.get_pixel(w, h);

            let mut newcolor = true;

            for x in 0..palette.width() as usize {
                if pixel == palette.get_pixel(x as u32, 0) {
                    newcolor = false;
                    break;
                }
            }

            if newcolor {
                if index >= 256 {
                    panic!("Palette has too many colors.");
                }

                palette.put_pixel(index*2, 0, pixel.to_owned());
                palette.put_pixel(index*2+1, 0, pixel.to_owned());

                index += 1;
            }
        }
    }

    palette
}

pub fn make_r8(uniticon: &RgbaImage, palette: &RgbaImage) -> GrayImage {
    let mut r8_icon = ImageBuffer::new(uniticon.width(), uniticon.height());
    
    for w in 0..uniticon.width() {
        for h in 0..uniticon.height() {
            let pixel = uniticon.get_pixel(w, h);

            for y in 0..palette.width() as usize {
                let index = y as u8;
                let color = palette.get_pixel(index as u32 * 2, 0);

                if pixel == color {
                    let r8_slice = [index];
                    let r8_color = Luma::from_slice(&r8_slice);
                    r8_icon.put_pixel(w, h, r8_color.to_owned());
                    break;
                }
            }
        }
    }
    
    r8_icon
}

#[unity::hook("App", "GameIcon", "TryGetSkill")]
pub fn get_skill_icon(name: Option<&Il2CppString>, method_info: OptionalMethod)-> &'static mut Sprite {
    let icon = load_sprite(name, "patches/icon/skill", 56, 56, FilterMode::Trilinear);
    match icon {
        Some(sprite) => sprite,
        None => call_original!(name, method_info),
    }
}

//#[unity::hook("App", "GameIcon", "TryGetItem")]
#[skyline::hook(offset = 0x227cd50)]
pub fn get_item_icon_string(iconname: Option<&Il2CppString>, method_info: OptionalMethod)-> &'static mut Sprite {
    let icon = load_sprite(iconname, "patches/icon/item", 64, 64, FilterMode::Trilinear);
    match icon {
        Some(sprite) => sprite,
        None => call_original!(iconname, method_info),
    }
}

//#[unity::hook("App", "GameIcon", "TryGetItem")]
#[skyline::hook(offset = 0x227cdd0)]
pub fn get_item_icon_itemdata(item: &ItemData, method_info: OptionalMethod)-> &'static mut Sprite {
	// println!("Item Icon: {}", item.icon.unwrap().to_string());

    let icon = load_sprite(item.icon, "patches/icon/item", 64, 64, FilterMode::Trilinear);
    match icon {
        Some(sprite) => sprite,
        None => call_original!(item, method_info),
    }
}

#[skyline::from_offset(0x2d51d80)]
pub fn facethumbnail_getpath_unit(unit: &mut Unit, method_info: OptionalMethod) -> Option<&'static Il2CppString>;

#[skyline::from_offset(0x2d52340)]
pub fn facethumbnail_getpath_god(god: &mut GodData, method_info: OptionalMethod) -> Option<&'static Il2CppString>;

#[skyline::hook(offset = 0x2d51cb0)]
pub fn facethumb_get_unit(unit: &mut Unit, method_info: OptionalMethod)-> &'static mut Sprite {
    let facethumb_path = unsafe { facethumbnail_getpath_unit(unit, None).unwrap() };

    let icon = load_sprite(Some(facethumb_path), "patches/icon/facethumb/unit", 188, 74, FilterMode::Trilinear);
    match icon {
        Some(sprite) => sprite,
        None => call_original!(unit, method_info),
    }
}

#[skyline::hook(offset = 0x2d52270)]
pub fn facethumb_get_god(god: &mut GodData, method_info: OptionalMethod)-> &'static mut Sprite {
    let facethumb_path = unsafe { facethumbnail_getpath_god(god, None) }.unwrap();

    let icon = load_sprite(Some(facethumb_path), "patches/icon/facethumb/emblem", 188, 74, FilterMode::Trilinear);
    match icon {
        Some(sprite) => sprite,
        None => call_original!(god, method_info),
    }
}

#[skyline::hook(offset = 0x2d52620)]
pub fn facethumb_get_ring(ring: &mut RingData, method_info: OptionalMethod)-> &'static mut Sprite {
    let facethumb_path = ring.icon;

    let icon = load_sprite(Some(facethumb_path), "patches/icon/facethumb/bond", 188, 74, FilterMode::Trilinear);
    match icon {
        Some(sprite) => sprite,
        None => call_original!(ring, method_info),
    }
}

#[skyline::hook(offset = 0x298ac80)]
pub fn bondsringfacepicture_get(ring: &mut RingData, method_info: OptionalMethod)-> &'static mut Sprite {
    let facethumb_path = ring.icon;

    let icon = load_sprite(Some(facethumb_path), "patches/icon/face", 300, 300, FilterMode::Point);
    match icon {
        Some(sprite) => sprite,
        None => call_original!(ring, method_info),
    }
}

#[skyline::hook(offset = 0x232F8A0)]
pub fn godfacepicture_getsprite(god: &mut GodData, method_info: OptionalMethod) -> &'static mut Sprite {
    let ascii_name = god.get_ascii_name();

    let icon = load_sprite(ascii_name, "patches/icon/emblem/godface", 388, 388, FilterMode::Point);
    match icon {
        Some(sprite) => sprite,
        None => call_original!(god, method_info),
    }
}

#[skyline::hook(offset = 0x227D2F0)]
pub fn gameicon_trygetgodring_unit(unit: &mut Unit, method_info: OptionalMethod) -> &'static mut Sprite {
    match unit.get_god_unit() {
        Some(god) => {
            let ascii_name = god.data.get_ascii_name();
            // println!("TryGetGodRing(Unit unit) => {}", ascii_name.unwrap().to_string());

            let icon = load_sprite(ascii_name, "patches/icon/emblem/godring", 74, 74, FilterMode::Trilinear);
            match icon {
                Some(sprite) => sprite,
                None => call_original!(unit, method_info),
            }
        },

        None => call_original!(unit, method_info),
    }
}

#[skyline::hook(offset = 0x227D480)]
pub fn gameicon_trygetgodring_god(god: &mut GodData, method_info: OptionalMethod) -> &'static mut Sprite {
    let ascii_name = god.get_ascii_name();
    // println!("TryGetGodRing(GodData godData) => {}", ascii_name.unwrap().to_string());

    let icon = load_sprite(ascii_name, "patches/icon/emblem/godring", 74, 74, FilterMode::Trilinear);
    match icon {
        Some(sprite) => sprite,
        None => call_original!(god, method_info),
    }
}

#[skyline::hook(offset = 0x227D290)]
pub fn gameicon_trygetgodsymbol(god: &mut GodData, method_info: OptionalMethod) -> &'static mut Sprite {
    let ascii_name = god.get_ascii_name();

    let icon = load_sprite(ascii_name, "patches/icon/emblem/godsymbol", 72, 72, FilterMode::Trilinear);
    match icon {
        Some(sprite) => sprite,
        None => call_original!(god, method_info),
    }
}

fn hex_to_rgba(hex: String) -> Color {
    let hex_str = hex.trim_start_matches('#'); // Trim # if present
    let default_color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }; // Default to white if invalid format

    // Check for invalid hex digits
    if hex_str.chars().any(|c| !c.is_digit(16)) {
        return default_color;
    }

    match hex_str.len() {
        6 => { // RGB hex format
            let hex_u32 = u32::from_str_radix(&format!("FF{}", hex_str), 16).unwrap();
            unsafe { colorutils_to_rgba(hex_u32) }
        },
        8 => { // RGBA hex format
            let (rgb, alpha) = hex_str.split_at(6);
            let hex_u32 = u32::from_str_radix(&format!("{}{}", alpha, rgb), 16).unwrap();
            unsafe { colorutils_to_rgba(hex_u32) }
        },
        _ => default_color, 
    }
}

#[skyline::from_offset(0x3530580)]
fn colorutils_to_rgba(hex: u32) -> Color;

#[unity::hook("App", "GodColorRefineEmblem", "GetColor")]
pub fn godcolorrefineemblem_getcolor(_this: &(), god: &mut GodData, method_info: OptionalMethod) -> Color {
    let ascii_name = god.get_ascii_name().unwrap().to_string();
    let path = Utf8PathBuf::from("patches/icon/emblem/godsymbol")
            .join(ascii_name)
            .with_extension("txt");

    match mods::manager::Manager::get().get_file(&path) {
        Ok(hex_file) => match String::from_utf8(hex_file) {
            Ok(hex_string) => hex_to_rgba(hex_string),
            _ => call_original!(_this, god, method_info),
        },
        _ => call_original!(_this, god, method_info),
    }
}

#[repr(C)]
#[unity::class("", "MapUIGauge")]
pub struct MapUIGauge<'a> {
    _padding: [u8; 0x48],
    m_sprites: &'a mut Il2CppArray<&'a mut Sprite>,
    m_dictionary: &'a mut Dictionary<&'a Il2CppString, &'a mut Sprite>,
}

#[repr(C)]
pub struct MapUIGaugeStaticFields {
    pub icon_names: &'static mut Array<&'static mut Il2CppString>,
}

#[skyline::hook(offset = 0x201f8d0)]
pub fn mapuigauge_geticonindex(this: &MapUIGauge, name: Option<&'static mut Il2CppString>, method_info: OptionalMethod) -> i32 {
    let name = name.unwrap();

    let icons = this.get_class().get_static_fields_mut::<MapUIGaugeStaticFields>();

    // Check if the name of the sprite is already in the game's cache.
    if icons.icon_names.iter().find(|icon_name| **icon_name == name).is_none() {
        // Since our array stuff is poopoo, convert it to a Vec so we can append to it.
        let mut temp = icons.icon_names.to_vec();
        temp.push(name);

        // Turn the Vec back into a Il2CppArray
        icons.icon_names = Il2CppArray::from_slice(&mut temp).unwrap();

        // Return the latest index we just added in stead of the original function
        (icons.icon_names.len() - 1) as i32
    } else {
        call_original!(this, Some(name), method_info)
    }    
}

#[skyline::hook(offset = 0x201F830)]
pub fn mapuigauge_getspritebyname(this: &MapUIGauge, name: Option<&Il2CppString>, method_info: OptionalMethod) -> &'static mut Sprite {
    let mut result = Sprite::instantiate().unwrap();

    if this.m_dictionary.try_get_value(name.unwrap(), &mut result) {
        return call_original!(this, name, method_info);
    }

    let icon = load_sprite(name, "patches/icon/mapstatus", 0, 0, FilterMode::Point);
    match icon {
        Some(sprite) => {
            this.m_dictionary.add(name.unwrap(), sprite);
            sprite
        },
        None => call_original!(this, name, method_info),
    }
}

// TODO: Probably try to store the sprite in the Sprites array instead for speed when grabbing it again.
#[skyline::hook(offset = 0x201f7d0)]
pub fn mapuigauge_getspritebyindex(this: &'static mut MapUIGauge, index: usize, _method_info: OptionalMethod) -> &'static mut Sprite {
    if index <= 0xa6 {
        this.m_sprites[index]
    } else {
        // Check if the Sprite cache has already been expanded for our sprite
        if this.m_sprites.len() - 1 < index {
            // Grab the name of the sprite we stored in MapUIGauge::GetIconIndex
            let icons = this.get_class().get_static_fields_mut::<MapUIGaugeStaticFields>();
            let sprite_name = &icons.icon_names[index];

            let icon = load_sprite(Some(&sprite_name), "patches/icon/mapstatus", 0, 0, FilterMode::Bilinear).expect("could not find mapuigauge status sprite despite being added in cache??");

            // Since our array stuff is poopoo, convert it to a Vec so we can append to it.
            let mut temp = this.m_sprites.to_vec();
            temp.push(icon);

            // Turn the Vec back into a Il2CppArray
            this.m_sprites = Array::from_slice(&mut temp).unwrap();
        }

        this.m_sprites[index]
    }
}

#[unity::class("App", "GmapMapInfoContent")]
pub struct GmapMapInfoContent {
    unk1: [u8; 0x18],
    pub map_info_image: &'static mut Image,
    unk2: [u8;0x90],
    pub map_info_sprite: &'static Sprite,
    // ...
}

#[unity::class("App", "GmapSpot")]
pub struct GmapSpot { }

impl GmapSpot {
    pub fn get_chapter(&self) -> &'static mut ChapterData {
        unsafe { gmapspot_get_chapter(self, None) }
    }
}

#[unity::from_offset("App", "GmapSpot", "get_Chapter")]
pub fn gmapspot_get_chapter(this: &GmapSpot, method_info: OptionalMethod) -> &'static mut ChapterData;

#[unity::hook("App", "GmapMapInfoContent", "SetMapInfo")]
pub fn gmapinfocontent_setmapinfo_hook(this: &mut GmapMapInfoContent, gmap_spot: &GmapSpot, method_info: OptionalMethod) {
    // Call it first so the game can assign everything that is not related to the InfoThumb.
    call_original!(this, gmap_spot, method_info);

    let chapter = gmap_spot.get_chapter();
    let prefixless_cid = chapter.get_prefixless_cid();

    if let Some(infothumb) = load_sprite(Some(prefixless_cid), "patches/ui/gmap/infothumb", 468, 256, FilterMode::Trilinear) {
        this.map_info_sprite = infothumb;
        this.map_info_image.set_sprite(infothumb);
    }
}