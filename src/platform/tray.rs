use std::path::PathBuf;
use std::sync::atomic::{AtomicIsize, AtomicU8, Ordering};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NotificationTarget {
    TasksActive = 0,
    TasksCompleted = 1,
    QueueTab = 2,
}

impl NotificationTarget {
    fn from_u8(val: u8) -> Self {
        match val {
            1 => Self::TasksCompleted,
            2 => Self::QueueTab,
            _ => Self::TasksActive,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    ShowMainWindow,
    ShowAddPopup,
    NotificationClicked(NotificationTarget),
    Quit,
}

pub struct SystemTray;

impl SystemTray {
    fn resolve_ico_path() -> Option<PathBuf> {
        let cwd_ico = PathBuf::from(r"assets\logo\speedflow.ico");
        if cwd_ico.exists() {
            return Some(cwd_ico);
        }
        if let Ok(mut exe) = std::env::current_exe() {
            exe.pop();
            let cand1 = exe.join(r"assets\logo\speedflow.ico");
            if cand1.exists() {
                return Some(cand1);
            }
            exe.pop();
            exe.pop();
            let cand2 = exe.join(r"assets\logo\speedflow.ico");
            if cand2.exists() {
                return Some(cand2);
            }
        }
        None
    }

    #[cfg(windows)]
    pub fn spawn(tx: UnboundedSender<TrayEvent>) {
        std::thread::spawn(move || {
            unsafe {
                run_win32_tray_loop(tx);
            }
        });
    }

    #[cfg(not(windows))]
    pub fn spawn(_tx: UnboundedSender<TrayEvent>) {}

    #[cfg(windows)]
    pub fn show_balloon(title: &str, body: &str, target: NotificationTarget) {
        LAST_BALLOON_TARGET.store(target as u8, Ordering::Release);
        let hwnd = TRAY_HWND.load(Ordering::Acquire);
        let h_icon = TRAY_HICON.load(Ordering::Acquire);
        if hwnd == 0 {
            return;
        }

        unsafe {
            let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
            nid.cb_size = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.h_wnd = hwnd;
            nid.u_id = 1;
            nid.u_flags = NIF_INFO;
            nid.dw_info_flags = NIIF_INFO;
            nid.h_balloon_icon = h_icon;

            let title_wide = to_wide(title);
            let title_len = title_wide.len().min(63);
            nid.sz_info_title[..title_len].copy_from_slice(&title_wide[..title_len]);

            let body_wide = to_wide(body);
            let body_len = body_wide.len().min(255);
            nid.sz_info[..body_len].copy_from_slice(&body_wide[..body_len]);

            Shell_NotifyIconW(NIM_MODIFY, &nid);
        }
    }

    #[cfg(not(windows))]
    pub fn show_balloon(_title: &str, _body: &str, _target: NotificationTarget) {}
}

#[cfg(windows)]
const WM_USER: u32 = 0x0400;
#[cfg(windows)]
const WM_TRAYICON: u32 = WM_USER + 1;
#[cfg(windows)]
const WM_COMMAND: u32 = 0x0111;
#[cfg(windows)]
const WM_DESTROY: u32 = 0x0002;
#[cfg(windows)]
const WM_NULL: u32 = 0x0000;
#[cfg(windows)]
const WM_LBUTTONUP: u32 = 0x0202;
#[cfg(windows)]
const WM_LBUTTONDBLCLK: u32 = 0x0203;
#[cfg(windows)]
const WM_RBUTTONUP: u32 = 0x0205;
#[cfg(windows)]
const WM_CONTEXTMENU: u32 = 0x007B;

#[cfg(windows)]
const NIM_ADD: u32 = 0x00000000;
#[cfg(windows)]
const NIM_MODIFY: u32 = 0x00000001;
#[cfg(windows)]
const NIM_DELETE: u32 = 0x00000002;
#[cfg(windows)]
const NIF_MESSAGE: u32 = 0x00000001;
#[cfg(windows)]
const NIF_ICON: u32 = 0x00000002;
#[cfg(windows)]
const NIF_TIP: u32 = 0x00000004;
#[cfg(windows)]
const NIF_INFO: u32 = 0x00000010;
#[cfg(windows)]
const NIIF_INFO: u32 = 0x00000001;
#[cfg(windows)]
const NIN_BALLOONUSERCLICK: u32 = WM_USER + 5;

#[cfg(windows)]
const IMAGE_ICON: u32 = 1;
#[cfg(windows)]
const LR_LOADFROMFILE: u32 = 0x00000010;

#[cfg(windows)]
const MF_STRING: u32 = 0x00000000;
#[cfg(windows)]
const MF_SEPARATOR: u32 = 0x00000800;
#[cfg(windows)]
const TPM_BOTTOMALIGN: u32 = 0x0020;
#[cfg(windows)]
const TPM_LEFTALIGN: u32 = 0x0000;
#[cfg(windows)]
const TPM_RETURNCMD: u32 = 0x0100;
#[cfg(windows)]
const TPM_RIGHTBUTTON: u32 = 0x0002;

#[cfg(windows)]
const ID_TRAY_OPEN: usize = 1001;
#[cfg(windows)]
const ID_TRAY_ADD: usize = 1002;
#[cfg(windows)]
const ID_TRAY_QUIT: usize = 1003;

#[cfg(windows)]
#[repr(C)]
struct GUID {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[cfg(windows)]
#[repr(C)]
struct NOTIFYICONDATAW {
    cb_size: u32,
    h_wnd: isize,
    u_id: u32,
    u_flags: u32,
    u_callback_message: u32,
    h_icon: isize,
    sz_tip: [u16; 128],
    dw_state: u32,
    dw_state_mask: u32,
    sz_info: [u16; 256],
    u_timeout_or_version: u32,
    sz_info_title: [u16; 64],
    dw_info_flags: u32,
    guid_item: GUID,
    h_balloon_icon: isize,
}

#[cfg(windows)]
#[repr(C)]
struct WNDCLASSW {
    style: u32,
    lpfn_wnd_proc: unsafe extern "system" fn(isize, u32, usize, isize) -> isize,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: isize,
    h_icon: isize,
    h_cursor: isize,
    hbr_background: isize,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
}

#[cfg(windows)]
#[repr(C)]
struct POINT {
    x: i32,
    y: i32,
}

#[cfg(windows)]
#[repr(C)]
struct MSG {
    hwnd: isize,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt: POINT,
}

#[cfg(windows)]
#[link(name = "shell32")]
unsafe extern "system" {
    fn Shell_NotifyIconW(dwMessage: u32, lpData: *const NOTIFYICONDATAW) -> i32;
}

#[cfg(windows)]
#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: isize,
        hMenu: isize,
        hInstance: isize,
        lpParam: isize,
    ) -> isize;
    fn DefWindowProcW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: isize, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn PostQuitMessage(nExitCode: i32);
    fn PostMessageW(hWnd: isize, Msg: u32, wParam: usize, lParam: isize) -> i32;
    fn LoadImageW(
        hInst: isize,
        name: *const u16,
        type_: u32,
        cx: i32,
        cy: i32,
        fuLoad: u32,
    ) -> isize;
    fn LoadIconW(hInstance: isize, lpIconName: *const u16) -> isize;
    fn CreatePopupMenu() -> isize;
    fn AppendMenuW(hMenu: isize, uFlags: u32, uIDNewItem: usize, lpNewItem: *const u16) -> i32;
    fn TrackPopupMenu(
        hMenu: isize,
        uFlags: u32,
        x: i32,
        y: i32,
        nReserved: i32,
        hWnd: isize,
        prcRect: isize,
    ) -> i32;
    fn DestroyMenu(hMenu: isize) -> i32;
    fn GetCursorPos(lpPoint: *mut POINT) -> i32;
    fn SetForegroundWindow(hWnd: isize) -> i32;
}

#[cfg(windows)]
static TRAY_SENDER: std::sync::OnceLock<UnboundedSender<TrayEvent>> = std::sync::OnceLock::new();
#[cfg(windows)]
static TRAY_HWND: AtomicIsize = AtomicIsize::new(0);
#[cfg(windows)]
static TRAY_HICON: AtomicIsize = AtomicIsize::new(0);
#[cfg(windows)]
static LAST_BALLOON_TARGET: AtomicU8 = AtomicU8::new(0);

#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(windows)]
unsafe extern "system" fn tray_wnd_proc(
    hwnd: isize,
    msg: u32,
    w_param: usize,
    l_param: isize,
) -> isize {
    unsafe {
        match msg {
            WM_TRAYICON => {
                let mouse_msg = (l_param as u32) & 0xFFFF;
                if mouse_msg == WM_LBUTTONUP || mouse_msg == WM_LBUTTONDBLCLK {
                    if let Some(tx) = TRAY_SENDER.get() {
                        let _ = tx.send(TrayEvent::ShowMainWindow);
                    }
                } else if mouse_msg == NIN_BALLOONUSERCLICK {
                    if let Some(tx) = TRAY_SENDER.get() {
                        let target = NotificationTarget::from_u8(LAST_BALLOON_TARGET.load(Ordering::Acquire));
                        let _ = tx.send(TrayEvent::NotificationClicked(target));
                    }
                } else if mouse_msg == WM_RBUTTONUP || mouse_msg == WM_CONTEXTMENU {
                    let hmenu = CreatePopupMenu();
                    if hmenu != 0 {
                        let open_str = to_wide("Open SpeedFlow");
                        let add_str = to_wide("Add New Download...");
                        let quit_str = to_wide("Quit SpeedFlow");

                        AppendMenuW(hmenu, MF_STRING, ID_TRAY_OPEN, open_str.as_ptr());
                        AppendMenuW(hmenu, MF_STRING, ID_TRAY_ADD, add_str.as_ptr());
                        AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
                        AppendMenuW(hmenu, MF_STRING, ID_TRAY_QUIT, quit_str.as_ptr());

                        let mut pt = POINT { x: 0, y: 0 };
                        GetCursorPos(&mut pt);
                        SetForegroundWindow(hwnd);

                        let cmd = TrackPopupMenu(
                            hmenu,
                            TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON | TPM_RETURNCMD,
                            pt.x,
                            pt.y,
                            0,
                            hwnd,
                            0,
                        ) as usize;

                        PostMessageW(hwnd, WM_NULL, 0, 0);
                        DestroyMenu(hmenu);

                        if let Some(tx) = TRAY_SENDER.get() {
                            match cmd {
                                ID_TRAY_OPEN => {
                                    let _ = tx.send(TrayEvent::ShowMainWindow);
                                }
                                ID_TRAY_ADD => {
                                    let _ = tx.send(TrayEvent::ShowAddPopup);
                                }
                                ID_TRAY_QUIT => {
                                    let _ = tx.send(TrayEvent::Quit);
                                    PostQuitMessage(0);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                0
            }
            WM_COMMAND => 0,
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, w_param, l_param),
        }
    }
}

#[cfg(windows)]
unsafe fn run_win32_tray_loop(tx: UnboundedSender<TrayEvent>) {
    unsafe {
        let _ = TRAY_SENDER.set(tx);

        let class_name = to_wide("SpeedFlowTrayWindowClass");
        let wnd_class = WNDCLASSW {
            style: 0,
            lpfn_wnd_proc: tray_wnd_proc,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: 0,
            h_icon: 0,
            h_cursor: 0,
            hbr_background: 0,
            lpsz_menu_name: std::ptr::null(),
            lpsz_class_name: class_name.as_ptr(),
        };

        RegisterClassW(&wnd_class);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        );

        if hwnd == 0 {
            return;
        }

        let mut h_icon = 0isize;
        if let Some(ico_path) = SystemTray::resolve_ico_path() {
            let wide_ico = to_wide(&ico_path.to_string_lossy());
            h_icon = LoadImageW(0, wide_ico.as_ptr(), IMAGE_ICON, 32, 32, LR_LOADFROMFILE);
        }
        if h_icon == 0 {
            const IDI_APPLICATION: *const u16 = 32512 as *const u16;
            h_icon = LoadIconW(0, IDI_APPLICATION);
        }

        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cb_size = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.h_wnd = hwnd;
        nid.u_id = 1;
        nid.u_flags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.u_callback_message = WM_TRAYICON;
        nid.h_icon = h_icon;

        let tip = to_wide("SpeedFlow — Download Manager (Running in Background)");
        let copy_len = tip.len().min(127);
        nid.sz_tip[..copy_len].copy_from_slice(&tip[..copy_len]);

        Shell_NotifyIconW(NIM_ADD, &nid);
        TRAY_HWND.store(hwnd, Ordering::Release);
        TRAY_HICON.store(h_icon, Ordering::Release);

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, 0, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        TRAY_HWND.store(0, Ordering::Release);
        Shell_NotifyIconW(NIM_DELETE, &nid);
    }
}
