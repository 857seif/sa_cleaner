#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#define REDUCT_WORKING_SET      0x01
#define REDUCT_SYSTEM_CACHE     0x02
#define REDUCT_STANDBY_PRIO0    0x04
#define REDUCT_STANDBY_LIST     0x08
#define REDUCT_MODIFIED_LIST    0x10
#define REDUCT_COMBINE_MEM      0x20
#define REDUCT_REGISTRY         0x40
#define REDUCT_FLUSH_VOL        0x80
#define REDUCT_ALL_SAFE         0x6F
#define REDUCT_ALL              0xFF

int sa_clean(unsigned int mask);
void sa_get_err(char* buf, unsigned int len);
void sa_get_mem(unsigned long long* total, unsigned long long* used, unsigned long long* free);
int sa_is_admin(void);
void sa_uac_restart(void);
unsigned long long sa_estimate_clean(unsigned int mask);
int sa_check_excluded(void);
void sa_set_auto_threshold(unsigned int percent);
void sa_stop_auto_clean(void);
void sa_set_priority_low(void);
void sa_add_exclusion(const char* name);
void sa_remove_exclusion(const char* name);
int sa_is_excluded(const char* name);
void sa_get_exclusions(char* buf, int len);

#ifdef __cplusplus
}
#endif
