#include "engine.h"
#include <windows.h>
#include <string.h>
#include <stdio.h>
#include <psapi.h>
#include <shlobj.h>
#include <shellapi.h>

#pragma comment(lib, "psapi.lib")
#pragma comment(lib, "shell32.lib")
#pragma comment(lib, "advapi32.lib")

extern "C" {
    typedef LONG NTSTATUS;
    typedef enum _SYS_INFO_CLASS {
        SysMemList = 80,
        SysRegRec = 166,
        SysCombine = 174,
        SysFileCache = 21
    } SYS_INFO_CLASS;
    typedef enum _MEM_CMD {
        EmptyWS = 1,
        FlushMod = 2,
        PurgeStandby = 3,
        PurgeLowPrio = 4,
        PurgeStandbyPriority = 5
    } MEM_CMD;
    typedef struct _FILECACHE_INFO {
        SIZE_T cs, ps;
        ULONG pfc;
        SIZE_T mi, ma;
        SIZE_T csit, psit;
        ULONG trpc, fl;
    } FILECACHE_INFO;
    typedef struct _COMBINE_INFO {
        SIZE_T hc, pc;
        ULONG fl;
    } COMBINE_INFO;
    typedef struct _SYSTEM_MEMORY_LIST_INFORMATION {
        SIZE_T ZeroPageCount;
        SIZE_T FreePageCount;
        SIZE_T ModifiedPageCount;
        SIZE_T ModifiedNoWritePageCount;
        SIZE_T BadPageCount;
        SIZE_T PageCountByPriority[8];
        SIZE_T RepurposedPagesByPriority[8];
        SIZE_T ModifiedPageCountPageFile;
    } SYSTEM_MEMORY_LIST_INFORMATION;
    #define ST_OK ((NTSTATUS)0L)
    #define ST_NS ((NTSTATUS)0xC00000BBL)
    #define ST_IC ((NTSTATUS)0xC0000003L)
}

static char g_err[8192] = {0};
static int g_ok = 0, g_try = 0;
static unsigned int g_auto_threshold = 0;
static BOOL g_auto_running = FALSE;
static HANDLE g_auto_thread = NULL;
static int g_aggressive = 0;
static unsigned int g_last_cleaned_mb = 0;
static CRITICAL_SECTION g_cs;
static BOOL g_cs_init = FALSE;

static wchar_t g_user_exclusions[64][64] = {0};
static int g_user_excl_count = 0;

static const wchar_t* g_builtin_exclusions[] = {
    L"explorer.exe", L"csrss.exe", L"lsass.exe", L"services.exe",
    L"svchost.exe", L"winlogon.exe", L"smss.exe", L"fontdrvhost.exe",
    L"dwm.exe", L"memory_compression", L"system", L"registry",
    L"secure system", L"wininit.exe", L"sihost.exe", L"taskhostw.exe",
    L"runtimebroker.exe", L"searchindexer.exe", L"searchhost.exe",
    L"startmenuexperiencehost.exe", L"shellhost.exe", L"ctfmon.exe",
    L"conhost.exe", L"audiodg.exe", L"spoolsv.exe", L"dllhost.exe"
};

static void init_cs() {
    if (!g_cs_init) {
        InitializeCriticalSection(&g_cs);
        g_cs_init = TRUE;
    }
}

static BOOL is_adm() {
    BOOL e = FALSE;
    HANDLE t = NULL;
    if (OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &t)) {
        TOKEN_ELEVATION v;
        DWORD s = sizeof(v);
        if (GetTokenInformation(t, TokenElevation, &v, sizeof(v), &s))
            e = v.TokenIsElevated;
        CloseHandle(t);
    }
    return e;
}

static BOOL en_priv(LPCWSTR p) {
    HANDLE t;
    TOKEN_PRIVILEGES tp;
    LUID u;
    if (!OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &t))
        return FALSE;
    if (!LookupPrivilegeValueW(NULL, p, &u)) {
        CloseHandle(t);
        return FALSE;
    }
    tp.PrivilegeCount = 1;
    tp.Privileges[0].Luid = u;
    tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
    AdjustTokenPrivileges(t, FALSE, &tp, sizeof(tp), NULL, NULL);
    BOOL r = GetLastError() != ERROR_NOT_ALL_ASSIGNED;
    CloseHandle(t);
    return r;
}

