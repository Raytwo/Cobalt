use unity::prelude::*;

use crate::sequences::mainmenu::cobaltmenu::sequences::settings::lod::get_lod;

#[skyline::hook(offset = 0x2012000)]
pub fn lod_hook(lod_bias: f32, lod_bias_on_gpu_saved: f32, method_info: OptionalMethod) {
    let lod = get_lod();
    match lod {
        Some(lod) => {
            call_original!(lod, lod, method_info)
        },
        None => {
            call_original!(lod_bias, lod_bias_on_gpu_saved, method_info)
        },
    }
}
