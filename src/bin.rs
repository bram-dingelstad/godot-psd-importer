use godot_psd::psd_lib::{psd::{Psd, ColorMode}, PsdTree};

fn main() {
    // let psd = include_bytes!("../test.psd");
    // // TODO: find a way to get bytes from Godot's resource format?
    // let psd = Psd::from_bytes(psd).unwrap();
    //
    // // TODO: Find a way to strictly require this in the Godot editor.
    // assert_eq!(psd.color_mode(), ColorMode::Rgb);
    //
    // let tree = PsdTree::new(psd);
    //
    // println!("{}", tree.list().join("\n"));
    //
    // tree.export_all_to_file();
}

