use libvfs::VfsInode;
use ramfs::{RamFs, TimeSpec};

struct Provider;
impl ramfs::RamFsProvider for Provider {
    fn current_time() -> ramfs::TimeSpec {
        TimeSpec { sec: 0, nsec: 0 }
    }
}

fn main() {
    env_logger::init();
    let ramfs = RamFs::<Provider>::new();
    let root = ramfs.root();
    assert_eq!(root.write_at(0, &[]), 0);
    let f1 = root.create("f1");
    assert!(f1.is_some());
    let f1 = f1.unwrap();
    let w = f1.write_at(0, "hello world!".as_bytes());
    assert!(w == 12);
    let mut buf = [0; 20];
    let r = f1.read_at(0, &mut buf);
    assert_eq!(r, 12);
    println!("read: {:?}", core::str::from_utf8(&buf[..r]));
}