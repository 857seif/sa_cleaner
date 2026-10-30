#![windows_subsystem = "windows"]

use std::ptr;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

type HWND = isize;
type HINSTANCE = isize;
type HBRUSH = isize;
type HDC = isize;
type HMENU = isize;
type WPARAM = usize;
type LPARAM = isize;
type LRESULT = isize;
type UINT = u32;
type BOOL = i32;
type DWORD = u32;
type COLORREF = u32;
type HFONT = isize;
type HPEN = isize;

const WM_CREATE: UINT = 0x0001;
const WM_COMMAND: UINT = 0x0111;
const WM_PAINT: UINT = 0x000F;
const WM_TIMER: UINT = 0x0113;
const WM_DESTROY: UINT = 0x0002;
const WM_SIZE: UINT = 0x0005;
const WM_CTLCOLORSTATIC: UINT = 0x0138;
const WM_CTLCOLORBTN: UINT = 0x0135;
const WM_ERASEBKGND: UINT = 0x0014;
const BM_GETCHECK: UINT = 0x00F0;
const BM_SETCHECK: UINT = 0x00F1;
const BST_CHECKED: WPARAM = 1;
const BST_UNCHECKED: WPARAM = 0;
const BS_AUTOCHECKBOX: DWORD = 0x00000003;
const BS_PUSHBUTTON: DWORD = 0x00000000;
const BS_DEFPUSHBUTTON: DWORD = 0x00000001;
const SS_LEFT: DWORD = 0x00000000;
const SS_CENTER: DWORD = 0x00000001;
const ES_NUMBER: DWORD = 0x2000;
const ES_AUTOHSCROLL: DWORD = 0x0080;
const WS_BORDER: DWORD = 0x00800000;
const WS_CHILD: DWORD = 0x40000000;
const WS_VISIBLE: DWORD = 0x10000000;
const WS_OVERLAPPEDWINDOW: DWORD = 0x00CF0000;
const WS_FIXED_FRAME: DWORD = 0x00CA0000; // caption+sysmenu+minimize, no maximize/size
const WS_TABSTOP: DWORD = 0x00010000;
const WS_GROUP: DWORD = 0x00020000;
const SW_SHOW: i32 = 5;
const COLOR_WINDOW: i32 = 5;
const DT_LEFT: UINT = 0x0000;
const DT_CENTER: UINT = 0x0001;
const DT_VCENTER: UINT = 0x0004;
const DT_SINGLELINE: UINT = 0x0020;
const DT_WORDBREAK: UINT = 0x0010;
const MB_OK: UINT = 0x00000000;
const MB_ICONINFORMATION: UINT = 0x00000040;
const MB_ICONERROR: UINT = 0x00000010;
const MB_ICONWARNING: UINT = 0x00000030;
const MB_YESNO: UINT = 0x00000004;
const IDOK: i32 = 1;
const IDYES: i32 = 6;
const FW_BOLD: i32 = 700;
const FW_NORMAL: i32 = 400;
const DEFAULT_CHARSET: u32 = 1;
const OUT_DEFAULT_PRECIS: u32 = 0;
const CLIP_DEFAULT_PRECIS: u32 = 0;
const CLEARTYPE_QUALITY: u32 = 5;
const DEFAULT_PITCH: u32 = 0;
const FF_DONTCARE: u32 = 0;
const PS_SOLID: i32 = 0;
const TRANSPARENT: i32 = 1;
const SRCCOPY: DWORD = 0x00CC0020;

const WM_TRAYICON: UINT = 0x8001;
const WM_APP_CLEAN_DONE: UINT = 0x8002;
const WM_SYSCOMMAND: UINT = 0x0112;
const SC_MINIMIZE: WPARAM = 0xF020;
const SC_CLOSE: WPARAM = 0xF060;
const NIM_ADD: DWORD = 0x00000000;
const NIM_DELETE: DWORD = 0x00000002;
const NIF_MESSAGE: DWORD = 0x00000001;
const NIF_ICON: DWORD = 0x00000002;
const NIF_TIP: DWORD = 0x00000004;
const IDI_APPLICATION: usize = 32512;
const SW_HIDE: i32 = 0;
const SW_RESTORE: i32 = 9;

#[repr(C)]
struct NOTIFYICONDATAW {
    cb_size: DWORD,
    h_wnd: HWND,
    u_id: UINT,
    u_flags: UINT,
    u_callback_message: UINT,
    h_icon: isize,
    sz_tip: [u16; 128],
}

#[link(name = "shell32")]
extern "system" {
    fn Shell_NotifyIconW(dw_message: DWORD, lp_data: *mut NOTIFYICONDATAW) -> BOOL;
    fn ExtractIconW(h_inst: HINSTANCE, lpsz_exe: *const u16, n_icon_index: UINT) -> isize;
}

#[link(name = "user32")]
extern "system" {
    fn LoadIconW(h_instance: HINSTANCE, lp_icon_name: usize) -> isize;
    fn IsWindowVisible(h_wnd: HWND) -> BOOL;
}


