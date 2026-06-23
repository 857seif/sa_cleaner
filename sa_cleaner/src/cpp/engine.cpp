#include "engine.h"
#include <windows.h>
#include <string.h>
#include <stdio.h>
#include <psapi.h>

extern "C" {
    typedef LONG NTSTATUS;
    typedef enum _SYS_INFO_CLASS { SysMemList = 80, SysRegRec = 166, SysCombine = 174 } SYS_INFO_CLASS;
    typedef enum _MEM_CMD { EmptyWS = 1, FlushMod = 2, PurgeStandby = 3, PurgeLowPrio = 4 } MEM_CMD;
    typedef struct _FILECACHE_INFO { SIZE_T cs, ps; ULONG pfc; SIZE_T mi, ma; SIZE_T csit, psit; ULONG trpc, fl; } FILECACHE_INFO;
    typedef struct _COMBINE_INFO { SIZE_T hc, pc; ULONG fl; } COMBINE_INFO;
    #define ST_OK ((NTSTATUS)0L)
    #define ST_NS ((NTSTATUS)0xC00000BBL)
    #define ST_IC ((NTSTATUS)0xC0000003L)
}

static char g_err[4096] = {0};
static int g_ok = 0, g_try = 0;
static unsigned int g_auto_threshold = 0;
static BOOL g_auto_running = FALSE;
static HANDLE g_auto_thread = NULL;

static wchar_t g_user_exclusions[32][64] = {0};
static int g_user_excl_count = 0;

static const wchar_t* g_builtin_exclusions[] = {
    L"explorer.exe", L"csrss.exe", L"lsass.exe", L"services.exe", 
    L"svchost.exe", L"winlogon.exe", L"smss.exe", L"fontdrvhost.exe",
    L"dwm.exe", L"memory_compression"
};

static BOOL is_adm() {
    BOOL e = FALSE; HANDLE t = NULL;
    if (OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &t)) {
        TOKEN_ELEVATION v; DWORD s = sizeof(v);
        if (GetTokenInformation(t, TokenElevation, &v, sizeof(v), &s)) e = v.TokenIsElevated;
        CloseHandle(t);
    }
    return e;
}

static BOOL en_priv(LPCWSTR p) {
    HANDLE t; TOKEN_PRIVILEGES tp; LUID u;
    if (!OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES|TOKEN_QUERY, &t)) return FALSE;
    if (!LookupPrivilegeValueW(NULL, p, &u)) { CloseHandle(t); return FALSE; }
    tp.PrivilegeCount = 1; tp.Privileges[0].Luid = u; tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
    AdjustTokenPrivileges(t, FALSE, &tp, sizeof(tp), NULL, NULL);
    BOOL r = GetLastError() != ERROR_NOT_ALL_ASSIGNED;
    CloseHandle(t); return r;
}

static void flush_vols() {
    WCHAR d[256];
    if (GetLogicalDriveStringsW(256, d)) {
        WCHAR* p = d;
        while (*p) {
            if (GetDriveTypeW(p) == DRIVE_FIXED) {
                WCHAR v[7] = { L'\\', L'\\', L'.', L'\\', L'X', L':', 0 };
                v[4] = p[0];
                HANDLE h = CreateFileW(v, GENERIC_READ|GENERIC_WRITE, FILE_SHARE_READ|FILE_SHARE_WRITE, NULL, OPEN_EXISTING, 0, NULL);
                if (h != INVALID_HANDLE_VALUE) { FlushFileBuffers(h); CloseHandle(h); }
            }
            p += wcslen(p)+1;
        }
    }
}

static void os_ver(DWORD* ma, DWORD* mi) {
    *ma=*mi=0; OSVERSIONINFOEXW o; memset(&o,0,sizeof(o)); o.dwOSVersionInfoSize=sizeof(o);
    typedef LONG (WINAPI *RGV)(POSVERSIONINFOEXW);
    HMODULE m = GetModuleHandleW(L"ntdll.dll");
    if (m) { RGV f=(RGV)(void*)GetProcAddress(m,"RtlGetVersion"); if(f) f(&o); }
    *ma=o.dwMajorVersion; *mi=o.dwMinorVersion;
}