static void flush_vols() {
    WCHAR d[512];
    if (GetLogicalDriveStringsW(512, d)) {
        WCHAR* p = d;
        while (*p) {
            if (GetDriveTypeW(p) == DRIVE_FIXED) {
                WCHAR v[8] = { L'\\', L'\\', L'.', L'\\', L'X', L':', 0 };
                v[4] = p[0];
                HANDLE h = CreateFileW(v, GENERIC_READ | GENERIC_WRITE,
                                       FILE_SHARE_READ | FILE_SHARE_WRITE,
                                       NULL, OPEN_EXISTING, 0, NULL);
                if (h != INVALID_HANDLE_VALUE) {
                    FlushFileBuffers(h);
                    CloseHandle(h);
                }
            }
            p += wcslen(p) + 1;
        }
    }
}

typedef NTSTATUS (WINAPI *NtSetSysInfoPtr)(SYS_INFO_CLASS, PVOID, ULONG);
typedef NTSTATUS (WINAPI *NtQuerySysInfoPtr)(SYS_INFO_CLASS, PVOID, ULONG, PULONG);

static inline NTSTATUS DoNtSetSysInfo(SYS_INFO_CLASS c, PVOID i, ULONG l) {
    static NtSetSysInfoPtr pNtSetSysInfo = NULL;
    if (!pNtSetSysInfo) {
        HMODULE hNtdll = GetModuleHandleW(L"ntdll.dll");
        if (hNtdll)
            pNtSetSysInfo = (NtSetSysInfoPtr)(void*)GetProcAddress(hNtdll, "NtSetSystemInformation");
    }
    if (!pNtSetSysInfo) return ST_NS;
    Sleep(g_aggressive ? 50 : 150);
    return pNtSetSysInfo(c, i, l);
}

static inline NTSTATUS DoNtQuerySysInfo(SYS_INFO_CLASS c, PVOID i, ULONG l, PULONG r) {
    static NtQuerySysInfoPtr pNtQuerySysInfo = NULL;
    if (!pNtQuerySysInfo) {
        HMODULE hNtdll = GetModuleHandleW(L"ntdll.dll");
        if (hNtdll)
            pNtQuerySysInfo = (NtQuerySysInfoPtr)(void*)GetProcAddress(hNtdll, "NtQuerySystemInformation");
    }
    if (!pNtQuerySysInfo) return ST_NS;
    return pNtQuerySysInfo(c, i, l, r);
}

static BOOL is_process_running(const wchar_t* name) {
    DWORD procs[4096], needed;
    if (!EnumProcesses(procs, sizeof(procs), &needed)) return FALSE;
    DWORD count = needed / sizeof(DWORD);
    for (DWORD i = 0; i < count; i++) {
        if (procs[i] == 0) continue;
        HANDLE h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, procs[i]);
        if (h) {
            WCHAR pname[MAX_PATH] = {0};
            DWORD size = MAX_PATH;
            if (QueryFullProcessImageNameW(h, 0, pname, &size)) {
                WCHAR* base = wcsrchr(pname, L'\\');
                if (base) base++;
                else base = pname;
                if (_wcsicmp(base, name) == 0) {
                    CloseHandle(h);
                    return TRUE;
                }
            }
            CloseHandle(h);
        }
    }
    return FALSE;
}

void sa_add_exclusion(const char* name) {
    if (!name || !*name || g_user_excl_count >= 64) return;
    mbstowcs(g_user_exclusions[g_user_excl_count], name, 63);
    g_user_excl_count++;
}

