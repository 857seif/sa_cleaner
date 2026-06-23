#![windows_subsystem = "windows"]
use std::ffi::{c_char, c_int, c_uint, CStr, CString};
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

#[link(name = "sacleaner_cpp", kind = "static")]
extern "C" {
    fn sa_clean(mask: c_uint) -> c_int;
    fn sa_get_err(buf: *mut c_char, len: c_uint);
    fn sa_get_mem(total: *mut u64, used: *mut u64, free: *mut u64);
    fn sa_is_admin() -> c_int;
    fn sa_uac_restart();
    fn sa_estimate_clean(mask: c_uint) -> u64;
    fn sa_check_excluded() -> c_int;
    fn sa_set_auto_threshold(percent: c_uint);
    fn sa_stop_auto_clean();
    fn sa_set_priority_low();
    fn sa_add_exclusion(name: *const c_char);
    fn sa_remove_exclusion(name: *const c_char);
    fn sa_get_exclusions(buf: *mut c_char, len: c_int);
}

extern "system" {
    fn MessageBoxA(hWnd: isize, lpText: *const u8, lpCaption: *const u8, uType: u32) -> i32;
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
    fn CreateWindowExW(dwExStyle: u32, lpClassName: *const u16, lpWindowName: *const u16,
        dwStyle: u32, X: i32, Y: i32, nWidth: i32, nHeight: i32,
        hWndParent: isize, hMenu: isize, hInstance: isize, lpParam: *mut ()) -> isize;
    fn ShowWindow(hWnd: isize, nCmdShow: i32) -> i32;
    fn UpdateWindow(hWnd: isize) -> i32;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: isize, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn PostQuitMessage(nExitCode: i32);
    fn DefWindowProcW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn PostMessageW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> i32;
    fn SendMessageW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn CheckDlgButton(hDlg: isize, nIDButton: i32, uCheck: u32);
    fn IsDlgButtonChecked(hDlg: isize, nIDButton: i32) -> u32;
    fn SetDlgItemTextW(hDlg: isize, nIDDlgItem: i32, lpString: *const u16);
    fn GetDlgItemTextW(hDlg: isize, nIDDlgItem: i32, lpString: *mut u16, nMaxCount: i32) -> i32;
    fn GetModuleHandleW(lpModuleName: *const u16) -> isize;
    fn CreateFontW(cHeight: i32, cWidth: i32, cEscapement: i32, cOrientation: i32,
        cWeight: i32, bItalic: u32, bUnderline: u32, bStrikeOut: u32,
        iCharSet: u32, iOutPrecision: u32, iClipPrecision: u32, iQuality: u32,
        iPitchAndFamily: u32, pszFaceName: *const u16) -> isize;
    fn GetDlgItem(hDlg: isize, nIDDlgItem: i32) -> isize;
    fn SetWindowPos(hWnd: isize, hWndInsertAfter: isize, X: i32, Y: i32, cx: i32, cy: i32, uFlags: u32) -> i32;
    fn SetWindowTextW(hWnd: isize, lpString: *const u16) -> i32;
    fn IsWindowVisible(hWnd: isize) -> i32;
    fn SetTimer(hWnd: isize, nIDEvent: usize, uElapse: u32, lpTimerFunc: usize) -> usize;
    fn KillTimer(hWnd: isize, uIDEvent: usize) -> i32;
    fn GetClientRect(hWnd: isize, lpRect: *mut RECT) -> i32;
    fn InvalidateRect(hWnd: isize, lpRect: *const RECT, bErase: i32) -> i32;
    fn SetBkColor(hdc: isize, color: u32) -> u32;
    fn SetTextColor(hdc: isize, color: u32) -> u32;
    fn CreateSolidBrush(color: u32) -> isize;
    fn DeleteObject(hObject: isize) -> i32;
    fn FillRect(hDC: isize, lprc: *const RECT, hbr: isize) -> i32;
    fn GetDC(hWnd: isize) -> isize;
    fn ReleaseDC(hWnd: isize, hDC: isize) -> i32;
    fn SetWindowLongW(hWnd: isize, nIndex: i32, dwNewLong: i32) -> i32;
    fn GetWindowLongW(hWnd: isize, nIndex: i32) -> i32;
    fn GetWindow(hWnd: isize, uCmd: u32) -> isize;
}

