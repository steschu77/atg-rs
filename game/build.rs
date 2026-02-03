extern crate embed_resource;

fn main() {
    let _ = embed_resource::compile("game.rc", embed_resource::NONE);
    static_vcruntime::metabuild();
}
