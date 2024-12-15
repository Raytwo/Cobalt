use unity::prelude::*;
use engage::{proc::{desc::ProcDesc, ProcVoidMethod, Proc, Bindable, ProcInst}, menu::{MenuSequence, BasicMenuResult, BasicMenuItemAttribute, BasicMenuItem, BasicMenuItemMethods}, database::{database_release, database_load, database_completed}, dialog::yesno::{BasicDialogItemYes, TwoChoiceDialogMethods, YesNoDialog}, fade::Fade};

pub struct ReloadXmlMenuItem;

impl BasicMenuItemMethods for ReloadXmlMenuItem {
    extern "C" fn get_name(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> &'static Il2CppString {
        localize::mess::get("reload_game_data").into()
    }

    extern "C" fn a_call(this: &'static mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        YesNoDialog::bind::<ReloadXmlConfirmDialog>(
            this.menu,
            localize::mess::get("reload_game_data_text"),
            localize::mess::get("reload_game_data_proceed"),
            localize::mess::get("reload_game_data_cancel"),
        );

        BasicMenuResult::se_decide()
    }

    extern "C" fn build_attributes(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuItemAttribute {
        BasicMenuItemAttribute::Enable
    }
}

pub struct ReloadXmlConfirmDialog;

impl TwoChoiceDialogMethods for ReloadXmlConfirmDialog {
    extern "C" fn on_first_choice(this: &mut BasicDialogItemYes, _method_info: OptionalMethod) -> engage::menu::BasicMenuResult {
        XmlReloadSequence::bind(*this.fields.parent.parent.menu.proc.parent.as_ref().unwrap());

        BasicMenuResult::se_decide().with_close_this(true)
    }
}

pub struct XmlReloadSequence;

impl XmlReloadSequence {
    pub extern "C" fn enable_boost_mode(_parent: &mut ProcInst, _method_info: OptionalMethod) {
        unsafe { skyline::nn::oe::SetCpuBoostMode(skyline::nn::oe::CpuBoostMode::Boost); }
    }

    pub extern "C" fn reload(_parent: &mut impl Bindable, _method_info: OptionalMethod) {
        unsafe {
            let reload = std::time::Instant::now();
        
            database_release(None);
            database_load(None);

            println!("Reload took {}ms", reload.elapsed().as_millis());
        }
    }

    pub extern "C" fn reload_complete(_parent: &mut impl Bindable, _method_info: OptionalMethod) {
        unsafe {
            database_completed(None);
            
        }
    }

    pub extern "C" fn disable_boost_mode(_parent: &mut ProcInst, _method_info: OptionalMethod) {
        println!("boost mode off");
        unsafe { skyline::nn::oe::SetCpuBoostMode(skyline::nn::oe::CpuBoostMode::Disabled); }
    }
}

impl MenuSequence for XmlReloadSequence {
    fn get_proc_desc(this: &'static ProcInst) -> Vec<&'static mut ProcDesc> {
        // Load resources here
        vec![
            Fade::black_out(0.5, 4),
            ProcDesc::wait_time(0.5),
            // Set vsync to Slow
            // descs.push(unsafe { proc_vsync(1, None) });
            ProcDesc::call(ProcVoidMethod::new(None, Self::enable_boost_mode)),
            // Loop until the resources are loaded
            ProcDesc::call(ProcVoidMethod::new(this, Self::reload)),
            ProcDesc::call(ProcVoidMethod::new(this, Self::reload_complete)),
            ProcDesc::call(ProcVoidMethod::new(None, Self::disable_boost_mode)),
            // Restore vsync to Normal
            Proc::vsync(0),
            Fade::black_in(0.5, 4),
            ProcDesc::end()
        ]
    }

    fn proc_name() -> &'static str {
        "XmlReloadSequence"
    }
}