const WM_CREATE: u32 = 0x0001;
const WM_COMMAND: u32 = 0x0111;
const WM_DESTROY: u32 = 0x0002;
const WM_TIMER: u32 = 0x0113;
const WM_SIZE: u32 = 0x0005;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_CTLCOLORSTATIC: u32 = 0x0138;
const WM_CTLCOLORBTN: u32 = 0x0135;
const WM_CTLCOLOREDIT: u32 = 0x0133;
const WM_CTLCOLORLISTBOX: u32 = 0x0134;
const WM_USER: u32 = 0x0400;
const WM_CLEAN_DONE: u32 = WM_USER + 10;

const BS_AUTOCHECKBOX: u32 = 0x00000003;
const BS_PUSHBUTTON: u32 = 0x00000000;
const BS_GROUPBOX: u32 = 0x00000007;
const WS_CHILD: u32 = 0x40000000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_TABSTOP: u32 = 0x00010000;
const WS_BORDER: u32 = 0x00800000;
const WS_VSCROLL: u32 = 0x00200000;
const ES_NUMBER: u32 = 0x2000;
const ES_AUTOHSCROLL: u32 = 0x0080;
const LBS_STANDARD: u32 = 0xA00003;
const LBS_NOTIFY: u32 = 0x0001;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const WS_THICKFRAME: u32 = 0x00040000;
const WS_MAXIMIZEBOX: u32 = 0x00010000;
const WS_MINIMIZEBOX: u32 = 0x00020000;
const WS_CAPTION: u32 = 0x00C00000;
const WS_SYSMENU: u32 = 0x00080000;
const SW_SHOW: i32 = 5;
const SW_HIDE: i32 = 0;
const MB_OK: u32 = 0x00000000;
const MB_OKCANCEL: u32 = 0x00000001;
const MB_ICONINFORMATION: u32 = 0x00000040;
const MB_ICONWARNING: u32 = 0x00000030;
const MB_ICONERROR: u32 = 0x00000010;
const IDOK: i32 = 1;
const IDCANCEL: i32 = 2;
const BST_CHECKED: u32 = 1;
const BST_UNCHECKED: u32 = 0;
const COLOR_WINDOW: i32 = 5;
const FW_BOLD: i32 = 700;
const FW_NORMAL: i32 = 400;
const GWL_STYLE: i32 = -16;

const PBM_SETRANGE: u32 = WM_USER + 1;
const PBM_SETPOS: u32 = WM_USER + 2;
const PBM_SETSTEP: u32 = WM_USER + 4;
const PBS_SMOOTH: u32 = 0x01;

// Light colors
const CLR_L_BG: u32 = 0xFFFFFF;
const CLR_L_TEXT: u32 = 0x000000;
const CLR_L_PANEL: u32 = 0xF0F0F0;
const CLR_L_BAR: u32 = 0x00CC00;

// Dark colors
const CLR_D_BG: u32 = 0x1E1E2E;
const CLR_D_TEXT: u32 = 0xCDD6F4;
const CLR_D_PANEL: u32 = 0x2D2D44;
const CLR_D_BAR: u32 = 0x89B4FA;
const CLR_D_GROUP: u32 = 0x313244;

const IDC_CHK_WS: i32 = 101;
const IDC_CHK_SC: i32 = 102;
const IDC_CHK_P0: i32 = 103;
const IDC_CHK_SB: i32 = 104;
const IDC_CHK_ML: i32 = 105;
const IDC_CHK_CB: i32 = 106;
const IDC_CHK_RG: i32 = 107;
const IDC_CHK_FV: i32 = 108;
const IDC_BTN_CLEAN: i32 = 201;
const IDC_BTN_CLEAN_SAFE: i32 = 202;
const IDC_BTN_SEL_ALL: i32 = 203;
const IDC_BTN_SEL_NONE: i32 = 204;
const IDC_BTN_ABOUT: i32 = 205;
const IDC_STATIC_MEM: i32 = 301;
const IDC_STATIC_EST: i32 = 302;
const IDC_RAM_BAR: i32 = 401;
const IDC_GRP_SAFE: i32 = 501;
const IDC_GRP_SYS: i32 = 502;
const IDC_GRP_AUTO: i32 = 503;
const IDC_GRP_EXCL: i32 = 504;
const IDC_CHK_AUTO: i32 = 601;
const IDC_EDIT_THRESHOLD: i32 = 602;
const IDC_STATIC_THRESHOLD: i32 = 603;
const IDC_LST_EXCL: i32 = 701;
const IDC_EDIT_EXCL: i32 = 702;
const IDC_BTN_ADD_EXCL: i32 = 703;
const IDC_BTN_REM_EXCL: i32 = 704;
const IDC_STATIC_STATUS: i32 = 801;
const IDC_CHK_DARK: i32 = 901;

