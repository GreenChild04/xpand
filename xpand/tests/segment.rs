use std::{fs::File, io::{Write, BufWriter}};
use xpand::segment::segment;

#[test]
fn segment_file() {
    let mut file = BufWriter::new(File::create("test_tmp/segment/sourcefile").unwrap());
    (0..(1024 * 1024 * 26)).into_iter().for_each(|_| file.write_all(&[b'd']).unwrap());
    file.flush().unwrap();
    std::mem::drop(file);

    segment("test_tmp/segment/sourcefile", "test_tmp/segment").unwrap();
}