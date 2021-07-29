use crate::ObjectFile;

#[test]
fn replace_atof() {
    fn other_atof(_: *const libc::c_char) -> libc::c_double {
        42.0
    }

    let param = b"100\0".as_ptr().cast();

    let object = ObjectFile::open_main_program().unwrap();

    assert_eq!(unsafe { libc::atof(param) }, 100.0);

    let initial_atof = unsafe { object.replace("atof", other_atof as *const _).unwrap() };

    assert_eq!(unsafe { libc::atof(param) }, 42.0);

    unsafe { object.replace("atof", initial_atof).unwrap() };

    assert_eq!(unsafe { libc::atof(param) }, 100.0);
}