void sa_remove_exclusion(const char* name) {
    if (!name) return;
    wchar_t wname[64];
    mbstowcs(wname, name, 63);
    for (int i = 0; i < g_user_excl_count; i++) {
        if (_wcsicmp(g_user_exclusions[i], wname) == 0) {
            for (int j = i; j < g_user_excl_count - 1; j++)
                wcscpy(g_user_exclusions[j], g_user_exclusions[j + 1]);
            g_user_excl_count--;
            return;
        }
    }
}

int sa_is_excluded(const char* name) {
    if (!name) return 0;
    wchar_t wname[64];
    mbstowcs(wname, name, 63);
    for (size_t i = 0; i < sizeof(g_builtin_exclusions) / sizeof(g_builtin_exclusions[0]); i++)
        if (_wcsicmp(g_builtin_exclusions[i], wname) == 0) return 1;
    for (int i = 0; i < g_user_excl_count; i++)
        if (_wcsicmp(g_user_exclusions[i], wname) == 0) return 1;
    return 0;
}

void sa_get_exclusions(char* buf, int len) {
    if (!buf || len < 2) return;
    buf[0] = 0;
    int pos = 0;
    for (int i = 0; i < g_user_excl_count; i++) {
        char temp[128];
        wcstombs(temp, g_user_exclusions[i], 127);
        int l = (int)strlen(temp);
        if (pos + l + 3 >= len) break;
        if (pos > 0) {
            buf[pos++] = ',';
            buf[pos++] = ' ';
        }
        strcpy(buf + pos, temp);
        pos += l;
    }
    buf[pos] = 0;
}

int sa_check_excluded(void) {
    for (size_t i = 0; i < sizeof(g_builtin_exclusions) / sizeof(g_builtin_exclusions[0]); i++)
        if (is_process_running(g_builtin_exclusions[i])) return 1;
    for (int i = 0; i < g_user_excl_count; i++)
        if (is_process_running(g_user_exclusions[i])) return 1;
    return 0;
}

unsigned long long sa_estimate_clean(unsigned int mask) {
    unsigned long long est = 0;
    if (mask & REDUCT_WORKING_SET)     est += 200ULL * 1024 * 1024;
    if (mask & REDUCT_SYSTEM_CACHE)    est += 500ULL * 1024 * 1024;
    if (mask & REDUCT_STANDBY_LIST)    est += 800ULL * 1024 * 1024;
    if (mask & REDUCT_MODIFIED_LIST)   est += 300ULL * 1024 * 1024;
    if (mask & REDUCT_REGISTRY)        est += 50ULL  * 1024 * 1024;
    if (mask & REDUCT_COMBINE)         est += 150ULL * 1024 * 1024;
    if (mask & REDUCT_LOW_PRIORITY)    est += 100ULL * 1024 * 1024;
    if (mask & REDUCT_CLIPBOARD)       est += 5ULL   * 1024 * 1024;
    if (mask & REDUCT_DNS_CACHE)       est += 2ULL   * 1024 * 1024;
    if (mask & REDUCT_TEMP_FILES)      est += 300ULL * 1024 * 1024;
    if (mask & REDUCT_PREFETCH)        est += 50ULL  * 1024 * 1024;
    if (mask & REDUCT_THUMBNAIL_CACHE) est += 100ULL * 1024 * 1024;
    return est;
}

void sa_get_standby_size(unsigned long long* size) {
    *size = 0;
    SYSTEM_MEMORY_LIST_INFORMATION info;
    memset(&info, 0, sizeof(info));
    ULONG ret = 0;
    if (DoNtQuerySysInfo(SysMemList, &info, sizeof(info), &ret) == ST_OK) {
        SIZE_T standby = 0;
        for (int i = 0; i < 8; i++)
            standby += info.PageCountByPriority[i];
        *size = (unsigned long long)standby * 4096ULL;
    }
}

