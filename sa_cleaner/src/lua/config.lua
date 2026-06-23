-- SA Cleaner Configuration - By Seif Afandi
config = {
    safe = {
        working_set = true,
        system_cache = true,
        standby_prio0 = true,
        registry_cache = true,
        combine_memory = true,
        flush_volumes = true
    },
    system = {
        standby_list = false,
        modified_list = false
    },
    exclusions = {
        "explorer.exe", "csrss.exe", "lsass.exe", "services.exe",
        "svchost.exe", "winlogon.exe", "dwm.exe", "smss.exe"
    },
    user_exclusions = {},
    ui = {
        theme = "light",
        language = "en",
        show_warnings = true,
        auto_clean_threshold = 60,
        by_seif_afandi = true,
        resizable = true,
        dark_mode = false
    },
    advanced = {
        enable_privileges = true,
        log_operations = false,
        minimize_to_tray = true,
        threaded_clean = true,
        delay_between_ops = 200,
        low_priority_mode = true
    }
}

function get_clean_mask()
    local mask = 0
    if config.safe.working_set then mask = mask | 0x01 end
    if config.safe.system_cache then mask = mask | 0x02 end
    if config.safe.standby_prio0 then mask = mask | 0x04 end
    if config.system.standby_list then mask = mask | 0x08 end
    if config.system.modified_list then mask = mask | 0x10 end
    if config.safe.combine_memory then mask = mask | 0x20 end
    if config.safe.registry_cache then mask = mask | 0x40 end
    if config.safe.flush_volumes then mask = mask | 0x80 end
    return mask
end

return config