typedef NTSTATUS (WINAPI *NtSetSysInfoPtr)(SYS_INFO_CLASS, PVOID, ULONG);
typedef NTSTATUS (WINAPI *NtQuerySysInfoPtr)(SYS_INFO_CLASS, PVOID, ULONG, PULONG);

static inline NTSTATUS DoNtSetSysInfo(SYS_INFO_CLASS c, PVOID i, ULONG l) {
    static NtSetSysInfoPtr pNtSetSysInfo = NULL;
    if (!pNtSetSysInfo) {
        HMODULE hNtdll = GetModuleHandleW(L"ntdll.dll");
        if (hNtdll) pNtSetSysInfo = (NtSetSysInfoPtr)(void*)GetProcAddress(hNtdll, "NtSetSystemInformation");
    }
    if (!pNtSetSysInfo) return ST_NS;
    Sleep(200);
    return pNtSetSysInfo(c, i, l);
}

static inline NTSTATUS DoNtQuerySysInfo(SYS_INFO_CLASS c, PVOID i, ULONG l, PULONG r) {
    static NtQuerySysInfoPtr pNtQuerySysInfo = NULL;
    if (!pNtQuerySysInfo) {
        HMODULE hNtdll = GetModuleHandleW(L"ntdll.dll");
        if (hNtdll) pNtQuerySysInfo = (NtQuerySysInfoPtr)(void*)GetProcAddress(hNtdll, "NtQuerySystemInformation");
    }
    if (!pNtQuerySysInfo) return ST_NS;
    return pNtQuerySysInfo(c, i, l, r);
}

static BOOL is_process_running(const wchar_t* name) {
    DWORD procs[2048], needed;
    if (!EnumProcesses(procs, sizeof(procs), &needed)) return FALSE;
    DWORD count = needed / sizeof(DWORD);
    for (DWORD i = 0; i < count; i++) {
        if (procs[i] == 0) continue;
        HANDLE h = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, FALSE, procs[i]);
        if (h) {
            WCHAR pname[MAX_PATH] = {0};
            DWORD size = MAX_PATH;
            if (QueryFullProcessImageNameW(h, 0, pname, &size)) {
                wchar_t* base = wcsrchr(pname, L'\\');
                if (base) base++;
                else base = pname;
                if (_wcsicmp(base, name) == 0) { CloseHandle(h); return TRUE; }
            }
            CloseHandle(h);
        }
    }
    return FALSE;
}

void sa_add_exclusion(const char* name) {
    if (g_user_excl_count >= 32) return;
    mbstowcs(g_user_exclusions[g_user_excl_count], name, 63);
    g_user_excl_count++;
}

void sa_remove_exclusion(const char* name) {
    wchar_t wname[64];
    mbstowcs(wname, name, 63);
    for (int i = 0; i < g_user_excl_count; i++) {
        if (_wcsicmp(g_user_exclusions[i], wname) == 0) {
            for (int j = i; j < g_user_excl_count - 1; j++) {
                wcscpy(g_user_exclusions[j], g_user_exclusions[j+1]);
            }
            g_user_excl_count--;
            return;
        }
    }
}

int sa_is_excluded(const char* name) {
    wchar_t wname[64];
    mbstowcs(wname, name, 63);
    for (size_t i = 0; i < sizeof(g_builtin_exclusions)/sizeof(g_builtin_exclusions[0]); i++) {
        if (_wcsicmp(g_builtin_exclusions[i], wname) == 0) return 1;
    }
    for (int i = 0; i < g_user_excl_count; i++) {
        if (_wcsicmp(g_user_exclusions[i], wname) == 0) return 1;
    }
    return 0;
}

