use std::{env, fs, path::Path};
fn main() {
    fn walk(p: &Path, out: &mut Vec<String>) {
        for e in fs::read_dir(p).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                walk(&p, out)
            } else {
                out.push(p.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut files = vec![];
    walk(Path::new("assets"), &mut files);
    files.sort();
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut s = String::from("pub fn asset(name:&str)-> &'static [u8] {match name {\n");
    for f in files {
        s += &format!(
            "{:?} => include_bytes!({:?}),\n",
            &f[7..],
            format!("{root}/{f}")
        );
    }
    s += "_ => panic!(\"missing embedded asset: {}\",name)}}\n";
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("assets.rs"),
        s,
    )
    .unwrap();
    println!("cargo:rerun-if-changed=assets");
}