#[repr(C)]
struct RECT { left: i32, top: i32, right: i32, bottom: i32 }
#[repr(C)]
struct WNDCLASSEXW { cbSize: u32, style: u32, lpfnWndProc: usize, cbClsExtra: i32,
    cbWndExtra: i32, hInstance: isize, hIcon: isize, hCursor: isize,
    hbrBackground: isize, lpszMenuName: *const u16, lpszClassName: *const u16, hIconSm: isize }
#[repr(C)]
struct MSG { hwnd: isize, message: u32, wParam: usize, lParam: isize, time: u32, pt_x: i32, pt_y: i32 }
#[repr(C)]
struct CREATESTRUCTW { lpCreateParams: *mut (), hInstance: isize, hMenu: isize,
    hwndParent: isize, cy: i32, cx: i32, y: i32, x: i32, style: i32,
    lpszName: *const u16, lpszClass: *const u16, dwExStyle: u32 }

static CLEANING: AtomicBool = AtomicBool::new(false);
static mut DARK_MODE: bool = false;
static mut G_HWND: isize = 0;

fn w(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }

fn msgbox(title: &str, text: &str, icon: u32) {
    let t = CString::new(title).unwrap();
    let m = CString::new(text).unwrap();
    unsafe { MessageBoxA(0, m.as_ptr() as *const u8, t.as_ptr() as *const u8, MB_OK | icon); }
}

fn msgbox_yn(title: &str, text: &str, icon: u32) -> i32 {
    let t = CString::new(title).unwrap();
    let m = CString::new(text).unwrap();
    unsafe { MessageBoxA(0, m.as_ptr() as *const u8, t.as_ptr() as *const u8, MB_OKCANCEL | icon) }
}

fn get_err() -> String {
    let mut buf = vec![0u8; 4096];
    unsafe { sa_get_err(buf.as_mut_ptr() as *mut c_char, 4096); }
    unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char).to_string_lossy().into_owned() }
}

fn fmt_mem(b: u64) -> String {
    let u = ["B","KB","MB","GB","TB"];
    let mut s = b as f64; let mut i = 0;
    while s >= 1024.0 && i < u.len()-1 { s /= 1024.0; i += 1; }
    format!("{:.1} {}", s, u[i])
}

fn is_dark() -> bool {
    unsafe { DARK_MODE }
}

fn bg_color() -> u32 { if is_dark() { CLR_D_BG } else { CLR_L_BG } }
fn text_color() -> u32 { if is_dark() { CLR_D_TEXT } else { CLR_L_TEXT } }
fn panel_color() -> u32 { if is_dark() { CLR_D_PANEL } else { CLR_L_PANEL } }
fn bar_color() -> u32 { if is_dark() { CLR_D_BAR } else { CLR_L_BAR } }

fn update_mem(hwnd: isize) {
    let mut t: u64 = 0; let mut u: u64 = 0; let mut f: u64 = 0;
    unsafe { sa_get_mem(&mut t, &mut u, &mut f); }
    let pct = if t > 0 { (u * 100 / t) as u32 } else { 0 };
    let info = format!("RAM: {} used / {} total ({} free) | {}% used", fmt_mem(u), fmt_mem(t), fmt_mem(f), pct);
    unsafe { SetDlgItemTextW(hwnd, IDC_STATIC_MEM, w(&info).as_ptr()); }
    unsafe {
        SendMessageW(GetDlgItem(hwnd, IDC_RAM_BAR), PBM_SETPOS, pct as usize, 0);
    }
}

fn update_estimate(hwnd: isize) {
    let m = get_mask(hwnd);
    let est = unsafe { sa_estimate_clean(m) };
    let txt = format!("Estimated to clean: ~{}", fmt_mem(est));
    unsafe { SetDlgItemTextW(hwnd, IDC_STATIC_EST, w(&txt).as_ptr()); }
}

fn update_status(hwnd: isize, text: &str) {
    unsafe { SetDlgItemTextW(hwnd, IDC_STATIC_STATUS, w(text).as_ptr()); }
}

