extern crate hedgehog;

use hedgehog::gen;
use hedgehog::range;

fn main() {
    gen::print_sample(gen::vec(range::constant(1, 10))(gen::u8(range::constant(
        5, 15,
    ))));
}
