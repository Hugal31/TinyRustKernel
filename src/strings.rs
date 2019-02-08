use core::slice;

pub unsafe fn cstr_to_str_unchecked<'a>(c: *const u8) -> &'a str {
    let len = strlen(c);
    let s = slice::from_raw_parts(c, len);
    ::core::str::from_utf8_unchecked(s)
}

unsafe fn strlen(c: *const u8) -> usize {
    let mut len: usize = 0;
    while *c.add(len) != 0 {
        len += 1;
    }

    len
}
