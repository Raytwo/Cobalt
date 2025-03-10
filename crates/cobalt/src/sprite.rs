use std::{
    alloc::Layout,
    ops::Deref,
    sync::OnceLock
};

use engage::{
    gamedata::{dispos::ChapterData, item::ItemData, ring::RingData, unit::Unit, GodData},
    uniticon::UnitIcon,
};

use camino::Utf8PathBuf;

use unity::{
    engine::{
        ui::{Image, IsImage},
        Color, FilterMode, ImageConversion, Material, Rect, Sprite, SpriteMeshType, Texture2D, Vector2,
    }, prelude::*, system::Dictionary
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

static mut SPRITE_MATERIAL: OnceLock<&'static Material> = OnceLock::new();

#[inline]
fn try_set_material(this: &mut UnitIcon) {
    if let Some(material) = unsafe { SPRITE_MATERIAL.get() } {
        this.set_material(material);
    }
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

#[unity::hook("App", "UnitIcon", "OnDestroy")]
pub fn icon_destroy(this: &mut UnitIcon, method_info: OptionalMethod) {
    try_set_material(this); // Change material to original before destroying it
    call_original!(this, method_info);
}

// TODO: Investigate why load_sprite is called twice when dealing with the Unit Selection menu or highlighting the unit on a battle map

// #[skyline::hook(offset = 0x227d710)]
#[unity::hook("App", "GameIcon", "TyrGetUnitIconIndex")] // What does this even do?
pub fn trygetuniticonindex(name: Option<&Il2CppString>, method_info: OptionalMethod) -> &'static mut Sprite {
    let icon = load_sprite(name, "patches/icon/job", 48, 48, FilterMode::Point);
    match icon {
        Some(sprite) => sprite,
        None => call_original!(name, method_info),
    }
}

#[unity::hook("App", "UnitIcon", "TrySet")]
pub fn uniticon_tryset_hook(this: &mut UnitIcon, index_name: Option<&Il2CppString>, pallete_name: Option<&Il2CppString>, method_info: OptionalMethod) -> bool {
    let result = if let Some(index_name) = index_name {
        // println!("Icon name: {}", index_name.to_string());
        // println!("Pallete name: {}", pallete_name.unwrap().to_string());

        let icon = load_sprite(Some(index_name), "patches/icon/job", 48, 48, FilterMode::Point);
        match icon {
            Some(sprite) => {
                // Backup up the material
                unsafe { SPRITE_MATERIAL.set(this.get_material()); }

                // Load default material first
                this.set_material(Image::get_default_graphic_material());

                // Assign the new sprite to the UnitIcon
                this.set_sprite(sprite);
                true
            },
            None => {
                try_set_material(this);
                call_original!(this, Some(index_name), pallete_name, method_info)
            }
        }
    } else {
        // If the material was backup'd, restore it
        try_set_material(this);
        call_original!(this, index_name, pallete_name, method_info)
    };

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
    
    result
}

// #[skyline::hook(offset = 0x227d110)]
#[unity::hook("App", "GameIcon", "TryGetSkill")]
pub fn get_skill_icon(name: Option<&Il2CppString>, method_info: OptionalMethod)-> &'static mut Sprite {
	// println!("Skill Name: {}", name.unwrap().to_string());

    let icon = load_sprite(name, "patches/icon/skill", 56, 56, FilterMode::Trilinear);
    match icon {
        Some(sprite) => sprite,
        None => call_original!(name, method_info),
    }
}

//#[unity::hook("App", "GameIcon", "TryGetItem")]
#[skyline::hook(offset = 0x227cd50)]
pub fn get_item_icon_string(iconname: Option<&Il2CppString>, method_info: OptionalMethod)-> &'static mut Sprite {
	// println!("Item Icon: {}", iconname.unwrap().to_string());

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

pub struct AppGodColorRefineEmblemO {}

// #[skyline::hook(offset = 0x2B51BF0)]
#[unity::hook("App", "GodColorRefineEmblem", "GetColor")]
pub fn godcolorrefineemblem_getcolor(_this: &AppGodColorRefineEmblemO, god: &mut GodData, method_info: OptionalMethod) -> Color {
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
pub struct MapUIGauge<'a> {
    _padding: [u8; 0x58],
    m_sprites: &'a Il2CppArray<&'a mut Sprite>,
    m_dictionary: &'a mut Dictionary<&'a Il2CppString, &'a mut Sprite>,
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