fn get_mask(hwnd: isize) -> u32 {
    let mut m: u32 = 0;
    unsafe {
        if IsDlgButtonChecked(hwnd, IDC_CHK_WS) == BST_CHECKED { m |= 0x01; }
        if IsDlgButtonChecked(hwnd, IDC_CHK_SC) == BST_CHECKED { m |= 0x02; }
        if IsDlgButtonChecked(hwnd, IDC_CHK_P0) == BST_CHECKED { m |= 0x04; }
        if IsDlgButtonChecked(hwnd, IDC_CHK_SB) == BST_CHECKED { m |= 0x08; }
        if IsDlgButtonChecked(hwnd, IDC_CHK_ML) == BST_CHECKED { m |= 0x10; }
        if IsDlgButtonChecked(hwnd, IDC_CHK_CB) == BST_CHECKED { m |= 0x20; }
        if IsDlgButtonChecked(hwnd, IDC_CHK_RG) == BST_CHECKED { m |= 0x40; }
        if IsDlgButtonChecked(hwnd, IDC_CHK_FV) == BST_CHECKED { m |= 0x80; }
    }
    m
}

fn set_chk(hwnd: isize, id: i32, chk: bool) {
    unsafe { CheckDlgButton(hwnd, id, if chk { BST_CHECKED } else { BST_UNCHECKED }); }
}

fn sel_all(hwnd: isize, chk: bool) {
    for id in [IDC_CHK_WS, IDC_CHK_SC, IDC_CHK_P0, IDC_CHK_SB, IDC_CHK_ML, IDC_CHK_CB, IDC_CHK_RG, IDC_CHK_FV] {
        set_chk(hwnd, id, chk);
    }
    update_estimate(hwnd);
}

fn get_threshold(hwnd: isize) -> u32 {
    let mut buf = vec![0u16; 16];
    unsafe {
        let len = GetDlgItemTextW(hwnd, IDC_EDIT_THRESHOLD, buf.as_mut_ptr(), 16);
        if len > 0 {
            let s = String::from_utf16_lossy(&buf[..len as usize]);
            if let Ok(n) = s.parse::<u32>() {
                if n >= 10 && n <= 95 { return n; }
            }
        }
    }
    60
}

fn refresh_exclusion_list(hwnd: isize) {
    let mut buf = vec![0u8; 4096];
    unsafe { sa_get_exclusions(buf.as_mut_ptr() as *mut c_char, 4096); }
    let list = unsafe { CStr::from_ptr(buf.as_ptr() as *const c_char).to_string_lossy() };
    unsafe { SendMessageW(GetDlgItem(hwnd, IDC_LST_EXCL), 0x0184, 0, 0); }
    if !list.is_empty() {
        for item in list.split(", ") {
            let trimmed = item.trim();
            if !trimmed.is_empty() {
                unsafe { SendMessageW(GetDlgItem(hwnd, IDC_LST_EXCL), 0x0180, 0, w(trimmed).as_ptr() as isize); }
            }
        }
    }
}

fn do_clean_async(hwnd: isize, mask: u32) {
    if CLEANING.swap(true, Ordering::SeqCst) {
        msgbox("Busy", "Cleaning already in progress!", MB_ICONWARNING);
        return;
    }
    update_status(hwnd, "Cleaning in progress...");
    thread::spawn(move || {
        let res = unsafe { sa_clean(mask) };
        let _txt = get_err();
        unsafe { PostMessageW(hwnd, WM_CLEAN_DONE, res as usize, 0); }
    });
}

fn apply_dark_mode(hwnd: isize, dark: bool) {
    unsafe {
        DARK_MODE = dark;
        InvalidateRect(hwnd, ptr::null(), 1);
        // Force redraw all controls
        let mut child = GetWindow(hwnd, 5); // GW_CHILD
        while child != 0 {
            InvalidateRect(child, ptr::null(), 1);
            child = GetWindow(child, 2); // GW_HWNDNEXT
        }
    }
}

