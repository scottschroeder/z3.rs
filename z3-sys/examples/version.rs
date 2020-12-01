//! Print the version of z3 that we are linked to
extern crate z3_sys;

fn main() {
    let mut major: std::os::raw::c_uint = 0;
    let mut minor: std::os::raw::c_uint = 0;
    let mut build: std::os::raw::c_uint = 0;
    let mut revision: std::os::raw::c_uint = 0;

    unsafe {
        z3_sys::Z3_get_version(&mut major, &mut minor, &mut build, &mut revision);
    }

    println!("{}.{}.{}-{}", major, minor, build, revision);
}
