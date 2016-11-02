extern crate winapi;
extern crate user32;
extern crate gdi32;
extern crate comctl32;
extern crate kernel32;

use std::ptr::null_mut;
use std::mem::size_of;
use std::mem::zeroed;
use std::os::raw::c_int;

use winapi::winnt::MAKELANGID;
use winapi::winnt::LANG_NEUTRAL;
use winapi::winnt::SUBLANG_DEFAULT;
use winapi::winuser::*;
use winapi::winbase::*;
use winapi::HDC;
use winapi::DWORD;
use winapi::HINSTANCE;
use winapi::HWND;
use winapi::UINT;
use winapi::WPARAM;
use winapi::LPARAM;
use winapi::LRESULT;
use winapi::HBRUSH;
use winapi::HANDLE;
use winapi::WCHAR;
use winapi::LONG;
use winapi::COLORREF;
use winapi::HGDIOBJ;

type Res<T> = Result<T, String>;

#[link(name = "gdi32")]
extern {
  // why doesn't this show up in gdi32?? it's documented...
  //gdi32::SetBkMode(hdc, winapi::wingdi::TRANSPARENT);
  fn SetBkMode(hdc: HDC, mode: c_int) -> c_int;
}

#[link(name = "shell32")]
extern {
  fn Shell_NotifyIconW(mode: DWORD, iconData: *const NOTIFYICONDATA);
}

macro_rules! TEXT {
  ($x:expr) => {{
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new($x).encode_wide().chain(Some(0)).collect::<Vec<_>>()
  }.as_ptr();}
}

fn to_u16(s: String) -> Vec<u16> {
  use std::ffi::OsStr;
  use std::os::windows::ffi::OsStrExt;
  OsStr::new(&s).encode_wide().chain(Some(0)).collect::<Vec<_>>()
}

const WINDOW_CLASS: &'static str = "Window Hider";
const WINDOW_CLASS_STATIC: &'static str = "Window Hider Static";
const NOTIFYICONDATA_V2_SIZE: DWORD = 936;
const NIF_MESSAGE: UINT = 0x1;
const NIF_ICON: UINT = 0x2;
const NIF_TIP: UINT = 0x4;
const NIF_INFO: UINT = 0x10;
const NIM_ADD: UINT = 0;
const NIM_DELETE: UINT = 2;
const GCLP_HICONSM: winapi::INT = -34;

fn last_error() -> String {
  unsafe {
    let mut buf = [0 as WCHAR; 2048];
    let res = kernel32::FormatMessageW(
      FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
      null_mut(),
      kernel32::GetLastError(),
      MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT) as DWORD,
      buf.as_mut_ptr(), buf.len() as DWORD,
      null_mut()
    );
    if res == 0 {
      "unknown error".to_string()
    } else {
      from_u16(&buf[..(res+1) as usize])
    }
  }
}

fn message_box(hwnd: HWND, message: String, title: String, flags: DWORD) {
  unsafe {
    user32::MessageBoxW(hwnd, TEXT!(&message), TEXT!(&title), flags);
  }
}

struct ToolhelpSnapshot {
  first: bool,
  snapshot: HANDLE,
  pe32: winapi::tlhelp32::PROCESSENTRY32W,
}

impl Iterator for ToolhelpSnapshot {
  type Item = winapi::tlhelp32::PROCESSENTRY32W;
  fn next(&mut self) -> Option<Self::Item> {
    unsafe {
      if self.first {
        self.first = false;
          if kernel32::Process32FirstW(self.snapshot, &mut self.pe32) == 0 {
            None
          } else {
            Some(self.pe32)
          }
      } else {
        if kernel32::Process32NextW(self.snapshot, &mut self.pe32) == 0 {
          None
        } else {
          Some(self.pe32)
        }
      }
    }
  }
}

impl ToolhelpSnapshot {
  fn new() -> Res<ToolhelpSnapshot> {
    unsafe {
      use winapi::tlhelp32::TH32CS_SNAPPROCESS;
      let snapshot = kernel32::CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
      if snapshot == (0 as HANDLE) {
        Err(last_error())
      } else {
        let pe32 = winapi::tlhelp32::PROCESSENTRY32W {
          dwSize: size_of::<winapi::tlhelp32::PROCESSENTRY32W>() as DWORD,
          .. zeroed::<winapi::tlhelp32::PROCESSENTRY32W>()
        };
        Ok(ToolhelpSnapshot { first: true, snapshot: snapshot, pe32: pe32 })
      }
    }
  }
}

