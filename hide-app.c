#define WINVER 0x0600
#define NTDDI_VERSION NTDDI_VISTA

#define _WIN32_IE 0x0500

#include <windows.h>
#include <stdlib.h>
#include <stdio.h>
#include <shellapi.h>

/*
struct NAME_HANDLE {
    char* name;
    HWND hwnd;
    BOOL found;
};

BOOL CALLBACK WindowFoundCB(HWND hwnd, LPARAM param) {
    LPTSTR str = (LPTSTR) param;
    GetWindowText(hwnd, str, 256);
    if (!IsWindowVisible(hwnd))
        return TRUE;
    if (strlen(str) == 0)
        return TRUE;
    printf("0x%08x -- %s\n", hwnd, str);
    return TRUE;
}

BOOL CALLBACK FindWindowCB(HWND hwnd, LPARAM param) {
    struct NAME_HANDLE *nh = (struct NAME_HANDLE *) param;
    if (nh->found)
        return FALSE;
    LPTSTR str = (LPTSTR) malloc(256);
    GetWindowText(hwnd, str, 256);
    char *c = strstr(str, GetCommandLine());
    char *r = strstr(str, nh->name);
    free(str);
    if (c != NULL)
        return TRUE;
    if (r != NULL) {
        nh->found = TRUE;
        nh->hwnd  = hwnd;
    }
    return TRUE;
}

void error(const char* prefix) {
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
        
    fprintf(stderr, "%s%s\n", prefix, lpMsgBuf);
}
int main(int argc, char** argv) {
    if (argc < 2) {
        LPTSTR str = (LPTSTR) malloc(256);
        EnumWindows(WindowFoundCB, (LPARAM) str);
        free(str);
    } else {
        BOOL hide = TRUE;
        char *wnd = argv[1];
        char *result = wnd;
        int handle = strtol(wnd, &result, 0);

        if (result[0] != '\0') {
            struct NAME_HANDLE nh;
            hide = wnd[0] != '-';
            char *name = wnd[0] == '-' || wnd[0] == '+' ? wnd+1 : wnd;
            nh.found = FALSE;
            nh.name = name;
            EnumWindows(FindWindowCB, (LPARAM) &nh);
            if (!nh.found) {
                fprintf(stderr, "Invalid window handle: %s\n", wnd);
                return 1;
            }
            handle = (int) nh.hwnd;
        }
        if (handle < 0) {
            handle = abs(handle);
            hide = FALSE;
        }
        HWND hwnd = (HWND) handle;

        int flag = hide ? SW_HIDE : SW_RESTORE;
        if (!IsWindow(hwnd)) {
            fprintf(stderr, "%s is not a valid window id\n", wnd);
            return 1;
        }
        NOTIFYICONDATA iconData;
        TCHAR title[256];
        iconData.cbSize = NOTIFYICONDATA_V2_SIZE; // required to show bubble
        iconData.hWnd = hwnd;
        iconData.uID = hwnd;
        iconData.uFlags = NIF_ICON | NIF_TIP | NIF_INFO;
        GetWindowText(hwnd, title, 256);
        snprintf(iconData.szTip, 64, "[0x%x] - %s", hwnd, title);
        
        if (hide) {
            HICON icon = (HICON) GetClassLongPtr(hwnd, GCLP_HICONSM);
            iconData.hIcon = icon;
            iconData.hBalloonIcon = icon;
            strncpy(iconData.szInfoTitle, "Window Hidden", 64);
            snprintf(iconData.szInfo, 256, "[0x%08x] - %s", hwnd, title);
            iconData.dwInfoFlags = NIIF_INFO;
        }
        Shell_NotifyIcon(hide ? NIM_ADD : NIM_DELETE, &iconData);
        ShowWindow(hwnd, flag);
    }
    return 0;
}
*/

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
            //HWND target = WindowFromPoint(p);
            if (target) {
                TCHAR str[256];
                GetWindowText(target, str, 256);
                MessageBox(NULL,
                        str, str, MB_ICONERROR | MB_OK);
                PostQuitMessage(0);
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
    wc.hbrBackground = (HBRUSH)(COLOR_WINDOW+1);
    wc.hIcon = LoadIcon(NULL, IDI_APPLICATION);
    wc.hIconSm = LoadIcon(NULL, IDI_APPLICATION);
    wc.hCursor = LoadCursor(NULL, IDC_ARROW);
    wc.lpszClassName = MainWndClass;

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

