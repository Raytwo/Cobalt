use engage::app::basicdialogitemyes::BasicDialogItemYes;
use engage::app::basicmenuitem::{BasicMenuItem, IBasicMenuItemMethods};
use engage::app::database::Database;
use engage::app::fade::{Fade, Fade_Layer as FadeLayer};
use engage::app::gametime::GameTime_VsycMode;
use engage::app::proc::Proc;
use engage::app::procdesc::ProcDesc;
use engage::app::procinst::{IProcInstMethods, ProcInst};
use engage::app::procvoidmethod::ProcVoidMethod;
use engage::app::yesnodialog::YesNoDialog;
use unity::{Array, Cast, ClassIdentity, FromIlInstance, OptionalMethod};

use engage::{
    BasicMenuItemAttribute, BasicMenuItemMethods, BasicMenuResult, ProcExt, ProcVoidMethodExt,
    TwoChoiceDialogMethods, YesNoDialogExt,
};

pub struct ReloadXmlMenuItem;

impl BasicMenuItemMethods for ReloadXmlMenuItem {
    extern "C" fn get_name(_this: BasicMenuItem, _method_info: OptionalMethod) -> unity::Il2CppString {
        localize::mess::get("reload_game_data").into()
    }

    extern "C" fn a_call(this: BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
        YesNoDialog::bind_with::<ReloadXmlConfirmDialog>(
            this.get_menu(),
            localize::mess::get("reload_game_data_text"),
            localize::mess::get("reload_game_data_proceed"),
            localize::mess::get("reload_game_data_cancel"),
        );

        BasicMenuResult::se_decide()
    }

    extern "C" fn build_attribute(
        _this: BasicMenuItem,
        _method_info: OptionalMethod,
    ) -> BasicMenuItemAttribute {
        BasicMenuItemAttribute::enable()
    }
}

pub struct ReloadXmlConfirmDialog;

impl TwoChoiceDialogMethods for ReloadXmlConfirmDialog {
    extern "C" fn on_first_choice(
        this: BasicDialogItemYes,
        _method_info: OptionalMethod,
    ) -> BasicMenuResult {
        let item: BasicMenuItem = this.into();
        let menu_as_proc: ProcInst = item.get_menu().into();
        let parent = menu_as_proc.get_super();
        if !parent.is_null() {
            XmlReloadSequence::bind(parent);
        }

        BasicMenuResult::se_decide().with_close_this(true)
    }
}

pub struct XmlReloadSequence;

impl XmlReloadSequence {
    pub extern "C" fn enable_boost_mode(_parent: ProcInst, _method_info: OptionalMethod) {
        unsafe {
            skyline::nn::oe::SetCpuBoostMode(skyline::nn::oe::CpuBoostMode::Boost);
        }
    }

    pub extern "C" fn reload(_parent: ProcInst, _method_info: OptionalMethod) {
        let reload = std::time::Instant::now();
        Database::release();
        Database::load();
        println!("Release and load took {}ms", reload.elapsed().as_millis());
    }

    pub extern "C" fn reload_complete(_parent: ProcInst, _method_info: OptionalMethod) {
        let reload = std::time::Instant::now();
        Database::completed();
        println!("Completed took {}ms", reload.elapsed().as_millis());
    }

    pub extern "C" fn disable_boost_mode(_parent: ProcInst, _method_info: OptionalMethod) {
        println!("boost mode off");
        unsafe {
            skyline::nn::oe::SetCpuBoostMode(skyline::nn::oe::CpuBoostMode::Disabled);
        }
    }

    pub fn bind(parent: ProcInst) {
        let proc = <ProcInst as FromIlInstance>::instantiate()
            .expect("XmlReloadSequence: ProcInst allocation failed");

        let descs = [
            Fade::black_out(0.5, FadeLayer::current()),
            Proc::wait_time(0.5),
            Proc::vsync(GameTime_VsycMode::slow()),
            Proc::call_method(
                ProcVoidMethod::from_fn(proc.into(), Self::enable_boost_mode).unwrap(),
            ),
            Proc::call_method(
                ProcVoidMethod::from_fn(proc, Self::reload).unwrap(),
            ),
            Proc::wait_is_loading(),
            Proc::call_method(
                ProcVoidMethod::from_fn(proc, Self::reload_complete).unwrap(),
            ),
            Proc::call_method(
                ProcVoidMethod::from_fn(proc.into(), Self::disable_boost_mode).unwrap(),
            ),
            Proc::vsync(GameTime_VsycMode::normal()),
            Fade::black_in(0.5, FadeLayer::current()),
            Proc::end(),
        ];

        let arr = Array::<ProcDesc>::new(<ProcDesc as ClassIdentity>::class().raw(), descs.len())
            .expect("XmlReloadSequence: ProcDesc[] allocation failed");
        for (i, d) in descs.iter().enumerate() {
            arr.set(i, *d);
        }
        proc.create_bind(parent, arr, unity::Il2CppString::from("XmlReloadSequence"));
    }
}