fn create_ui(hwnd: isize, hinst: isize) {
    let f = unsafe { CreateFontW(13,0,0,0,FW_NORMAL,0,0,0,0,0,0,0,0, w("Segoe UI").as_ptr()) };
    let fb = unsafe { CreateFontW(14,0,0,0,FW_BOLD,0,0,0,0,0,0,0,0, w("Segoe UI").as_ptr()) };
    let ftitle = unsafe { CreateFontW(20,0,0,0,FW_BOLD,0,0,0,0,0,0,0,0, w("Segoe UI").as_ptr()) };

    // Title
    unsafe {
        let h = CreateWindowExW(0, w("STATIC").as_ptr(), w("SA Cleaner v1.0 - By Seif Afandi").as_ptr(),
            WS_CHILD|WS_VISIBLE, 15,10,500,28, hwnd, 0, hinst, ptr::null_mut());
        SendMessageW(h, 0x0030, ftitle as usize, 0);
    }

    // Dark mode toggle
    unsafe {
        let h = CreateWindowExW(0, w("BUTTON").as_ptr(), w("Dark Mode").as_ptr(),
            WS_CHILD|WS_VISIBLE|BS_AUTOCHECKBOX|WS_TABSTOP, 580,12,100,20, hwnd, IDC_CHK_DARK as isize, hinst, ptr::null_mut());
        SendMessageW(h, 0x0030, f as usize, 0);
    }

    // Memory info
    unsafe {
        let h = CreateWindowExW(0, w("STATIC").as_ptr(), w("").as_ptr(),
            WS_CHILD|WS_VISIBLE, 15,44,420,20, hwnd, IDC_STATIC_MEM as isize, hinst, ptr::null_mut());
        SendMessageW(h, 0x0030, fb as usize, 0);
    }

    // RAM Bar
    unsafe {
        CreateWindowExW(0, w("msctls_progress32").as_ptr(), w("").as_ptr(),
            WS_CHILD|WS_VISIBLE|PBS_SMOOTH, 445,44,240,20, hwnd, IDC_RAM_BAR as isize, hinst, ptr::null_mut());
        SendMessageW(GetDlgItem(hwnd, IDC_RAM_BAR), PBM_SETRANGE, 0, (0 << 16) | 100);
    }

    // Estimate
    unsafe {
        let h = CreateWindowExW(0, w("STATIC").as_ptr(), w("Estimated to clean: ~0 MB").as_ptr(),
            WS_CHILD|WS_VISIBLE, 15,68,480,18, hwnd, IDC_STATIC_EST as isize, hinst, ptr::null_mut());
        SendMessageW(h, 0x0030, f as usize, 0);
    }

    // Status
    unsafe {
        let h = CreateWindowExW(0, w("STATIC").as_ptr(), w("Ready").as_ptr(),
            WS_CHILD|WS_VISIBLE, 15,90,480,18, hwnd, IDC_STATIC_STATUS as isize, hinst, ptr::null_mut());
        SendMessageW(h, 0x0030, f as usize, 0);
    }

    // Safe Regions Group
    unsafe {
        CreateWindowExW(0, w("BUTTON").as_ptr(), w("SAFE REGIONS (Recommended)").as_ptr(),
            WS_CHILD|WS_VISIBLE|BS_GROUPBOX, 10,118,320,220, hwnd, IDC_GRP_SAFE as isize, hinst, ptr::null_mut());
    }

    let safe = [
        (IDC_CHK_WS,"Working Set","~150 MB",138,true),
        (IDC_CHK_SC,"System Cache","~400 MB",160,true),
        (IDC_CHK_P0,"Standby Priority-0","~300 MB",182,true),
        (IDC_CHK_CB,"Combine Memory","~100 MB",204,true),
        (IDC_CHK_RG,"Registry Cache","~50 MB",226,true),
        (IDC_CHK_FV,"Flush Volume Cache","~0 MB",248,true),
    ];
    for (id,txt,est,y,chk) in safe {
        let label = format!("{} ({})", txt, est);
        unsafe {
            let h = CreateWindowExW(0, w("BUTTON").as_ptr(), w(&label).as_ptr(),
                WS_CHILD|WS_VISIBLE|BS_AUTOCHECKBOX|WS_TABSTOP, 22,y,290,18, hwnd, id as isize, hinst, ptr::null_mut());
            SendMessageW(h, 0x0030, f as usize, 0);
            if chk { CheckDlgButton(hwnd, id, BST_CHECKED); }
        }
    }

    // System Regions Group (merged with safe in layout for space, or separate)
    unsafe {
        CreateWindowExW(0, w("BUTTON").as_ptr(), w("SYSTEM REGIONS (Advanced)").as_ptr(),
            WS_CHILD|WS_VISIBLE|BS_GROUPBOX, 340,118,340,100, hwnd, IDC_GRP_SYS as isize, hinst, ptr::null_mut());
    }

    let system = [
        (IDC_CHK_SB,"Standby List","~800 MB",138,false),
        (IDC_CHK_ML,"Modified Page List","~200 MB",160,false),
    ];
    for (id,txt,est,y,chk) in system {
        let label = format!("{} ({})", txt, est);
        unsafe {
            let h = CreateWindowExW(0, w("BUTTON").as_ptr(), w(&label).as_ptr(),
                WS_CHILD|WS_VISIBLE|BS_AUTOCHECKBOX|WS_TABSTOP, 352,y,320,18, hwnd, id as isize, hinst, ptr::null_mut());
            SendMessageW(h, 0x0030, f as usize, 0);
            if chk { CheckDlgButton(hwnd, id, BST_CHECKED); }
        }
    }

    // Auto-Clean Group
    unsafe {
        CreateWindowExW(0, w("BUTTON").as_ptr(), w("AUTO-CLEAN MONITOR").as_ptr(),
            WS_CHILD|WS_VISIBLE|BS_GROUPBOX, 340,228,340,110, hwnd, IDC_GRP_AUTO as isize, hinst, ptr::null_mut());
    }

    unsafe {
        let h = CreateWindowExW(0, w("BUTTON").as_ptr(), w("Enable Auto-Clean").as_ptr(),
            WS_CHILD|WS_VISIBLE|BS_AUTOCHECKBOX|WS_TABSTOP, 352,248,200,18, hwnd, IDC_CHK_AUTO as isize, hinst, ptr::null_mut());
        SendMessageW(h, 0x0030, f as usize, 0);
    }

    unsafe {
        let h = CreateWindowExW(0, w("STATIC").as_ptr(), w("Threshold %:").as_ptr(),
            WS_CHILD|WS_VISIBLE, 352,272,80,18, hwnd, IDC_STATIC_THRESHOLD as isize, hinst, ptr::null_mut());
        SendMessageW(h, 0x0030, f as usize, 0);
    }

    unsafe {
        CreateWindowExW(0, w("EDIT").as_ptr(), w("60").as_ptr(),
            WS_CHILD|WS_VISIBLE|WS_BORDER|ES_NUMBER|ES_AUTOHSCROLL|WS_TABSTOP, 440,270,50,20, hwnd, IDC_EDIT_THRESHOLD as isize, hinst, ptr::null_mut());
    }

    unsafe {
        let h = CreateWindowExW(0, w("STATIC").as_ptr(), w("(10-95%)").as_ptr(),
            WS_CHILD|WS_VISIBLE, 496,272,80,18, hwnd, 0, hinst, ptr::null_mut());
        SendMessageW(h, 0x0030, f as usize, 0);
    }

    // Exclusions Group
    unsafe {
        CreateWindowExW(0, w("BUTTON").as_ptr(), w("PROCESS EXCLUSIONS").as_ptr(),
            WS_CHILD|WS_VISIBLE|BS_GROUPBOX, 10,348,670,180, hwnd, IDC_GRP_EXCL as isize, hinst, ptr::null_mut());
    }

    unsafe {
        let h = CreateWindowExW(0, w("STATIC").as_ptr(), w("Built-in: explorer, csrss, lsass, svchost, dwm, etc.").as_ptr(),
            WS_CHILD|WS_VISIBLE, 22,368,640,16, hwnd, 0, hinst, ptr::null_mut());
        SendMessageW(h, 0x0030, f as usize, 0);
    }

    unsafe {
        CreateWindowExW(0, w("LISTBOX").as_ptr(), w("").as_ptr(),
            WS_CHILD|WS_VISIBLE|WS_BORDER|WS_VSCROLL|LBS_NOTIFY|LBS_STANDARD|WS_TABSTOP,
            22,388,280,120, hwnd, IDC_LST_EXCL as isize, hinst, ptr::null_mut());
    }

    unsafe {
        CreateWindowExW(0, w("EDIT").as_ptr(), w("").as_ptr(),
            WS_CHILD|WS_VISIBLE|WS_BORDER|ES_AUTOHSCROLL|WS_TABSTOP,
            316,388,240,22, hwnd, IDC_EDIT_EXCL as isize, hinst, ptr::null_mut());
    }

    unsafe {
        let h = CreateWindowExW(0, w("BUTTON").as_ptr(), w("Add").as_ptr(),
            WS_CHILD|WS_VISIBLE|BS_PUSHBUTTON|WS_TABSTOP, 566,388,60,24, hwnd, IDC_BTN_ADD_EXCL as isize, hinst, ptr::null_mut());
        SendMessageW(h, 0x0030, fb as usize, 0);
    }

    unsafe {
        let h = CreateWindowExW(0, w("BUTTON").as_ptr(), w("Remove").as_ptr(),
            WS_CHILD|WS_VISIBLE|BS_PUSHBUTTON|WS_TABSTOP, 636,388,60,24, hwnd, IDC_BTN_REM_EXCL as isize, hinst, ptr::null_mut());
        SendMessageW(h, 0x0030, fb as usize, 0);
    }

    // Buttons row
    let btns = [
        (IDC_BTN_CLEAN,"Clean Selected",10,536,140,30),
        (IDC_BTN_CLEAN_SAFE,"Clean Safe Only",160,536,140,30),
        (IDC_BTN_SEL_ALL,"Select All",310,536,100,30),
        (IDC_BTN_SEL_NONE,"Select None",420,536,100,30),
        (IDC_BTN_ABOUT,"About",540,536,80,30),
    ];
    for (id,txt,x,y,wd,ht) in btns {
        unsafe {
            let h = CreateWindowExW(0, w("BUTTON").as_ptr(), w(txt).as_ptr(),
                WS_CHILD|WS_VISIBLE|BS_PUSHBUTTON|WS_TABSTOP, x,y,wd,ht, hwnd, id as isize, hinst, ptr::null_mut());
            SendMessageW(h, 0x0030, fb as usize, 0);
        }
    }

    update_mem(hwnd);
    update_estimate(hwnd);
    refresh_exclusion_list(hwnd);
}