impl Drop for ToolhelpSnapshot {
  fn drop(&mut self) {
    unsafe {
      kernel32::CloseHandle(self.snapshot);
    }
  }
}

fn get_process_name(target: HWND) -> Res<String> {
  unsafe {
    let mut pid: DWORD = 0;
    if user32::GetWindowThreadProcessId(target, &mut pid as *mut DWORD) != 0 {
      let ths = ToolhelpSnapshot::new();
      ths.and_then(|mut i| {
        i.find(|pe32| pe32.th32ProcessID == pid)
          .map(|pe32| from_u16(&pe32.szExeFile))
          .ok_or("cannot find process".to_string())
      })
    } else {
      Err(last_error())
    }
  }
}

fn from_u16(s: &[u16]) -> String {
  // panic if there's no null terminator
  let pos = s.iter().position(|a| a == &0u16).unwrap();
  use std::ffi::OsString;
  use std::os::windows::ffi::OsStringExt;
  let s2: OsString = OsStringExt::from_wide(&s[..pos]);
  s2.to_string_lossy().to_string()
}

const IDI_TRAYICON: *const WCHAR= 1101 as *const WCHAR;
fn main() {
  unsafe {
    let hinst = kernel32::GetModuleHandleW(null_mut());
    let wc = WNDCLASSEXW {
      hInstance: hinst,
      cbSize: size_of::<WNDCLASSEXW>() as u32,
      hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
      hCursor: user32::LoadCursorW(0 as HINSTANCE, IDC_ARROW),
      lpszClassName: TEXT!(WINDOW_CLASS),
      
      hIcon: user32::LoadIconW(hinst, IDI_TRAYICON),
      hIconSm: user32::LoadIconW(hinst, IDI_TRAYICON),
      lpfnWndProc: Some(window_proc),
      .. zeroed::<WNDCLASSEXW>()
    };

    let wc2 = WNDCLASSW {
      style: CS_HREDRAW | CS_VREDRAW,
      lpfnWndProc: Some(user32::DefWindowProcW),
      hInstance: hinst,
      hbrBackground: (COLOR_WINDOW + 1) as HBRUSH,
      lpszClassName: TEXT!(WINDOW_CLASS_STATIC),
      hCursor: user32::LoadCursorW(0 as HINSTANCE, IDC_ARROW),
      .. zeroed::<WNDCLASSW>()
    };

    let icc = winapi::commctrl::INITCOMMONCONTROLSEX {
      dwSize: size_of::<winapi::commctrl::INITCOMMONCONTROLSEX>() as DWORD,
      dwICC: winapi::commctrl::ICC_WIN95_CLASSES,
    };
    comctl32::InitCommonControlsEx(&icc);

    let r = user32::RegisterClassExW(&wc) != 0 && user32::RegisterClassW(&wc2) != 0;
    if !r {
      message_box(0 as HWND, last_error(), "RegisterClass Failed".to_string(), MB_ICONERROR | MB_OK);
    } else {
      let hwnd = user32::CreateWindowExW(0 as DWORD,
        TEXT!(WINDOW_CLASS),
        TEXT!(WINDOW_CLASS),
        WS_OVERLAPPEDWINDOW ^ (WS_SIZEBOX | WS_MAXIMIZEBOX | WS_MINIMIZEBOX),
        CW_USEDEFAULT, CW_USEDEFAULT,
        320, 200,
        null_mut(), null_mut(), hinst, null_mut());
      if hwnd == (0 as HWND) {
        message_box(null_mut(), last_error(), "CreateWindow Error".to_string(), MB_ICONERROR | MB_OK);
      } else {
        user32::ShowWindow(hwnd, SW_SHOW);
        user32::SetWindowPos(hwnd, -1isize as HWND, 0, 0, 0, 0,
          SWP_NOMOVE | SWP_NOSIZE);

        let mut msg = zeroed::<MSG>();
        while user32::GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
          user32::TranslateMessage(&msg);
          user32::DispatchMessageW(&msg);
        }
      }
    }
  }
}

#[repr(C)] #[derive(Copy)] #[allow(non_snake_case)]
pub struct NOTIFYICONDATA {
  cbSize: DWORD,
  hWnd: HWND,
  uID: UINT,
  uFlags: UINT,
  uCallbackMessage: UINT,
  hIcon: winapi::HICON,
  szTip: [WCHAR; 128],
  dwState: DWORD,
  dwStateMask: DWORD,
  szInfo: [WCHAR; 256],
  uVersion: UINT,
  szInfoTitle: [WCHAR; 64],
  dwInfoFlags: DWORD,
  guidItem: winapi::GUID,
  hBalloonIcon: winapi::HICON,
}
impl Clone for NOTIFYICONDATA{ fn clone(&self) -> NOTIFYICONDATA { *self } }