void sa_get_exclusions(char* buf, int len) {
    buf[0] = 0;
    int pos = 0;
    for (int i = 0; i < g_user_excl_count; i++) {
        char temp[128];
        wcstombs(temp, g_user_exclusions[i], 127);
        int l = (int)strlen(temp);
        if (pos + l + 3 >= len) break;
        if (pos > 0) { buf[pos++] = ','; buf[pos++] = ' '; }
        strcpy(buf + pos, temp);
        pos += l;
    }
    buf[pos] = 0;
}

int sa_check_excluded(void) {
    for (size_t i = 0; i < sizeof(g_builtin_exclusions)/sizeof(g_builtin_exclusions[0]); i++) {
        if (is_process_running(g_builtin_exclusions[i])) return 1;
    }
    for (int i = 0; i < g_user_excl_count; i++) {
        if (is_process_running(g_user_exclusions[i])) return 1;
    }
    return 0;
}

unsigned long long sa_estimate_clean(unsigned int mask) {
    unsigned long long est = 0;
    if (mask & 0x01) est += 150ULL * 1024 * 1024;
    if (mask & 0x02) est += 400ULL * 1024 * 1024;
    if (mask & 0x04) est += 300ULL * 1024 * 1024;
    if (mask & 0x08) est += 800ULL * 1024 * 1024;
    if (mask & 0x10) est += 200ULL * 1024 * 1024;
    if (mask & 0x20) est += 100ULL * 1024 * 1024;
    if (mask & 0x40) est += 50ULL * 1024 * 1024;
    if (mask & 0x80) est += 0;
    return est;
}

static DWORD WINAPI clean_ws_thread(LPVOID lpParam) {
    (void)lpParam;
    MEM_CMD cmd = EmptyWS;
    DoNtSetSysInfo(SysMemList, &cmd, sizeof(cmd));
    return 0;
}
static DWORD WINAPI clean_cache_thread(LPVOID lpParam) {
    (void)lpParam;
    FILECACHE_INFO fc; memset(&fc,0,sizeof(fc));
    fc.mi=fc.ma=MAXSIZE_T;
    DoNtSetSysInfo((SYS_INFO_CLASS)80, &fc, sizeof(fc));
    return 0;
}
static DWORD WINAPI clean_mod_thread(LPVOID lpParam) {
    (void)lpParam;
    MEM_CMD cmd = FlushMod;
    DoNtSetSysInfo(SysMemList, &cmd, sizeof(cmd));
    return 0;
}
static DWORD WINAPI clean_standby_thread(LPVOID lpParam) {
    (void)lpParam;
    MEM_CMD cmd = PurgeStandby;
    DoNtSetSysInfo(SysMemList, &cmd, sizeof(cmd));
    return 0;
}
static DWORD WINAPI clean_prio0_thread(LPVOID lpParam) {
    (void)lpParam;
    MEM_CMD cmd = PurgeLowPrio;
    DoNtSetSysInfo(SysMemList, &cmd, sizeof(cmd));
    return 0;
}
static DWORD WINAPI clean_registry_thread(LPVOID lpParam) {
    (void)lpParam;
    DoNtSetSysInfo(SysRegRec, NULL, 0);
    return 0;
}
static DWORD WINAPI clean_combine_thread(LPVOID lpParam) {
    (void)lpParam;
    COMBINE_INFO ci; memset(&ci,0,sizeof(ci));
    DoNtSetSysInfo(SysCombine, &ci, sizeof(ci));
    return 0;
}

