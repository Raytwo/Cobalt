use unity::prelude::*;

use crate::sequences::mainmenu::cobaltmenu::sequences::settings::lod::CURRENT_LOD;

#[skyline::hook(offset = 0x2012000)]
pub fn lod_hook(_lod_bias: f32, _lod_bias_on_gpu_saved: f32, method_info: OptionalMethod) {
    // TODO: Add a toggle like we did for Render Scale
    let lod = *CURRENT_LOD.read().unwrap();
    call_original!(lod, lod, method_info)
}