void sa_get_detailed_mem(unsigned long long* total, unsigned long long* avail,
                         unsigned long long* standby, unsigned long long* modified,
                         unsigned long long* cached) {
    MEMORYSTATUSEX m;
    m.dwLength = sizeof(m);
    if (GlobalMemoryStatusEx(&m)) {
        *total = m.ullTotalPhys;
        *avail = m.ullAvailPhys;
    } else {
        *total = *avail = 0;
    }
    *standby = *modified = *cached = 0;
    SYSTEM_MEMORY_LIST_INFORMATION info;
    memset(&info, 0, sizeof(info));
    ULONG ret = 0;
    if (DoNtQuerySysInfo(SysMemList, &info, sizeof(info), &ret) == ST_OK) {
        SIZE_T st = 0;
        for (int i = 0; i < 8; i++) st += info.PageCountByPriority[i];
        *standby = (unsigned long long)st * 4096ULL;
        *modified = (unsigned long long)info.ModifiedPageCount * 4096ULL;
    }
    PERFORMANCE_INFORMATION pi;
    if (GetPerformanceInfo(&pi, sizeof(pi))) {
        *cached = (unsigned long long)pi.SystemCache * pi.PageSize;
    }
}

int sa_get_process_count(void) {
    DWORD procs[4096], needed;
    if (!EnumProcesses(procs, sizeof(procs), &needed)) return 0;
    return (int)(needed / sizeof(DWORD));
}

void sa_set_aggressive(int enabled) {
    g_aggressive = enabled ? 1 : 0;
}

void sa_empty_clipboard(void) {
    if (OpenClipboard(NULL)) {
        EmptyClipboard();
        CloseClipboard();
    }
}

void sa_flush_dns(void) {
    STARTUPINFOW si;
    PROCESS_INFORMATION pi;
    memset(&si, 0, sizeof(si));
    si.cb = sizeof(si);
    si.dwFlags = STARTF_USESHOWWINDOW;
    si.wShowWindow = SW_HIDE;
    wchar_t cmd[] = L"ipconfig /flushdns";
    if (CreateProcessW(NULL, cmd, NULL, NULL, FALSE, CREATE_NO_WINDOW, NULL, NULL, &si, &pi)) {
        WaitForSingleObject(pi.hProcess, 5000);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    }
}

static void delete_files_in_dir(const wchar_t* path, int recursive) {
    WIN32_FIND_DATAW fd;
    wchar_t search[MAX_PATH];
    _snwprintf(search, MAX_PATH, L"%s\\*", path);
    HANDLE h = FindFirstFileW(search, &fd);
    if (h == INVALID_HANDLE_VALUE) return;
    do {
        if (wcscmp(fd.cFileName, L".") == 0 || wcscmp(fd.cFileName, L"..") == 0) continue;
        wchar_t full[MAX_PATH];
        _snwprintf(full, MAX_PATH, L"%s\\%s", path, fd.cFileName);
        if (fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY) {
            if (recursive) {
                delete_files_in_dir(full, 1);
                RemoveDirectoryW(full);
            }
        } else {
            SetFileAttributesW(full, FILE_ATTRIBUTE_NORMAL);
            DeleteFileW(full);
        }
    } while (FindNextFileW(h, &fd));
    FindClose(h);
}

void sa_clean_temp(void) {
    wchar_t path[MAX_PATH];
    if (GetTempPathW(MAX_PATH, path))
        delete_files_in_dir(path, 1);
    if (SUCCEEDED(SHGetFolderPathW(NULL, CSIDL_LOCAL_APPDATA, NULL, 0, path))) {
        wchar_t temp2[MAX_PATH];
        _snwprintf(temp2, MAX_PATH, L"%s\\Temp", path);
        delete_files_in_dir(temp2, 1);
    }
    
    if (GetWindowsDirectoryW(path, MAX_PATH)) {
        wchar_t wtemp[MAX_PATH];
        _snwprintf(wtemp, MAX_PATH, L"%s\\Temp", path);
        delete_files_in_dir(wtemp, 1);
    }
}

void sa_clean_prefetch(void) {
    wchar_t path[MAX_PATH];
    if (GetWindowsDirectoryW(path, MAX_PATH)) {
        wchar_t pf[MAX_PATH];
        _snwprintf(pf, MAX_PATH, L"%s\\Prefetch", path);
        delete_files_in_dir(pf, 0);
    }
}

