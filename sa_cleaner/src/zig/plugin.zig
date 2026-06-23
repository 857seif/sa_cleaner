const std = @import("std");
export fn zig_scan_regions() u32 {
    var mask: u32 = 0;
    mask |= 0x01; mask |= 0x02; mask |= 0x04;
    return mask;
}
export fn zig_get_version() [*]const u8 {
    return "Zig Plugin - By Seif Afandi";
}