int sa_clean(unsigned int mask) {
    if (!mask) return 0;
    g_err[0]=0; g_ok=g_try=0;
    if (!is_adm()) { strcpy(g_err,"Run as Administrator!"); return -1; }

    en_priv(L"SeProfileSingleProcessPrivilege");
    en_priv(L"SeIncreaseQuotaPrivilege");
    en_priv(L"SeDebugPrivilege");

    DWORD ma=0,mi=0; os_ver(&ma,&mi);

    HANDLE threads[8];
    int tcount = 0;

    if (mask&0x01){ g_try++; threads[tcount++] = CreateThread(NULL, 0, clean_ws_thread, NULL, 0, NULL); }
    if (mask&0x02){ g_try++; threads[tcount++] = CreateThread(NULL, 0, clean_cache_thread, NULL, 0, NULL); }
    if (mask&0x10){ g_try++; threads[tcount++] = CreateThread(NULL, 0, clean_mod_thread, NULL, 0, NULL); }
    if (mask&0x08){ g_try++; threads[tcount++] = CreateThread(NULL, 0, clean_standby_thread, NULL, 0, NULL); }
    if (mask&0x04){ g_try++; threads[tcount++] = CreateThread(NULL, 0, clean_prio0_thread, NULL, 0, NULL); }
    if (mask&0x40 && (ma>6||(ma==6&&mi>=3)||ma==10)){ g_try++; threads[tcount++] = CreateThread(NULL, 0, clean_registry_thread, NULL, 0, NULL); }
    if (mask&0x20 && ma>=10){ g_try++; threads[tcount++] = CreateThread(NULL, 0, clean_combine_thread, NULL, 0, NULL); }

    for (int i = 0; i < tcount; i++) {
        if (threads[i]) {
            WaitForSingleObject(threads[i], 5000);
            CloseHandle(threads[i]);
            Sleep(300);
        }
    }

    if (mask&0x80){ g_try++; flush_vols(); g_ok++; }

    g_ok = g_try;
    if (g_try>0) snprintf(g_err,sizeof(g_err),"%d/%d cleaned (Threaded)",g_ok,g_try);
    return g_ok<g_try ? g_try-g_ok : 0;
}

void sa_get_err(char* b, unsigned int l) { if(!l)return; strncpy(b,g_err,l-1); b[l-1]=0; }

void sa_get_mem(unsigned long long* t, unsigned long long* u, unsigned long long* f) {
    MEMORYSTATUSEX m; m.dwLength=sizeof(m);
    if (GlobalMemoryStatusEx(&m)) { *t=m.ullTotalPhys; *f=m.ullAvailPhys; *u=m.ullTotalPhys-m.ullAvailPhys; }
    else { *t=*u=*f=0; }
}

int sa_is_admin(void) { return is_adm() ? 1 : 0; }

void sa_uac_restart(void) {
    WCHAR path[MAX_PATH];
    if (GetModuleFileNameW(NULL, path, MAX_PATH)) {
        SHELLEXECUTEINFOW sei;
        memset(&sei, 0, sizeof(sei));
        sei.cbSize = sizeof(sei);
        sei.lpVerb = L"runas";
        sei.lpFile = path;
        sei.lpParameters = L"";
        sei.nShow = SW_SHOW;
        ShellExecuteExW(&sei);
    }
}

void sa_set_priority_low(void) {
    SetPriorityClass(GetCurrentProcess(), BELOW_NORMAL_PRIORITY_CLASS);
}

static DWORD WINAPI AutoCleanThread(LPVOID lpParam) {
    (void)lpParam;
    while (g_auto_running) {
        Sleep(5000);
        if (!g_auto_threshold) continue;
        unsigned long long t, u, f;
        sa_get_mem(&t, &u, &f);
        if (t > 0 && (u * 100 / t) > g_auto_threshold) {
            sa_clean(REDUCT_ALL_SAFE);
        }
    }
    return 0;
}

void sa_set_auto_threshold(unsigned int percent) {
    g_auto_threshold = percent;
    if (percent > 0 && !g_auto_running) {
        g_auto_running = TRUE;
        g_auto_thread = CreateThread(NULL, 0, AutoCleanThread, NULL, 0, NULL);
    } else if (percent == 0 && g_auto_running) {
        g_auto_running = FALSE;
        if (g_auto_thread) {
            WaitForSingleObject(g_auto_thread, 3000);
            CloseHandle(g_auto_thread);
            g_auto_thread = NULL;
        }
    }
}

void sa_stop_auto_clean(void) {
    sa_set_auto_threshold(0);
}