void sa_clean_thumbnail(void) {
    wchar_t path[MAX_PATH];
    if (SUCCEEDED(SHGetFolderPathW(NULL, CSIDL_LOCAL_APPDATA, NULL, 0, path))) {
        wchar_t thumb[MAX_PATH];
        _snwprintf(thumb, MAX_PATH, L"%s\\Microsoft\\Windows\\Explorer", path);
        WIN32_FIND_DATAW fd;
        wchar_t search[MAX_PATH];
        _snwprintf(search, MAX_PATH, L"%s\\thumbcache_*.db", thumb);
        HANDLE h = FindFirstFileW(search, &fd);
        if (h != INVALID_HANDLE_VALUE) {
            do {
                wchar_t full[MAX_PATH];
                _snwprintf(full, MAX_PATH, L"%s\\%s", thumb, fd.cFileName);
                SetFileAttributesW(full, FILE_ATTRIBUTE_NORMAL);
                DeleteFileW(full);
            } while (FindNextFileW(h, &fd));
            FindClose(h);
        }
    }
}

static DWORD WINAPI clean_ws_thread(LPVOID) {
    MEM_CMD cmd = EmptyWS;
    if (DoNtSetSysInfo(SysMemList, &cmd, sizeof(cmd)) == ST_OK) InterlockedIncrement((LONG*)&g_ok);
    return 0;
}

static DWORD WINAPI clean_cache_thread(LPVOID) {
    FILECACHE_INFO fi;
    memset(&fi, 0, sizeof(fi));
    fi.mi = 1;
    if (DoNtSetSysInfo(SysFileCache, &fi, sizeof(fi)) == ST_OK) InterlockedIncrement((LONG*)&g_ok);
    
    MEM_CMD cmd = EmptyWS;
    DoNtSetSysInfo(SysMemList, &cmd, sizeof(cmd));
    return 0;
}

static DWORD WINAPI clean_standby_thread(LPVOID) {
    MEM_CMD cmd = PurgeStandby;
    if (DoNtSetSysInfo(SysMemList, &cmd, sizeof(cmd)) == ST_OK) InterlockedIncrement((LONG*)&g_ok);
    if (g_aggressive) {
        cmd = PurgeStandbyPriority;
        DoNtSetSysInfo(SysMemList, &cmd, sizeof(cmd));
    }
    return 0;
}

static DWORD WINAPI clean_modified_thread(LPVOID) {
    MEM_CMD cmd = FlushMod;
    if (DoNtSetSysInfo(SysMemList, &cmd, sizeof(cmd)) == ST_OK) InterlockedIncrement((LONG*)&g_ok);
    return 0;
}

static DWORD WINAPI clean_registry_thread(LPVOID) {
    
    if (DoNtSetSysInfo(SysRegRec, NULL, 0) == ST_OK) InterlockedIncrement((LONG*)&g_ok);
    return 0;
}

static DWORD WINAPI clean_combine_thread(LPVOID) {
    COMBINE_INFO ci;
    memset(&ci, 0, sizeof(ci));
    if (DoNtSetSysInfo(SysCombine, &ci, sizeof(ci)) == ST_OK) InterlockedIncrement((LONG*)&g_ok);
    return 0;
}

static DWORD WINAPI clean_lowprio_thread(LPVOID) {
    MEM_CMD cmd = PurgeLowPrio;
    if (DoNtSetSysInfo(SysMemList, &cmd, sizeof(cmd)) == ST_OK) InterlockedIncrement((LONG*)&g_ok);
    return 0;
}

static DWORD WINAPI clean_clipboard_thread(LPVOID) {
    sa_empty_clipboard();
    InterlockedIncrement((LONG*)&g_ok);
    return 0;
}

static DWORD WINAPI clean_dns_thread(LPVOID) {
    sa_flush_dns();
    InterlockedIncrement((LONG*)&g_ok);
    return 0;
}