fn cpy<T: Copy>(dst: &mut [T], src: &[T]) {
  let len = std::cmp::min(dst.len(), src.len());

  for i in 0..len {
    dst[i] = src[i]
  }
}

pub unsafe extern "system" fn window_proc(
  hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
  static mut WM_TRAY_ICON_NOTIFYICON: UINT = 0;
  static mut PARENT: HWND = 0 as HWND;
  static mut ICON_STATIC: HWND = 0 as HWND;
  static mut HBR_STATIC: HBRUSH = 0 as HBRUSH;
  static mut TARGET_HWND: HWND = 0 as HWND;
  if WM_TRAY_ICON_NOTIFYICON == 0 {
    WM_TRAY_ICON_NOTIFYICON = user32::RegisterWindowMessageW(TEXT!("NotifyIcon"));
  }
  static mut ICON_DATA: NOTIFYICONDATA = NOTIFYICONDATA {
    cbSize: NOTIFYICONDATA_V2_SIZE,
    hWnd: 0 as HWND,
    uID: 0,
    uFlags: NIF_ICON | NIF_INFO | NIF_TIP | NIF_MESSAGE | 0x80,
    uCallbackMessage: 0,
    hIcon: 0 as winapi::HICON,
    szTip: [0; 128],
    dwState: 0,
    dwStateMask: 0,
    szInfo: [0; 256],
    uVersion: 4,
    szInfoTitle: [0; 64],
    dwInfoFlags: 0x4, // NIIF_USER
    guidItem: winapi::GUID { Data1: 0, Data2: 0, Data3: 0, Data4: [0; 8] },
    hBalloonIcon: 0 as winapi::HICON,
  };
  if PARENT == (0 as HWND) {
    PARENT = hwnd;
  }

  match msg {
    x if x == WM_TRAY_ICON_NOTIFYICON => {
      if (lparam as UINT) == WM_LBUTTONUP {
        Shell_NotifyIconW(NIM_DELETE, &ICON_DATA);
        user32::ShowWindow(TARGET_HWND, SW_RESTORE);
        user32::SetForegroundWindow(TARGET_HWND);
        user32::PostQuitMessage(0);
      }
      0 as LRESULT
    },
    WM_PAINT => if hwnd == ICON_STATIC {
      let mut rc = zeroed::<winapi::windef::RECT>();
      let mut ps = zeroed::<PAINTSTRUCT>();

      let hdc = user32::BeginPaint(hwnd, &mut ps);
      let cursor = user32::LoadCursorW(0 as HINSTANCE, IDC_CROSS);
      user32::GetClientRect(hwnd, &mut rc);
      let w = user32::GetSystemMetrics(SM_CXCURSOR);
      user32::DrawIcon(hdc, (rc.right - rc.left - w) / 2, 0, cursor);
      SetBkMode(hdc, winapi::wingdi::TRANSPARENT);
      user32::EndPaint(hwnd, &ps);
      gdi32::DeleteObject(cursor as HGDIOBJ);
      0 as LRESULT
    } else {
      user32::DefWindowProcW(hwnd, msg, wparam, lparam)
    },
    WM_CTLCOLORSTATIC => {
      let hdc = wparam as HDC;
      let c = user32::GetSysColor(COLOR_WINDOW);
      gdi32::SetBkColor(hdc, c as COLORREF);
      if HBR_STATIC == (0 as HBRUSH) {
        HBR_STATIC = gdi32::CreateSolidBrush(c as COLORREF);
      }
      HBR_STATIC as LRESULT
    },
    WM_CREATE => {
      let mut r = zeroed::<winapi::windef::RECT>();
      
      if hwnd != ICON_STATIC {
        const SS_CENTER: DWORD = 1;
        user32::GetWindowRect(hwnd, &mut r);
        user32::CreateWindowExW(0, TEXT!("STATIC"),
          TEXT!("Drag the cursor into any window to hide it"),
          WS_CHILD | WS_VISIBLE | SS_CENTER,
          0, 16, r.right - r.left, 64,
          hwnd, null_mut(), kernel32::GetModuleHandleW(null_mut()), null_mut());

        ICON_STATIC = user32::CreateWindowExW(0, TEXT!(WINDOW_CLASS_STATIC),
          TEXT!(WINDOW_CLASS_STATIC),
          WS_CHILD | WS_VISIBLE,
          0, 64, r.right - r.left, r.bottom - r.top - 96,
          hwnd, null_mut(), kernel32::GetModuleHandleW(null_mut()), null_mut());
        user32::SetWindowLongPtrW(ICON_STATIC,
          GWLP_WNDPROC, window_proc as winapi::LONG_PTR);
        if ICON_STATIC == (0 as HWND) {
          message_box(hwnd, last_error(), "Error".to_string(), MB_ICONERROR | MB_OK);
        }
      }
      0 as LRESULT
    },
    WM_DESTROY => {
      user32::PostQuitMessage(0);
      gdi32::DeleteObject(HBR_STATIC as HGDIOBJ);
      0 as LRESULT
    },
    WM_CLOSE => {
      user32::DestroyWindow(hwnd);
      0 as LRESULT
    },
    WM_LBUTTONDOWN => {
      user32::SetCapture(hwnd);
      user32::ShowWindow(ICON_STATIC, SW_HIDE);
      user32::SetCursor(user32::LoadCursorW(0 as HINSTANCE, IDC_CROSS));
      0 as LRESULT
    },
    WM_LBUTTONUP => {
      const CWP_SKIPINVISIBLE: UINT = 1;
      const CWP_SKIPDISABLED: UINT = 2;
      const CWP_SKIPTRANSPARENT: UINT = 4;

      user32::ReleaseCapture();
      let mut p = winapi::POINT {
        x: winapi::minwindef::LOWORD(lparam as DWORD) as LONG,
        y: winapi::minwindef::HIWORD(lparam as DWORD) as LONG
      };
      user32::ClientToScreen(hwnd, &mut p);
      let target = user32::ChildWindowFromPointEx(user32::GetDesktopWindow(), p,
        CWP_SKIPDISABLED | CWP_SKIPINVISIBLE | CWP_SKIPTRANSPARENT);
      user32::ShowWindow(ICON_STATIC, SW_RESTORE);
      if target == PARENT {
        message_box(hwnd, "Unable to hide self".to_string(), "Error".to_string(), MB_ICONERROR | MB_OK);
      } else if target != (0 as HWND) {
        match get_process_name(target) {
          Ok(n) => {
            if n == "explorer.exe".to_string() {
              message_box(hwnd,
                "Cannot hide the Windows Shell, try again".to_string(), "Error".to_string(),
                MB_ICONERROR | MB_OK);
            } else {
              let mut title = [0 as WCHAR; winapi::minwindef::MAX_PATH];
              // need actual size, not NOTIFYICONDATA_V2_SIZE or bubble won't show
              ICON_DATA.cbSize = size_of::<NOTIFYICONDATA>() as DWORD;
              ICON_DATA.hWnd = hwnd;
              ICON_DATA.uID =  target as UINT;
              ICON_DATA.uCallbackMessage = WM_TRAY_ICON_NOTIFYICON;
              user32::GetWindowTextW(target, title.as_mut_ptr(), winapi::minwindef::MAX_PATH as winapi::INT);
              let sz_tip = format!("[0x{0:x}] - {1}", target as UINT, from_u16(&title));
              cpy(&mut ICON_DATA.szTip, to_u16(sz_tip).as_ref());

              let icon = user32::GetClassLongPtrW(target, GCLP_HICONSM) as winapi::HICON;
              ICON_DATA.hIcon = icon;
              ICON_DATA.hBalloonIcon = icon;
              cpy(&mut ICON_DATA.szInfoTitle, to_u16("Window Hidden".to_string()).as_ref());

              let sz_info = format!("[0x{0:08x}] - {1}", target as UINT, from_u16(&title));
              cpy(&mut ICON_DATA.szInfo, to_u16(sz_info).as_ref());
              Shell_NotifyIconW(NIM_ADD, &ICON_DATA);
              user32::ShowWindow(target, SW_HIDE);
              user32::ShowWindow(PARENT, SW_HIDE);
              TARGET_HWND = target;
            }
          },
          Err(s) => message_box(hwnd, s, "Failed to identify process".to_string(), MB_ICONERROR | MB_OK),
        }
      } else {
        message_box(hwnd,
          "Failed to identify window, try again".to_string(), "Error".to_string(),
          MB_ICONERROR | MB_OK);
      }

      0 as LRESULT
    },
    _ => user32::DefWindowProcW(hwnd, msg, wparam, lparam),
  }
}

// vim: sw=2
