use godot_psd::psd_lib::{psd::{Psd, ColorMode}, PsdTree};

fn main() {
    let psd = Psd::from_bytes(include_bytes!("../test_import/test.psd")).unwrap();

    assert_eq!(psd.color_mode(), ColorMode::Rgb);

    let tree = PsdTree::new(psd);

    println!("{}", tree.list().join("\n"));

    tree.export_all_to_file();
}

