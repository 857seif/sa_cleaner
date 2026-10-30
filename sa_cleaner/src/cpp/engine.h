#ifndef SA_ENGINE_H
#define SA_ENGINE_H

#ifdef __cplusplus
extern "C" {
#endif

#define REDUCT_WORKING_SET      0x01
#define REDUCT_SYSTEM_CACHE     0x02
#define REDUCT_STANDBY_LIST     0x04
#define REDUCT_MODIFIED_LIST    0x08
#define REDUCT_REGISTRY         0x10
#define REDUCT_COMBINE          0x20
#define REDUCT_LOW_PRIORITY     0x40
#define REDUCT_FLUSH_VOLUMES    0x80
#define REDUCT_CLIPBOARD        0x100
#define REDUCT_DNS_CACHE        0x200
#define REDUCT_TEMP_FILES       0x400
#define REDUCT_PREFETCH         0x800
#define REDUCT_THUMBNAIL_CACHE  0x1000
#define REDUCT_ALL_SAFE         0x1FFF

int  sa_clean(unsigned int mask);
void sa_get_err(char* buf, unsigned int len);
void sa_get_mem(unsigned long long* total, unsigned long long* used, unsigned long long* free);
int  sa_is_admin(void);
void sa_uac_restart(void);
void sa_set_priority_low(void);
void sa_set_auto_threshold(unsigned int percent);
void sa_stop_auto_clean(void);
void sa_add_exclusion(const char* name);
void sa_remove_exclusion(const char* name);
int  sa_is_excluded(const char* name);
void sa_get_exclusions(char* buf, int len);
int  sa_check_excluded(void);
unsigned long long sa_estimate_clean(unsigned int mask);
void sa_get_standby_size(unsigned long long* size);
void sa_get_detailed_mem(unsigned long long* total, unsigned long long* avail,
                         unsigned long long* standby, unsigned long long* modified,
                         unsigned long long* cached);
int  sa_get_process_count(void);
void sa_set_aggressive(int enabled);
void sa_empty_clipboard(void);
void sa_flush_dns(void);
void sa_clean_temp(void);
void sa_clean_prefetch(void);
void sa_clean_thumbnail(void);
unsigned int sa_get_last_cleaned_mb(void);

#ifdef __cplusplus
}
#endif

#endif