const IDC_TITLE: i32 = 100;
const IDC_MEM_INFO: i32 = 101;
const IDC_STANDBY: i32 = 102;
const IDC_STATUS: i32 = 103;
const IDC_ESTIMATE: i32 = 104;
const IDC_CHK_WS: i32 = 201;
const IDC_CHK_CACHE: i32 = 202;
const IDC_CHK_STANDBY: i32 = 203;
const IDC_CHK_MODIFIED: i32 = 204;
const IDC_CHK_REGISTRY: i32 = 205;
const IDC_CHK_COMBINE: i32 = 206;
const IDC_CHK_LOWPRIO: i32 = 207;
const IDC_CHK_CLIP: i32 = 208;
const IDC_CHK_DNS: i32 = 209;
const IDC_CHK_TEMP: i32 = 210;
const IDC_CHK_PREFETCH: i32 = 211;
const IDC_CHK_THUMB: i32 = 212;
const IDC_CHK_FLUSH: i32 = 213;
const IDC_BTN_CLEAN: i32 = 301;
const IDC_BTN_AUTO: i32 = 302;
const IDC_BTN_STOP_AUTO: i32 = 303;
const IDC_BTN_AGGRESSIVE: i32 = 304;
const IDC_BTN_SELECT_ALL: i32 = 305;
const IDC_BTN_SELECT_SAFE: i32 = 306;
const IDC_AUTO_LABEL: i32 = 401;
const IDC_PROC_COUNT: i32 = 402;
const IDC_LAST_FREED: i32 = 403;
const IDC_BTN_THEME: i32 = 307;
const IDC_BTN_BG: i32 = 308;
const IDC_HELP: i32 = 404;
const IDC_THRESHOLD_EDIT: i32 = 405;
const IDC_THRESHOLD_LABEL: i32 = 406;
const IDC_BTN_TRAY: i32 = 309;

const CLR_DARK_BG: COLORREF = 0x00000000;
const CLR_DARK_PANEL: COLORREF = 0x00121212;
const CLR_DARK_TEXT: COLORREF = 0x00F0F0F0;
const CLR_DARK_MUTED: COLORREF = 0x00999999;
const CLR_LIGHT_BG: COLORREF = 0x00F5F5F5;
const CLR_LIGHT_PANEL: COLORREF = 0x00FFFFFF;
const CLR_LIGHT_TEXT: COLORREF = 0x00111111;
const CLR_LIGHT_MUTED: COLORREF = 0x00666666;
const CLR_ACCENT: COLORREF = 0x00E94560;
const CLR_ACCENT2: COLORREF = 0x0000AAFF;
const CLR_SUCCESS: COLORREF = 0x0000C853;

#[repr(C)]
struct WNDCLASSEXW {
    cb_size: UINT,
    style: UINT,
    lpfn_wnd_proc: usize,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: HINSTANCE,
    h_icon: isize,
    h_cursor: isize,
    hbr_background: HBRUSH,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: isize,
}

#[repr(C)]
struct MSG {
    hwnd: HWND,
    message: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
    time: DWORD,
    pt_x: i32,
    pt_y: i32,
}

#[repr(C)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct PAINTSTRUCT {
    hdc: HDC,
    f_erase: BOOL,
    rc_paint: RECT,
    f_restore: BOOL,
    f_inc_update: BOOL,
    rgb_reserved: [u8; 32],
}

