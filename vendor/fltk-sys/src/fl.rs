/* automatically generated by rust-bindgen */

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Fl_Widget {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Fl_Widget_Tracker {
    _unused: [u8; 0],
}
pub type Fl_Awake_Handler =
    ::std::option::Option<unsafe extern "C" fn(data: *mut ::std::os::raw::c_void)>;
extern "C" {
    pub fn Fl_run() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_lock() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_unlock();
}
extern "C" {
    pub fn Fl_awake(
        handler: Fl_Awake_Handler,
        data: *mut ::std::os::raw::c_void,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_key() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_text() -> *const ::std::os::raw::c_char;
}
extern "C" {
    pub fn Fl_event_button() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_clicks() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_x() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_y() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_x_root() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_y_root() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_dx() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_dy() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_get_mouse(arg1: *mut ::std::os::raw::c_int, arg2: *mut ::std::os::raw::c_int);
}
extern "C" {
    pub fn Fl_event_is_click() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_length() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_state() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_screen_h() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_screen_w() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_paste(arg1: *mut Fl_Widget, src: ::std::os::raw::c_int);
}
extern "C" {
    pub fn Fl_set_scheme(scheme: *const ::std::os::raw::c_char);
}
extern "C" {
    pub fn Fl_get_color(
        r: ::std::os::raw::c_uchar,
        g: ::std::os::raw::c_uchar,
        b: ::std::os::raw::c_uchar,
    ) -> ::std::os::raw::c_uint;
}
extern "C" {
    pub fn Fl_get_font(idx: ::std::os::raw::c_int) -> *const ::std::os::raw::c_char;
}
extern "C" {
    pub fn Fl_set_fonts(c: *const ::std::os::raw::c_char) -> ::std::os::raw::c_uchar;
}
extern "C" {
    pub fn Fl_add_handler(
        ev_handler: ::std::option::Option<
            unsafe extern "C" fn(ev: ::std::os::raw::c_int) -> ::std::os::raw::c_int,
        >,
    );
}
extern "C" {
    pub fn Fl_awake_msg(msg: *mut ::std::os::raw::c_void);
}
extern "C" {
    pub fn Fl_thread_msg() -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn Fl_wait() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_add_timeout(
        t: f64,
        arg1: ::std::option::Option<unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void)>,
        arg2: *mut ::std::os::raw::c_void,
    );
}
extern "C" {
    pub fn Fl_repeat_timeout(
        t: f64,
        arg1: ::std::option::Option<unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void)>,
        arg2: *mut ::std::os::raw::c_void,
    );
}
extern "C" {
    pub fn Fl_remove_timeout(
        arg1: ::std::option::Option<unsafe extern "C" fn(arg1: *mut ::std::os::raw::c_void)>,
        arg2: *mut ::std::os::raw::c_void,
    );
}
extern "C" {
    pub fn Fl_dnd() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_first_window() -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn Fl_next_window(arg1: *const ::std::os::raw::c_void) -> *mut ::std::os::raw::c_void;
}
extern "C" {
    pub fn Fl_should_program_quit() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_program_should_quit(flag: ::std::os::raw::c_int);
}
extern "C" {
    pub fn Fl_rand() -> ::std::os::raw::c_uint;
}
extern "C" {
    pub fn Fl_event_inside(
        arg1: ::std::os::raw::c_int,
        arg2: ::std::os::raw::c_int,
        arg3: ::std::os::raw::c_int,
        arg4: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_belowmouse() -> *mut Fl_Widget;
}
extern "C" {
    pub fn Fl_delete_widget(w: *mut Fl_Widget);
}
extern "C" {
    pub fn Fl_Widget_Tracker_new(w: *mut Fl_Widget) -> *mut Fl_Widget_Tracker;
}
extern "C" {
    pub fn Fl_Widget_Tracker_deleted(self_: *mut Fl_Widget_Tracker) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_Widget_Tracker_delete(self_: *mut Fl_Widget_Tracker);
}
extern "C" {
    pub fn Fl_init_all();
}
extern "C" {
    pub fn Fl_redraw();
}
extern "C" {
    pub fn Fl_event_shift() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_ctrl() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_command() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_event_alt() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_set_damage(flag: ::std::os::raw::c_int);
}
extern "C" {
    pub fn Fl_damage() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_visual(arg1: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_own_colormap();
}
extern "C" {
    pub fn Fl_pushed() -> *mut Fl_Widget;
}
extern "C" {
    pub fn Fl_focus() -> *mut Fl_Widget;
}
extern "C" {
    pub fn Fl_set_focus(arg1: *mut ::std::os::raw::c_void);
}
extern "C" {
    pub fn Fl_version() -> f64;
}
extern "C" {
    pub fn Fl_api_version() -> ::std::os::raw::c_int;
}
extern "C" {
    pub fn Fl_abi_version() -> ::std::os::raw::c_int;
}
