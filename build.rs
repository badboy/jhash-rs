extern crate gcc;

fn main() {
    gcc::compile_library("libjhash.a", &["src/jhash.c"]);
}