#[link(name = "user32")]
extern "system" {
    fn RegisterClassExW(wc: *const WNDCLASSEXW) -> u16;
    fn CreateWindowExW(
        dw_ex_style: DWORD, lp_class_name: *const u16, lp_window_name: *const u16,
        dw_style: DWORD, x: i32, y: i32, n_width: i32, n_height: i32,
        h_wnd_parent: HWND, h_menu: HMENU, h_instance: HINSTANCE, lp_param: *mut u8,
    ) -> HWND;
    fn ShowWindow(h_wnd: HWND, n_cmd_show: i32) -> BOOL;
    fn UpdateWindow(h_wnd: HWND) -> BOOL;
    fn GetMessageW(lp_msg: *mut MSG, h_wnd: HWND, w_msg_filter_min: UINT, w_msg_filter_max: UINT) -> BOOL;
    fn TranslateMessage(lp_msg: *const MSG) -> BOOL;
    fn DispatchMessageW(lp_msg: *const MSG) -> LRESULT;
    fn DefWindowProcW(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
    fn PostQuitMessage(n_exit_code: i32);
    fn BeginPaint(h_wnd: HWND, lp_paint: *mut PAINTSTRUCT) -> HDC;
    fn EndPaint(h_wnd: HWND, lp_paint: *const PAINTSTRUCT) -> BOOL;
    fn FillRect(h_dc: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    fn SetTimer(h_wnd: HWND, n_id_event: usize, u_elapse: UINT, lp_timer_func: usize) -> usize;
    fn KillTimer(h_wnd: HWND, u_id_event: usize) -> BOOL;
    fn InvalidateRect(h_wnd: HWND, lp_rect: *const RECT, b_erase: BOOL) -> BOOL;
    fn GetClientRect(h_wnd: HWND, lp_rect: *mut RECT) -> BOOL;
    fn SendMessageW(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
    fn SetWindowTextW(h_wnd: HWND, lp_string: *const u16) -> BOOL;
    fn GetWindowTextW(h_wnd: HWND, lp_string: *mut u16, n_max_count: i32) -> i32;
    fn MessageBoxW(h_wnd: HWND, lp_text: *const u16, lp_caption: *const u16, u_type: UINT) -> i32;
    fn GetDC(h_wnd: HWND) -> HDC;
    fn ReleaseDC(h_wnd: HWND, h_dc: HDC) -> i32;
    fn DrawTextW(h_dc: HDC, lp_ch: *const u16, n_count: i32, lp_rect: *mut RECT, u_format: UINT) -> i32;
    fn SetTextColor(h_dc: HDC, color: COLORREF) -> COLORREF;
    fn SetBkMode(h_dc: HDC, mode: i32) -> i32;
    fn SelectObject(h_dc: HDC, h_object: isize) -> isize;
    fn CreateSolidBrush(color: COLORREF) -> HBRUSH;
    fn DeleteObject(ho: isize) -> BOOL;
    fn CreatePen(style: i32, width: i32, color: COLORREF) -> HPEN;
    fn MoveToEx(h_dc: HDC, x: i32, y: i32, lp_point: *mut u8) -> BOOL;
    fn LineTo(h_dc: HDC, x: i32, y: i32) -> BOOL;
    fn RoundRect(h_dc: HDC, left: i32, top: i32, right: i32, bottom: i32, width: i32, height: i32) -> BOOL;
    fn GetSysColorBrush(n_index: i32) -> HBRUSH;
    fn LoadCursorW(h_instance: HINSTANCE, lp_cursor_name: usize) -> isize;
    fn SetFocus(h_wnd: HWND) -> HWND;
    fn EnableWindow(h_wnd: HWND, b_enable: BOOL) -> BOOL;
    fn GetDlgItem(h_wnd: HWND, n_id: i32) -> HWND;
    fn PostMessageW(h_wnd: HWND, msg: UINT, w_param: WPARAM, l_param: LPARAM) -> BOOL;
}

#[link(name = "gdi32")]
extern "system" {
    fn CreateFontW(
        c_height: i32, c_width: i32, c_escapement: i32, c_orientation: i32, c_weight: i32,
        b_italic: DWORD, b_underline: DWORD, b_strike_out: DWORD, i_char_set: DWORD,
        i_out_precision: DWORD, i_clip_precision: DWORD, i_quality: DWORD,
        i_pitch_and_family: DWORD, psz_face_name: *const u16,
    ) -> HFONT;
    fn TextOutW(h_dc: HDC, x: i32, y: i32, lp_string: *const u16, c: i32) -> BOOL;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(lp_module_name: *const u16) -> HINSTANCE;
}

#[link(name = "sacleaner_cpp", kind = "static")]
extern "C" {
    fn sa_clean(mask: u32) -> i32;
    fn sa_get_err(buf: *mut i8, len: u32);
    fn sa_get_mem(total: *mut u64, used: *mut u64, free: *mut u64);
    fn sa_is_admin() -> i32;
    fn sa_uac_restart();
    fn sa_set_priority_low();
    fn sa_set_auto_threshold(percent: u32);
    fn sa_stop_auto_clean();
    fn sa_estimate_clean(mask: u32) -> u64;
    fn sa_estimate_each(out13: *mut u64);
    fn sa_get_standby_size(size: *mut u64);
    fn sa_get_detailed_mem(total: *mut u64, avail: *mut u64, standby: *mut u64, modified: *mut u64, cached: *mut u64);
    fn sa_get_process_count() -> i32;
    fn sa_set_aggressive(enabled: i32);
    fn sa_get_last_cleaned_mb() -> u32;
}

fn w(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

fn msgbox(title: &str, text: &str, flags: UINT) {
    unsafe {
        MessageBoxW(0, w(text).as_ptr(), w(title).as_ptr(), flags | MB_OK);
    }
}

fn msgbox_yn(title: &str, text: &str, flags: UINT) -> i32 {
    unsafe {
        MessageBoxW(0, w(text).as_ptr(), w(title).as_ptr(), flags | MB_YESNO)
    }
}

fn set_text(hwnd: HWND, id: i32, text: &str) {
    unsafe {
        let ctrl = GetDlgItem(hwnd, id);
        if ctrl != 0 {
            SetWindowTextW(ctrl, w(text).as_ptr());
        }
    }
}

fn is_checked(hwnd: HWND, id: i32) -> bool {
    unsafe {
        let ctrl = GetDlgItem(hwnd, id);
        if ctrl == 0 { return false; }
        SendMessageW(ctrl, BM_GETCHECK, 0, 0) == BST_CHECKED as isize
    }
}

fn set_check(hwnd: HWND, id: i32, checked: bool) {
    unsafe {
        let ctrl = GetDlgItem(hwnd, id);
        if ctrl != 0 {
            SendMessageW(ctrl, BM_SETCHECK, if checked { BST_CHECKED } else { BST_UNCHECKED }, 0);
        }
    }
}

fn get_mask(hwnd: HWND) -> u32 {
    let mut mask = 0u32;
    if is_checked(hwnd, IDC_CHK_WS) { mask |= 0x01; }
    if is_checked(hwnd, IDC_CHK_CACHE) { mask |= 0x02; }
    if is_checked(hwnd, IDC_CHK_STANDBY) { mask |= 0x04; }
    if is_checked(hwnd, IDC_CHK_MODIFIED) { mask |= 0x08; }
    if is_checked(hwnd, IDC_CHK_REGISTRY) { mask |= 0x10; }
    if is_checked(hwnd, IDC_CHK_COMBINE) { mask |= 0x20; }
    if is_checked(hwnd, IDC_CHK_LOWPRIO) { mask |= 0x40; }
    if is_checked(hwnd, IDC_CHK_FLUSH) { mask |= 0x80; }
    if is_checked(hwnd, IDC_CHK_CLIP) { mask |= 0x100; }
    if is_checked(hwnd, IDC_CHK_DNS) { mask |= 0x200; }
    if is_checked(hwnd, IDC_CHK_TEMP) { mask |= 0x400; }
    if is_checked(hwnd, IDC_CHK_PREFETCH) { mask |= 0x800; }
    if is_checked(hwnd, IDC_CHK_THUMB) { mask |= 0x1000; }
    mask
}

fn fmt_bytes(b: u64) -> String {
    if b >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", b as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if b >= 1024 * 1024 {
        format!("{:.1} MB", b as f64 / (1024.0 * 1024.0))
    } else {
        format!("{} KB", b / 1024)
    }
}

fn update_mem(hwnd: HWND) {
    unsafe {
        let mut total = 0u64;
        let mut avail = 0u64;
        let mut standby = 0u64;
        let mut modified = 0u64;
        let mut cached = 0u64;
        sa_get_detailed_mem(&mut total, &mut avail, &mut standby, &mut modified, &mut cached);
        let used = total.saturating_sub(avail);
        let pct = if total > 0 { (used * 100) / total } else { 0 };

        let info = format!(
            "RAM  {} / {}   ({}% used)   Free: {}",
            fmt_bytes(used), fmt_bytes(total), pct, fmt_bytes(avail)
        );
        set_text(hwnd, IDC_MEM_INFO, &info);

        let st = format!(
            "Standby: {}   |   Modified: {}   |   Cache: {}",
            fmt_bytes(standby), fmt_bytes(modified), fmt_bytes(cached)
        );
        set_text(hwnd, IDC_STANDBY, &st);

        let pc = sa_get_process_count();
        set_text(hwnd, IDC_PROC_COUNT, &format!("Processes: {}", pc));
    }
}

fn update_estimate(hwnd: HWND) {
    let mask = get_mask(hwnd);
    unsafe {
        let mut each = [0u64; 13];
        sa_estimate_each(each.as_mut_ptr());
        let labels = [
            (0x01, 0, "WS"),
            (0x02, 1, "Cache"),
            (0x04, 2, "Standby"),
            (0x08, 3, "Modified"),
            (0x10, 4, "Reg"),
            (0x20, 5, "Combine"),
            (0x40, 6, "LowPrio"),
            (0x80, 7, "Flush"),
            (0x100, 8, "Clip"),
            (0x200, 9, "DNS"),
            (0x400, 10, "Temp"),
            (0x800, 11, "Prefetch"),
            (0x1000, 12, "Thumb"),
        ];
        let mut parts: Vec<String> = Vec::new();
        let mut total = 0u64;
        for (bit, idx, name) in labels {
            if mask & bit != 0 {
                let v = each[idx];
                total += v;
                if v > 0 {
                    parts.push(format!("{} {}", name, fmt_bytes(v)));
                }
            }
        }
        let detail = if parts.is_empty() {
            "none selected".to_string()
        } else {
            parts.join(" | ")
        };
        set_text(hwnd, IDC_ESTIMATE, &format!("Total ~{}   [{}]", fmt_bytes(total), detail));
    }
}


fn help_for(id: i32) -> &'static str {
    match id {
        IDC_CHK_WS => "Working Set: Forces processes to release non-essential pages from their working set. Frees RAM held by idle apps without closing them. Safe and fast.",
        IDC_CHK_CACHE => "System Cache: Shrinks the Windows file cache (standby mapped files). Releases cached file data so free RAM increases. Very effective on systems with large file cache.",
        IDC_CHK_STANDBY => "Standby List: Purges the standby page list (cached pages ready for reuse). Often frees the largest amount of RAM. Core of memory cleaners.",
        IDC_CHK_MODIFIED => "Modified List: Flushes modified pages to disk then frees them. Reclaims dirty pages waiting for write. Slight disk activity possible.",
        IDC_CHK_REGISTRY => "Registry: Requests registry hive memory reduction. Trims paged registry data held in RAM. Low impact, small reclaim.",
        IDC_CHK_COMBINE => "Combine Pages: Asks the kernel to combine identical memory pages (page combining). Saves RAM when many processes share same content.",
        IDC_CHK_LOWPRIO => "Low Priority: Purges low-priority standby pages first. Gentler than full standby purge; good for background cleanup.",
        IDC_CHK_FLUSH => "Flush Volumes: Calls FlushFileBuffers on fixed drives. Ensures pending writes hit disk; pairs well with Modified List.",
        IDC_CHK_CLIP => "Clipboard: Clears the Windows clipboard contents from memory. Instant, tiny reclaim, useful after large copy operations.",
        IDC_CHK_DNS => "DNS Cache: Runs ipconfig /flushdns. Clears resolved hostnames. Small RAM win; also forces fresh DNS lookups.",
        IDC_CHK_TEMP => "Temp Files: Deletes files under user Temp, LocalAppData\\Temp, and Windows\\Temp. Can free hundreds of MB on disk and related cache.",
        IDC_CHK_PREFETCH => "Prefetch: Clears Windows Prefetch folder. Frees small RAM/disk related to app launch traces. Harmless; Windows rebuilds as needed.",
        IDC_CHK_THUMB => "Thumbnail Cache: Deletes Explorer thumbcache_*.db files. Reclaims space from image/video thumbnail database.",
        IDC_BTN_CLEAN => "CLEAN NOW: Runs all selected cleaners in parallel threads, measures free RAM before/after, reports MB recovered.",
        IDC_BTN_SELECT_SAFE => "Safe Profile: Selects only low-risk high-value options (Working Set, Cache, Standby, Modified, Clipboard, DNS).",
        IDC_BTN_SELECT_ALL => "Select All: Enables every cleaner including Temp/Prefetch/Thumbnails. Stronger reclaim, more disk work.",
        IDC_BTN_AGGRESSIVE => "Aggressive: Faster Nt calls, second standby pass, shorter timeouts. More RAM reclaimed, slightly higher CPU spike.",
        IDC_BTN_AUTO => "Start Auto: Background thread checks RAM usage. When used% >= threshold, runs Safe clean automatically.",
        IDC_BTN_STOP_AUTO => "Stop Auto: Stops the background auto-clean thread immediately.",
        IDC_BTN_THEME => "Theme: Toggle true black Dark mode and Light mode. Affects background and text colors.",
        IDC_BTN_BG => "BG Shade: Cycles dark background shades (pure black, soft black, blue-black, red-black). Dark mode only.",
        IDC_THRESHOLD_EDIT => "Threshold %: Auto-clean triggers when used RAM reaches this percent (1-99). Default 75.",
        _ => "SA Cleaner — select options, then CLEAN NOW. Hover controls for details.",
    }
}

fn update_status(hwnd: HWND, text: &str) {
    set_text(hwnd, IDC_STATUS, text);
}

fn create_font(size: i32, bold: bool, name: &str) -> HFONT {
    unsafe {
        CreateFontW(
            size, 0, 0, 0,
            if bold { FW_BOLD } else { FW_NORMAL },
            0, 0, 0, DEFAULT_CHARSET,
            OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY,
            DEFAULT_PITCH | FF_DONTCARE, w(name).as_ptr(),
        )
    }
}

fn create_ctrl(
    parent: HWND, class: &str, text: &str, style: DWORD,
    x: i32, y: i32, w: i32, h: i32, id: i32, hinst: HINSTANCE,
) -> HWND {
    unsafe {
        CreateWindowExW(
            0, OsStr::new(class).encode_wide().chain(Some(0)).collect::<Vec<_>>().as_ptr(),
            OsStr::new(text).encode_wide().chain(Some(0)).collect::<Vec<_>>().as_ptr(),
            WS_CHILD | WS_VISIBLE | style,
            x, y, w, h, parent, id as HMENU, hinst, ptr::null_mut(),
        )
    }
}

static mut G_FONT_TITLE: HFONT = 0;
static mut G_FONT_NORMAL: HFONT = 0;
static mut G_FONT_SMALL: HFONT = 0;
static mut G_BRUSH_BG: HBRUSH = 0;
static mut G_BRUSH_PANEL: HBRUSH = 0;
static mut G_AUTO_ON: bool = false;
static mut G_AGGRESSIVE: bool = false;
static mut G_DARK: bool = true;
static mut G_BG_COLOR: COLORREF = CLR_DARK_BG;
static mut G_TEXT_COLOR: COLORREF = CLR_DARK_TEXT;
static mut G_TRAY: bool = false;
static mut G_HWND: HWND = 0;
static mut G_CLEANING: bool = false;

unsafe fn apply_theme(hwnd: HWND) {
    if G_BRUSH_BG != 0 { DeleteObject(G_BRUSH_BG); }
    if G_BRUSH_PANEL != 0 { DeleteObject(G_BRUSH_PANEL); }
    if G_DARK {
        G_BG_COLOR = CLR_DARK_BG;
        G_TEXT_COLOR = CLR_DARK_TEXT;
        G_BRUSH_BG = CreateSolidBrush(CLR_DARK_BG);
        G_BRUSH_PANEL = CreateSolidBrush(CLR_DARK_PANEL);
        set_text(hwnd, IDC_BTN_THEME, "Theme: Dark");
    } else {
        G_BG_COLOR = CLR_LIGHT_BG;
        G_TEXT_COLOR = CLR_LIGHT_TEXT;
        G_BRUSH_BG = CreateSolidBrush(CLR_LIGHT_BG);
        G_BRUSH_PANEL = CreateSolidBrush(CLR_LIGHT_PANEL);
        set_text(hwnd, IDC_BTN_THEME, "Theme: Light");
    }
    InvalidateRect(hwnd, ptr::null(), 1);
}


unsafe fn tray_add(hwnd: HWND) {
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cb_size = std::mem::size_of::<NOTIFYICONDATAW>() as DWORD;
    nid.h_wnd = hwnd;
    nid.u_id = 1;
    nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.u_callback_message = WM_TRAYICON;
    nid.h_icon = LoadIconW(0, IDI_APPLICATION);
    let tip = w("SA Cleaner — running in background");
    for (i, ch) in tip.iter().take(127).enumerate() {
        nid.sz_tip[i] = *ch;
    }
    Shell_NotifyIconW(NIM_ADD, &mut nid);
    G_TRAY = true;
}

unsafe fn tray_remove(hwnd: HWND) {
    if !G_TRAY { return; }
    let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
    nid.cb_size = std::mem::size_of::<NOTIFYICONDATAW>() as DWORD;
    nid.h_wnd = hwnd;
    nid.u_id = 1;
    Shell_NotifyIconW(NIM_DELETE, &mut nid);
    G_TRAY = false;
}

unsafe fn hide_to_tray(hwnd: HWND) {
    ShowWindow(hwnd, SW_HIDE);
    if !G_TRAY { tray_add(hwnd); }
}

unsafe fn restore_from_tray(hwnd: HWND) {
    ShowWindow(hwnd, SW_RESTORE);
    SetFocus(hwnd);
}

unsafe fn cycle_bg(hwnd: HWND) {
    if !G_DARK { return; }
    const PRESETS: [COLORREF; 5] = [
        0x00000000,
        0x000A0A0A,
        0x00101010,
        0x00080812,
        0x00120000,
    ];
    static mut IDX: usize = 0;
    IDX = (IDX + 1) % PRESETS.len();
    G_BG_COLOR = PRESETS[IDX];
    if G_BRUSH_BG != 0 { DeleteObject(G_BRUSH_BG); }
    G_BRUSH_BG = CreateSolidBrush(G_BG_COLOR);
    InvalidateRect(hwnd, ptr::null(), 1);
}

unsafe extern "system" fn wndproc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            let hinst = GetModuleHandleW(ptr::null());
            G_FONT_TITLE = create_font(26, true, "Segoe UI");
            G_FONT_NORMAL = create_font(15, false, "Segoe UI");
            G_FONT_SMALL = create_font(13, false, "Segoe UI");
            G_DARK = true;
            G_BG_COLOR = CLR_DARK_BG;
            G_TEXT_COLOR = CLR_DARK_TEXT;
            G_BRUSH_BG = CreateSolidBrush(CLR_DARK_BG);
            G_BRUSH_PANEL = CreateSolidBrush(CLR_DARK_PANEL);

            let t = create_ctrl(hwnd, "STATIC", "SA Cleaner  v2.0", SS_CENTER, 20, 10, 700, 32, IDC_TITLE, hinst);
            SendMessageW(t, 0x0030, G_FONT_TITLE as WPARAM, 1);

            let m = create_ctrl(hwnd, "STATIC", "Loading memory...", SS_LEFT, 30, 50, 680, 20, IDC_MEM_INFO, hinst);
            SendMessageW(m, 0x0030, G_FONT_NORMAL as WPARAM, 1);

            let s = create_ctrl(hwnd, "STATIC", "", SS_LEFT, 30, 72, 680, 18, IDC_STANDBY, hinst);
            SendMessageW(s, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let p = create_ctrl(hwnd, "STATIC", "Processes: -", SS_LEFT, 30, 94, 200, 16, IDC_PROC_COUNT, hinst);
            SendMessageW(p, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let lf = create_ctrl(hwnd, "STATIC", "Last freed: -", SS_LEFT, 250, 94, 250, 16, IDC_LAST_FREED, hinst);
            SendMessageW(lf, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let checks = [
                (IDC_CHK_WS, "Working Set", 28, 128),
                (IDC_CHK_CACHE, "System Cache", 200, 128),
                (IDC_CHK_STANDBY, "Standby List", 380, 128),
                (IDC_CHK_MODIFIED, "Modified List", 555, 128),
                (IDC_CHK_REGISTRY, "Registry", 28, 160),
                (IDC_CHK_COMBINE, "Combine Pages", 200, 160),
                (IDC_CHK_LOWPRIO, "Low Priority", 380, 160),
                (IDC_CHK_FLUSH, "Flush Volumes", 555, 160),
                (IDC_CHK_CLIP, "Clipboard", 28, 192),
                (IDC_CHK_DNS, "DNS Cache", 200, 192),
                (IDC_CHK_TEMP, "Temp Files", 380, 192),
                (IDC_CHK_PREFETCH, "Prefetch", 555, 192),
                (IDC_CHK_THUMB, "Thumbnail Cache", 28, 224),
            ];
            for (id, text, x, y) in checks {
                let c = create_ctrl(hwnd, "BUTTON", text, BS_AUTOCHECKBOX | WS_TABSTOP, x, y, 170, 24, id, hinst);
                SendMessageW(c, 0x0030, G_FONT_SMALL as WPARAM, 1);
                if matches!(id, IDC_CHK_WS | IDC_CHK_CACHE | IDC_CHK_STANDBY | IDC_CHK_MODIFIED | IDC_CHK_CLIP | IDC_CHK_DNS) {
                    set_check(hwnd, id, true);
                }
            }

            let btn_clean = create_ctrl(hwnd, "BUTTON", "CLEAN NOW", BS_DEFPUSHBUTTON | WS_TABSTOP, 28, 270, 140, 34, IDC_BTN_CLEAN, hinst);
            SendMessageW(btn_clean, 0x0030, G_FONT_NORMAL as WPARAM, 1);

            let btn_safe = create_ctrl(hwnd, "BUTTON", "Safe Profile", BS_PUSHBUTTON | WS_TABSTOP, 180, 270, 105, 34, IDC_BTN_SELECT_SAFE, hinst);
            SendMessageW(btn_safe, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let btn_all = create_ctrl(hwnd, "BUTTON", "Select All", BS_PUSHBUTTON | WS_TABSTOP, 295, 270, 95, 34, IDC_BTN_SELECT_ALL, hinst);
            SendMessageW(btn_all, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let btn_agg = create_ctrl(hwnd, "BUTTON", "Aggressive: OFF", BS_PUSHBUTTON | WS_TABSTOP, 400, 270, 125, 34, IDC_BTN_AGGRESSIVE, hinst);
            SendMessageW(btn_agg, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let btn_theme = create_ctrl(hwnd, "BUTTON", "Theme: Dark", BS_PUSHBUTTON | WS_TABSTOP, 535, 270, 100, 34, IDC_BTN_THEME, hinst);
            SendMessageW(btn_theme, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let btn_tray = create_ctrl(hwnd, "BUTTON", "To Tray", BS_PUSHBUTTON | WS_TABSTOP, 645, 270, 80, 34, IDC_BTN_TRAY, hinst);
            SendMessageW(btn_tray, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let al = create_ctrl(hwnd, "STATIC", "Auto-Clean when RAM usage exceeds threshold", SS_LEFT, 28, 320, 420, 18, IDC_AUTO_LABEL, hinst);
            SendMessageW(al, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let btn_auto = create_ctrl(hwnd, "BUTTON", "Start Auto", BS_PUSHBUTTON | WS_TABSTOP, 28, 348, 120, 30, IDC_BTN_AUTO, hinst);
            SendMessageW(btn_auto, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let btn_stop = create_ctrl(hwnd, "BUTTON", "Stop Auto", BS_PUSHBUTTON | WS_TABSTOP, 160, 348, 100, 30, IDC_BTN_STOP_AUTO, hinst);
            SendMessageW(btn_stop, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let btn_bg = create_ctrl(hwnd, "BUTTON", "BG Shade", BS_PUSHBUTTON | WS_TABSTOP, 270, 348, 90, 30, IDC_BTN_BG, hinst);
            SendMessageW(btn_bg, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let est = create_ctrl(hwnd, "STATIC", "Estimated reclaim: -", SS_LEFT, 28, 395, 700, 36, IDC_ESTIMATE, hinst);
            SendMessageW(est, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let st = create_ctrl(hwnd, "STATIC", "Ready", SS_LEFT, 28, 435, 700, 18, IDC_STATUS, hinst);
            SendMessageW(st, 0x0030, G_FONT_NORMAL as WPARAM, 1);

            let hl = create_ctrl(hwnd, "STATIC", "Click any option for a detailed description.", SS_LEFT, 28, 458, 700, 55, IDC_HELP, hinst);
            SendMessageW(hl, 0x0030, G_FONT_SMALL as WPARAM, 1);

            let tl = create_ctrl(hwnd, "STATIC", "Threshold %:", SS_LEFT, 380, 352, 90, 18, IDC_THRESHOLD_LABEL, hinst);
            SendMessageW(tl, 0x0030, G_FONT_SMALL as WPARAM, 1);
            let te = create_ctrl(hwnd, "EDIT", "75", WS_BORDER | ES_NUMBER | ES_AUTOHSCROLL | WS_TABSTOP, 475, 348, 48, 26, IDC_THRESHOLD_EDIT, hinst);
            SendMessageW(te, 0x0030, G_FONT_SMALL as WPARAM, 1);

            G_HWND = hwnd;
            SetTimer(hwnd, 1, 2000, 0);
            update_mem(hwnd);
            update_estimate(hwnd);
            0
        }
        WM_ERASEBKGND => {
            let mut rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            GetClientRect(hwnd, &mut rc);
            FillRect(wparam as HDC, &rc, G_BRUSH_BG);
            1
        }
        WM_CTLCOLORSTATIC | WM_CTLCOLORBTN => {
            let hdc = wparam as HDC;
            SetTextColor(hdc, G_TEXT_COLOR);
            SetBkMode(hdc, TRANSPARENT);
            G_BRUSH_BG
        }
        WM_APP_CLEAN_DONE => {
            G_CLEANING = false;
            let res = wparam as i32;
            let mut buf = [0i8; 512];
            sa_get_err(buf.as_mut_ptr(), 512);
            let txt = std::ffi::CStr::from_ptr(buf.as_ptr()).to_string_lossy().into_owned();
            let freed = sa_get_last_cleaned_mb();
            set_text(hwnd, IDC_LAST_FREED, &format!("Last freed: {} MB", freed));
            if res == -1 {
                msgbox("Admin Required", &txt, MB_ICONERROR);
            } else if res == 0 {
                msgbox("Success", &format!("{} | Freed approximately {} MB", txt, freed), MB_ICONINFORMATION);
            } else {
                msgbox("Partial", &txt, MB_ICONWARNING);
            }
            update_status(hwnd, "Ready");
            update_mem(hwnd);
            update_estimate(hwnd);
            0
        }
        WM_TRAYICON => {
            // left double-click restore
            let mouse = (wparam >> 16) as u32; // not used; lparam has msg
            let msg_tray = lparam as u32;
            if msg_tray == 0x0203 || msg_tray == 0x0201 { // WM_LBUTTONDBLCLK / DOWN
                restore_from_tray(hwnd);
            }
            0
        }
        WM_SYSCOMMAND => {
            let cmd = wparam & 0xFFF0;
            if cmd == SC_MINIMIZE {
                hide_to_tray(hwnd);
                return 0;
            }
            // block maximize / fullscreen-style
            if cmd == 0xF030 || cmd == 0xF032 { // SC_MAXIMIZE / SC_MAXIMIZE2
                return 0;
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as i32;
            match id {
                IDC_BTN_CLEAN => {
                    if G_CLEANING {
                        update_status(hwnd, "Already cleaning...");
                        return 0;
                    }
                    let mask = get_mask(hwnd);
                    if mask == 0 {
                        msgbox("Nothing Selected", "Select at least one cleaning option.", MB_ICONWARNING);
                        update_status(hwnd, "Ready");
                        return 0;
                    }
                    G_CLEANING = true;
                    update_status(hwnd, "Cleaning in background...");
                    let hwnd_copy = hwnd;
                    std::thread::spawn(move || {
                        let res = unsafe { sa_clean(mask) };
                        unsafe { PostMessageW(hwnd_copy, WM_APP_CLEAN_DONE, res as WPARAM, 0); }
                    });
                }
                IDC_BTN_SELECT_ALL => {
                    for id in [
                        IDC_CHK_WS, IDC_CHK_CACHE, IDC_CHK_STANDBY, IDC_CHK_MODIFIED,
                        IDC_CHK_REGISTRY, IDC_CHK_COMBINE, IDC_CHK_LOWPRIO, IDC_CHK_FLUSH,
                        IDC_CHK_CLIP, IDC_CHK_DNS, IDC_CHK_TEMP, IDC_CHK_PREFETCH, IDC_CHK_THUMB,
                    ] {
                        set_check(hwnd, id, true);
                    }
                    update_estimate(hwnd);
                }
                IDC_BTN_SELECT_SAFE => {
                    for id in [
                        IDC_CHK_WS, IDC_CHK_CACHE, IDC_CHK_STANDBY, IDC_CHK_MODIFIED,
                        IDC_CHK_REGISTRY, IDC_CHK_COMBINE, IDC_CHK_LOWPRIO, IDC_CHK_FLUSH,
                        IDC_CHK_CLIP, IDC_CHK_DNS, IDC_CHK_TEMP, IDC_CHK_PREFETCH, IDC_CHK_THUMB,
                    ] {
                        set_check(hwnd, id, false);
                    }
                    set_check(hwnd, IDC_CHK_WS, true);
                    set_check(hwnd, IDC_CHK_CACHE, true);
                    set_check(hwnd, IDC_CHK_STANDBY, true);
                    set_check(hwnd, IDC_CHK_MODIFIED, true);
                    set_check(hwnd, IDC_CHK_CLIP, true);
                    set_check(hwnd, IDC_CHK_DNS, true);
                    update_estimate(hwnd);
                }
                IDC_BTN_AGGRESSIVE => {
                    G_AGGRESSIVE = !G_AGGRESSIVE;
                    sa_set_aggressive(if G_AGGRESSIVE { 1 } else { 0 });
                    let txt = if G_AGGRESSIVE { "Aggressive: ON" } else { "Aggressive: OFF" };
                    set_text(hwnd, IDC_BTN_AGGRESSIVE, txt);
                    update_status(hwnd, if G_AGGRESSIVE { "Aggressive mode enabled" } else { "Normal mode" });
                }
                IDC_BTN_THEME => {
                    G_DARK = !G_DARK;
                    apply_theme(hwnd);
                    update_status(hwnd, if G_DARK { "Dark theme" } else { "Light theme" });
                }
                IDC_BTN_BG => {
                    cycle_bg(hwnd);
                    update_status(hwnd, "Background shade changed");
                }
                IDC_BTN_TRAY => {
                    hide_to_tray(hwnd);
                    update_status(hwnd, "Running in system tray (background)");
                }
                IDC_BTN_AUTO => {
                    if !G_AUTO_ON {
                        let mut pct: u32 = 75;
                        let edit = GetDlgItem(hwnd, IDC_THRESHOLD_EDIT);
                        if edit != 0 {
                            let mut buf = [0u16; 8];
                            let n = GetWindowTextW(edit, buf.as_mut_ptr(), 8);
                            if n > 0 {
                                let s = String::from_utf16_lossy(&buf[..n as usize]);
                                if let Ok(v) = s.trim().parse::<u32>() {
                                    pct = if v < 1 { 1 } else if v > 99 { 99 } else { v };
                                }
                            }
                        }
                        sa_set_auto_threshold(pct);
                        G_AUTO_ON = true;
                        set_text(hwnd, IDC_BTN_AUTO, &format!("Auto Running ({}%)", pct));
                        update_status(hwnd, &format!("Auto-clean started — triggers at {}% RAM usage", pct));
                        set_text(hwnd, IDC_HELP, help_for(IDC_BTN_AUTO));
                    }
                }
                IDC_BTN_STOP_AUTO => {
                    sa_stop_auto_clean();
                    G_AUTO_ON = false;
                    set_text(hwnd, IDC_BTN_AUTO, "Start Auto (75%)");
                    update_status(hwnd, "Auto-clean stopped");
                }
                IDC_CHK_WS | IDC_CHK_CACHE | IDC_CHK_STANDBY | IDC_CHK_MODIFIED |
                IDC_CHK_REGISTRY | IDC_CHK_COMBINE | IDC_CHK_LOWPRIO | IDC_CHK_FLUSH |
                IDC_CHK_CLIP | IDC_CHK_DNS | IDC_CHK_TEMP | IDC_CHK_PREFETCH | IDC_CHK_THUMB => {
                    update_estimate(hwnd);
                    set_text(hwnd, IDC_HELP, help_for(id));
                }
                IDC_BTN_CLEAN | IDC_BTN_SELECT_SAFE | IDC_BTN_SELECT_ALL | IDC_BTN_AGGRESSIVE |
                IDC_BTN_AUTO | IDC_BTN_STOP_AUTO | IDC_BTN_THEME | IDC_BTN_BG => {
                    set_text(hwnd, IDC_HELP, help_for(id));
                }
                _ => {}
            }
            0
        }
        WM_TIMER => {
            update_mem(hwnd);
            0
        }
        WM_SIZE => {
            InvalidateRect(hwnd, ptr::null(), 1);
            0
        }
        WM_DESTROY => {
            KillTimer(hwnd, 1);
            sa_stop_auto_clean();
            tray_remove(hwnd);
            if G_FONT_TITLE != 0 { DeleteObject(G_FONT_TITLE); }
            if G_FONT_NORMAL != 0 { DeleteObject(G_FONT_NORMAL); }
            if G_FONT_SMALL != 0 { DeleteObject(G_FONT_SMALL); }
            if G_BRUSH_BG != 0 { DeleteObject(G_BRUSH_BG); }
            if G_BRUSH_PANEL != 0 { DeleteObject(G_BRUSH_PANEL); }
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn ensure_admin() {
    unsafe {
        if sa_is_admin() == 0 {
            let r = msgbox_yn(
                "Administrator Required",
                "SA Cleaner v2 needs Administrator privileges for memory operations.\n\nClick Yes to restart elevated.",
                MB_ICONINFORMATION,
            );
            if r == IDYES {
                sa_uac_restart();
            }
            std::process::exit(0);
        }
    }
}

fn main() {
    unsafe {
        sa_set_priority_low();
        ensure_admin();

        let hinst = GetModuleHandleW(ptr::null());
        let cn = w("SACleanerV2Dark");
        let wc = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfn_wnd_proc: wndproc as usize,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: hinst,
            h_icon: 0,
            h_cursor: LoadCursorW(0, 32512), 
            hbr_background: CreateSolidBrush(CLR_DARK_BG),
            lpsz_menu_name: ptr::null(),
            lpsz_class_name: cn.as_ptr(),
            h_icon_sm: 0,
        };
        RegisterClassExW(&wc);

        let hwnd = CreateWindowExW(
            0,
            cn.as_ptr(),
            w("SA Cleaner v2.0  —  Advanced Memory Cleaner").as_ptr(),
            WS_FIXED_FRAME,
            0x80000000u32 as i32,
            0x80000000u32 as i32,
            760,
            640,
            0,
            0,
            hinst,
            ptr::null_mut(),
        );
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
