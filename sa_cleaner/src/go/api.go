package main
/*
#cgo LDFLAGS: -L. -lsacleaner_cpp
#include "../cpp/engine.h"
*/
import "C"
//export go_get_status
func go_get_status() *C.char {
    var buf [4096]C.char
    C.sa_get_err(&buf[0], 4096)
    return C.CString(C.GoString(&buf[0]))
}
//export go_clean_mask
func go_clean_mask(mask C.uint) C.int {
    return C.sa_clean(mask)
}
//export go_estimate
func go_estimate(mask C.uint) C.ulonglong {
    return C.sa_estimate_clean(mask)
}
//export go_add_exclusion
func go_add_exclusion(name *C.char) {
    C.sa_add_exclusion(name)
}
//export go_remove_exclusion
func go_remove_exclusion(name *C.char) {
    C.sa_remove_exclusion(name)
}
func main() {}
