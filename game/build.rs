extern crate embed_resource;

fn main() {
    embed_resource::compile("game.rc", embed_resource::NONE);
    static_vcruntime::metabuild();
}