static DWORD WINAPI clean_temp_thread(LPVOID) {
    sa_clean_temp();
    InterlockedIncrement((LONG*)&g_ok);
    return 0;
}

static DWORD WINAPI clean_prefetch_thread(LPVOID) {
    sa_clean_prefetch();
    InterlockedIncrement((LONG*)&g_ok);
    return 0;
}

static DWORD WINAPI clean_thumb_thread(LPVOID) {
    sa_clean_thumbnail();
    InterlockedIncrement((LONG*)&g_ok);
    return 0;
}

int sa_clean(unsigned int mask) {
    init_cs();
    EnterCriticalSection(&g_cs);

    g_ok = 0;
    g_try = 0;
    g_err[0] = 0;
    g_last_cleaned_mb = 0;

    if (!is_adm()) {
        snprintf(g_err, sizeof(g_err), "Administrator privileges required.");
        LeaveCriticalSection(&g_cs);
        return -1;
    }

    en_priv(L"SeProfileSingleProcessPrivilege");
    en_priv(L"SeIncreaseQuotaPrivilege");
    en_priv(L"SeDebugPrivilege");

    unsigned long long before_total = 0, before_used = 0, before_free = 0;
    sa_get_mem(&before_total, &before_used, &before_free);

    HANDLE threads[16] = {0};
    int tcount = 0;

    if (mask & REDUCT_WORKING_SET) {
        g_try++;
        threads[tcount++] = CreateThread(NULL, 0, clean_ws_thread, NULL, 0, NULL);
    }
    if (mask & REDUCT_SYSTEM_CACHE) {
        g_try++;
        threads[tcount++] = CreateThread(NULL, 0, clean_cache_thread, NULL, 0, NULL);
    }
    if (mask & REDUCT_STANDBY_LIST) {
        g_try++;
        threads[tcount++] = CreateThread(NULL, 0, clean_standby_thread, NULL, 0, NULL);
    }
    if (mask & REDUCT_MODIFIED_LIST) {
        g_try++;
        threads[tcount++] = CreateThread(NULL, 0, clean_modified_thread, NULL, 0, NULL);
    }
    if (mask & REDUCT_REGISTRY) {
        g_try++;
        threads[tcount++] = CreateThread(NULL, 0, clean_registry_thread, NULL, 0, NULL);
    }
    if (mask & REDUCT_COMBINE) {
        g_try++;
        threads[tcount++] = CreateThread(NULL, 0, clean_combine_thread, NULL, 0, NULL);
    }
    if (mask & REDUCT_LOW_PRIORITY) {
        g_try++;
        threads[tcount++] = CreateThread(NULL, 0, clean_lowprio_thread, NULL, 0, NULL);
    }
    if (mask & REDUCT_CLIPBOARD) {
        g_try++;
        threads[tcount++] = CreateThread(NULL, 0, clean_clipboard_thread, NULL, 0, NULL);
    }
    if (mask & REDUCT_DNS_CACHE) {
        g_try++;
        threads[tcount++] = CreateThread(NULL, 0, clean_dns_thread, NULL, 0, NULL);
    }
    if (mask & REDUCT_TEMP_FILES) {
        g_try++;
        threads[tcount++] = CreateThread(NULL, 0, clean_temp_thread, NULL, 0, NULL);
    }
    if (mask & REDUCT_PREFETCH) {
        g_try++;
        threads[tcount++] = CreateThread(NULL, 0, clean_prefetch_thread, NULL, 0, NULL);
    }
    if (mask & REDUCT_THUMBNAIL_CACHE) {
        g_try++;
        threads[tcount++] = CreateThread(NULL, 0, clean_thumb_thread, NULL, 0, NULL);
    }

    for (int i = 0; i < tcount; i++) {
        if (threads[i]) {
            WaitForSingleObject(threads[i], g_aggressive ? 8000 : 12000);
            CloseHandle(threads[i]);
            Sleep(g_aggressive ? 80 : 200);
        }
    }

    if (mask & REDUCT_FLUSH_VOLUMES) {
        g_try++;
        flush_vols();
        InterlockedIncrement((LONG*)&g_ok);
    }

    
    if (g_aggressive && (mask & REDUCT_STANDBY_LIST)) {
        MEM_CMD cmd = PurgeStandby;
        DoNtSetSysInfo(SysMemList, &cmd, sizeof(cmd));
        cmd = EmptyWS;
        DoNtSetSysInfo(SysMemList, &cmd, sizeof(cmd));
    }

    unsigned long long after_total = 0, after_used = 0, after_free = 0;
    sa_get_mem(&after_total, &after_used, &after_free);

    long long freed = (long long)after_free - (long long)before_free;
    if (freed < 0) freed = 0;
    g_last_cleaned_mb = (unsigned int)(freed / (1024 * 1024));

    if (g_try > 0) {
        snprintf(g_err, sizeof(g_err),
                 "%d/%d operations OK | Freed ~%u MB | Standby purged | Threads done",
                 g_ok, g_try, g_last_cleaned_mb);
    } else {
        snprintf(g_err, sizeof(g_err), "Nothing selected.");
    }

    LeaveCriticalSection(&g_cs);
    return (g_ok < g_try) ? (g_try - g_ok) : 0;
}

