#define WINVER 0x0600
#define NTDDI_VERSION NTDDI_VISTA

#define _WIN32_IE 0x0500

#include <windows.h>
#include <stdlib.h>
#include <stdio.h>
#include <shellapi.h>
#include <commctrl.h>

#include "resource.h"

NOTIFYICONDATA iconData;

UINT WM_TRAY_ICON_NOTIFYICON;
HWND targetHwnd;

LPTSTR error() {
    LPTSTR lpMsgBuf;
    FormatMessage(
             FORMAT_MESSAGE_ALLOCATE_BUFFER | 
             FORMAT_MESSAGE_FROM_SYSTEM |
             FORMAT_MESSAGE_IGNORE_INSERTS,
             NULL,
             GetLastError(),
             MAKELANGID(LANG_NEUTRAL, SUBLANG_DEFAULT),
             (LPTSTR) &lpMsgBuf,
             0, NULL );
        
    return lpMsgBuf;
}

LRESULT CALLBACK WindowProc(HWND hWnd, UINT msg, WPARAM wParam, LPARAM lParam)
{
    if (msg == WM_TRAY_ICON_NOTIFYICON) {

        switch ((UINT)lParam) {
            case WM_LBUTTONDBLCLK:
                Shell_NotifyIcon(NIM_DELETE, &iconData);
                ShowWindow(targetHwnd, SW_RESTORE);
                PostQuitMessage(0);
                break;
        }
        return 0;
    }
    switch (msg) {
        case WM_CLOSE:
            DestroyWindow(hWnd);
            PostQuitMessage(0);
            break;
        case WM_CAPTURECHANGED:
            break;
        case WM_LBUTTONDOWN:
            SetCapture(hWnd);
            SetCursor(LoadCursor(NULL, IDC_CROSS));
            break;
        case WM_LBUTTONUP:
            ReleaseCapture();
            POINT p;
            p.x = LOWORD(lParam);
            p.y = HIWORD(lParam);
            ClientToScreen(hWnd, &p);
            HWND target = ChildWindowFromPointEx(GetDesktopWindow(), p,
                    CWP_SKIPDISABLED | CWP_SKIPINVISIBLE | CWP_SKIPTRANSPARENT);
            if (target == hWnd) {
                MessageBox(NULL,
                        TEXT("Unable to hide self"),
                        TEXT("Error"),
                        MB_ICONERROR | MB_OK);
            } else if (target) {
                TCHAR title[256];

                // required to show bubble
                iconData.cbSize = NOTIFYICONDATA_V2_SIZE;
                iconData.hWnd = hWnd;
                iconData.uID = (UINT) target;
                iconData.uFlags = NIF_ICON | NIF_TIP | NIF_INFO | NIF_MESSAGE;
                iconData.uCallbackMessage = WM_TRAY_ICON_NOTIFYICON;
                GetWindowText(target, title, 256);
                snprintf(iconData.szTip, 64,
                        "[0x%x] - %s", (UINT) target, title);

                HICON icon = (HICON) GetClassLongPtr(target, GCLP_HICONSM);
                iconData.hIcon = icon;
                iconData.hBalloonIcon = icon;
                strncpy(iconData.szInfoTitle, "Window Hidden", 64);
                snprintf(iconData.szInfo, 256,
                        "[0x%08x] - %s", (UINT) target, title);
                iconData.dwInfoFlags = 0x4; // NIIF_USER
                Shell_NotifyIcon(NIM_ADD, &iconData);
                ShowWindow(target, SW_HIDE);
                ShowWindow(hWnd, SW_HIDE);
                targetHwnd = target;
            } else {
                MessageBox(NULL,
                        TEXT("Unable to identify window, try again"),
                        TEXT("ChildWindowFromPoint Error"),
                        MB_ICONERROR | MB_OK);
            }
            break;
        default:
            return DefWindowProc(hWnd, msg, wParam, lParam);
    }
    return 0;
}


int WINAPI WinMain(HINSTANCE hInstance,
        HINSTANCE hPrevInstance,
        LPSTR lpCmdLine,
        int nCmdShow) {
    WM_TRAY_ICON_NOTIFYICON = RegisterWindowMessage(TEXT("NotifyIcon"));
    HWND hWnd;
    MSG msg;
    WNDCLASSEX wc;
    LPCTSTR MainWndClass = TEXT("Window Hider");
    wc.cbSize = sizeof(wc);
    wc.style = 0;
    wc.lpfnWndProc = &WindowProc;
    wc.cbClsExtra = 0;
    wc.cbWndExtra = 0;
    wc.hInstance = hInstance;
    wc.lpszMenuName = NULL;
    wc.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
    wc.hIcon   = LoadIcon(hInstance, MAKEINTRESOURCE(IDI_TRAYICON));
    wc.hIconSm = LoadIcon(hInstance, MAKEINTRESOURCE(IDI_TRAYICON));
    wc.hCursor = LoadCursor(NULL, IDC_ARROW);
    wc.lpszClassName = MainWndClass;

    INITCOMMONCONTROLSEX icc;

    // Initialise common controls.
    icc.dwSize = sizeof(icc);
    icc.dwICC = ICC_WIN95_CLASSES;
    InitCommonControlsEx(&icc);

    if (!RegisterClassEx(&wc)) {
        LPTSTR err = error();
        MessageBox(NULL,
                err, TEXT("RegisterClassEx Error"), MB_ICONERROR | MB_OK);
        return 0;
    }

    hWnd = CreateWindowEx(0, MainWndClass, MainWndClass,
            WS_OVERLAPPEDWINDOW, CW_USEDEFAULT, CW_USEDEFAULT,
            320, 200, NULL, NULL, hInstance, NULL);

    if (!hWnd) {
        LPTSTR err = error();
        MessageBox(NULL,
                err, TEXT("CreateWindowEx Error"), MB_ICONERROR | MB_OK);
        return 0;
    }

    ShowWindow(hWnd, nCmdShow);
    while (GetMessage(&msg, NULL, 0, 0) > 0) {
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
    return (int) msg.wParam;
}

