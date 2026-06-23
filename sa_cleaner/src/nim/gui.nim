{.compile: "../cpp/engine.cpp"}.
{.passL: "-ladvapi32 -lkernel32 -lgdi32 -luser32 -lshell32 -lpsapi"}.
proc sa_clean(mask: cuint): cint {.importc.}
proc sa_get_err(buf: ptr char, len: cuint) {.importc.}
proc sa_get_mem(total, used, free: ptr culonglong) {.importc.}
proc sa_add_exclusion(name: cstring) {.importc.}
proc sa_remove_exclusion(name: cstring) {.importc.}
proc sa_get_exclusions(buf: cstring, len: cint) {.importc.}
proc nim_format_mem*(bytes: culonglong): string =
  let units = ["B", "KB", "MB", "GB", "TB"]
  var size = float(bytes)
  var i = 0
  while size >= 1024.0 and i < units.len - 1:
    size /= 1024.0
    i += 1
  return fmt"{size:.1f} {units[i]}"
proc nim_get_mem_json*(): string =
  var total, used, free: culonglong
  sa_get_mem(addr total, addr used, addr free)
  return fmt"{{"total":{total},"used":{used},"free":{free}}}"
when isMainModule:
  echo "Nim GUI Plugin - By Seif Afandi loaded"