void sa_get_err(char* b, unsigned int l) {
    if (!b || !l) return;
    strncpy(b, g_err, l - 1);
    b[l - 1] = 0;
}

void sa_get_mem(unsigned long long* t, unsigned long long* u, unsigned long long* f) {
    MEMORYSTATUSEX m;
    m.dwLength = sizeof(m);
    if (GlobalMemoryStatusEx(&m)) {
        *t = m.ullTotalPhys;
        *f = m.ullAvailPhys;
        *u = m.ullTotalPhys - m.ullAvailPhys;
    } else {
        *t = *u = *f = 0;
    }
}

int sa_is_admin(void) {
    return is_adm() ? 1 : 0;
}

void sa_uac_restart(void) {
    WCHAR path[MAX_PATH];
    if (GetModuleFileNameW(NULL, path, MAX_PATH)) {
        SHELLEXECUTEINFOW sei;
        memset(&sei, 0, sizeof(sei));
        sei.cbSize = sizeof(sei);
        sei.lpVerb = L"runas";
        sei.lpFile = path;
        sei.nShow = SW_SHOW;
        ShellExecuteExW(&sei);
    }
}

void sa_set_priority_low(void) {
    SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS);
    SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
}

static DWORD WINAPI AutoCleanThread(LPVOID) {
    while (g_auto_running) {
        Sleep(g_aggressive ? 3000 : 8000);
        if (!g_auto_threshold || !g_auto_running) continue;
        unsigned long long t, u, f;
        sa_get_mem(&t, &u, &f);
        if (t > 0) {
            unsigned int used_pct = (unsigned int)((u * 100) / t);
            if (used_pct >= g_auto_threshold) {
                sa_clean(REDUCT_ALL_SAFE);
            }
        }
    }
    return 0;
}

void sa_set_auto_threshold(unsigned int percent) {
    init_cs();
    EnterCriticalSection(&g_cs);
    g_auto_threshold = percent > 99 ? 99 : percent;
    if (percent > 0 && !g_auto_running) {
        g_auto_running = TRUE;
        g_auto_thread = CreateThread(NULL, 0, AutoCleanThread, NULL, 0, NULL);
        if (g_auto_thread)
            SetThreadPriority(g_auto_thread, THREAD_PRIORITY_BELOW_NORMAL);
    } else if (percent == 0 && g_auto_running) {
        g_auto_running = FALSE;
        if (g_auto_thread) {
            WaitForSingleObject(g_auto_thread, 4000);
            CloseHandle(g_auto_thread);
            g_auto_thread = NULL;
        }
    }
    LeaveCriticalSection(&g_cs);
}

void sa_stop_auto_clean(void) {
    sa_set_auto_threshold(0);
}

unsigned int sa_get_last_cleaned_mb(void) {
    return g_last_cleaned_mb;
}