extern "system" fn wndproc(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> isize {
    match msg {
        WM_CREATE => {
            let cs = lparam as *const CREATESTRUCTW;
            create_ui(hwnd, unsafe { (*cs).hInstance });
            unsafe {
                G_HWND = hwnd;
                SetTimer(hwnd, 1, 2000, 0);
            }
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as i32;
            let notif = (wparam >> 16) as u32;
            if notif == 0 || notif == 1 || notif == 5 {
                match id {
                    IDC_BTN_CLEAN => {
                        let m = get_mask(hwnd);
                        if m == 0 { msgbox("SA Cleaner", "Select at least one region!", MB_ICONWARNING); }
                        else {
                            let sys = (m & 0x08) != 0 || (m & 0x10) != 0;
                            if sys {
                                let r = msgbox_yn("System Regions", 
                                    "Standby/Modified may pause 1-3s. Continue?", MB_ICONWARNING);
                                if r != IDOK { return 0; }
                            }
                            let exc = unsafe { sa_check_excluded() };
                            if exc != 0 {
                                let r = msgbox_yn("Protected Processes", 
                                    "Protected processes running. Clean anyway?", MB_ICONWARNING);
                                if r != IDOK { return 0; }
                            }
                            do_clean_async(hwnd, m);
                        }
                    }
                    IDC_BTN_CLEAN_SAFE => { do_clean_async(hwnd, 0x6F); }
                    IDC_BTN_SEL_ALL => { sel_all(hwnd, true); }
                    IDC_BTN_SEL_NONE => { sel_all(hwnd, false); }
                    IDC_BTN_ABOUT => {
                        msgbox("About SA Cleaner", 
                            "SA Cleaner v1.0\nBy Seif Afandi\n\n9-Language Memory Cleaner\nResizable | Dark Mode | Threaded", 
                            MB_ICONINFORMATION);
                    }
                    IDC_CHK_AUTO => {
                        let chk = unsafe { IsDlgButtonChecked(hwnd, IDC_CHK_AUTO) };
                        if chk == BST_CHECKED {
                            let thr = get_threshold(hwnd);
                            unsafe { sa_set_auto_threshold(thr); }
                            let msg = format!("Auto-clean enabled at {}% threshold.", thr);
                            msgbox("Auto-Clean", &msg, MB_ICONINFORMATION);
                        } else {
                            unsafe { sa_stop_auto_clean(); }
                            msgbox("Auto-Clean", "Auto-clean disabled.", MB_ICONINFORMATION);
                        }
                    }
                    IDC_CHK_DARK => {
                        let chk = unsafe { IsDlgButtonChecked(hwnd, IDC_CHK_DARK) };
                        apply_dark_mode(hwnd, chk == BST_CHECKED);
                    }
                    IDC_BTN_ADD_EXCL => {
                        let mut buf = vec![0u16; 128];
                        let len = unsafe { GetDlgItemTextW(hwnd, IDC_EDIT_EXCL, buf.as_mut_ptr(), 128) };
                        if len > 0 {
                            let s = String::from_utf16_lossy(&buf[..len as usize]);
                            let trimmed = s.trim();
                            if !trimmed.is_empty() {
                                let cname = CString::new(trimmed).unwrap();
                                unsafe { sa_add_exclusion(cname.as_ptr()); }
                                refresh_exclusion_list(hwnd);
                                unsafe { SetDlgItemTextW(hwnd, IDC_EDIT_EXCL, w("").as_ptr()); }
                            }
                        }
                    }
                    IDC_BTN_REM_EXCL => {
                        let idx = unsafe { SendMessageW(GetDlgItem(hwnd, IDC_LST_EXCL), 0x0188, 0, 0) };
                        if idx >= 0 {
                            let mut buf = vec![0u16; 128];
                            unsafe { SendMessageW(GetDlgItem(hwnd, IDC_LST_EXCL), 0x0189, idx as usize, buf.as_mut_ptr() as isize); }
                            let s = String::from_utf16_lossy(&buf);
                            let trimmed = s.trim();
                            if !trimmed.is_empty() {
                                let cname = CString::new(trimmed).unwrap();
                                unsafe { sa_remove_exclusion(cname.as_ptr()); }
                                refresh_exclusion_list(hwnd);
                            }
                        }
                    }
                    _ => { update_estimate(hwnd); }
                }
            }
            0
        }
        WM_TIMER => {
            update_mem(hwnd);
            0
        }
        WM_ERASEBKGND => {
            unsafe {
                let mut rc: RECT = std::mem::zeroed();
                GetClientRect(hwnd, &mut rc);
                let br = CreateSolidBrush(bg_color());
                FillRect(wparam as isize, &rc, br);
                DeleteObject(br);
                return 1;
            }
        }
        WM_CTLCOLORSTATIC => {
            unsafe {
                SetBkColor(wparam as isize, bg_color());
                SetTextColor(wparam as isize, text_color());
                return CreateSolidBrush(bg_color()) as isize;
            }
        }
        WM_CTLCOLORBTN => {
            unsafe {
                SetBkColor(wparam as isize, panel_color());
                return CreateSolidBrush(panel_color()) as isize;
            }
        }
        WM_CTLCOLOREDIT => {
            unsafe {
                SetBkColor(wparam as isize, panel_color());
                SetTextColor(wparam as isize, text_color());
                return CreateSolidBrush(panel_color()) as isize;
            }
        }
        WM_CTLCOLORLISTBOX => {
            unsafe {
                SetBkColor(wparam as isize, panel_color());
                SetTextColor(wparam as isize, text_color());
                return CreateSolidBrush(panel_color()) as isize;
            }
        }
        WM_CLEAN_DONE => {
            CLEANING.store(false, Ordering::SeqCst);
            let res = wparam as i32;
            let txt = get_err();
            if res == -1 {
                msgbox("Admin Required", &txt, MB_ICONERROR);
            } else if res == 0 {
                msgbox("Success!", &txt, MB_ICONINFORMATION);
            } else {
                msgbox("Results", &txt, MB_ICONWARNING);
            }
            update_status(hwnd, "Ready");
            update_mem(hwnd);
            update_estimate(hwnd);
            0
        }
        WM_SIZE => {
            unsafe { InvalidateRect(hwnd, ptr::null(), 1); }
            0
        }
        WM_DESTROY => {
            unsafe {
                KillTimer(hwnd, 1);
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }
}

fn ensure_admin() {
    let admin = unsafe { sa_is_admin() };
    if admin == 0 {
        let r = msgbox_yn("Administrator Required", 
            "SA Cleaner needs Administrator privileges.\n\nClick OK to restart as Admin.", 
            MB_ICONINFORMATION);
        if r == IDOK {
            unsafe { sa_uac_restart(); }
        }
        std::process::exit(0);
    }
}

fn main() {
    unsafe { sa_set_priority_low(); }
    ensure_admin();

    let hinst = unsafe { GetModuleHandleW(ptr::null()) };
    let cn = w("SACleanerResizable");
    let wc = WNDCLASSEXW { 
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32, 
        style: 0,
        lpfnWndProc: wndproc as usize, 
        cbClsExtra: 0, 
        cbWndExtra: 0, 
        hInstance: hinst,
        hIcon: 0, 
        hCursor: 0, 
        hbrBackground: (COLOR_WINDOW+1) as isize,
        lpszMenuName: ptr::null(), 
        lpszClassName: cn.as_ptr(), 
        hIconSm: 0 
    };
    unsafe {
        RegisterClassExW(&wc);
        // Resizable window: WS_OVERLAPPEDWINDOW includes thickframe, maximize, minimize
        let hwnd = CreateWindowExW(
            0,
            cn.as_ptr(), 
            w("SA Cleaner v1.0 - By Seif Afandi").as_ptr(),
            WS_OVERLAPPEDWINDOW,
            0x80000000u32 as i32, 
            0x80000000u32 as i32, 
            720, 620, 
            0, 0, hinst, ptr::null_mut()
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
