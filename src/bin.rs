use godot_psd::psd_lib::{
    psd::{ColorMode, Psd},
    PsdTree,
};
use std::path::PathBuf;

fn main() {
    let psd = Psd::from_bytes(include_bytes!("../test_import/test.psd")).unwrap();

    assert_eq!(psd.color_mode(), ColorMode::Rgb);

    let tree = PsdTree::new(psd);

    // println!("{}", tree.list().join("\n"));

    // let layer = tree.get_path("/Face Shadows/Masculine")

    let path = PathBuf::from("/Face Shadows/Masculine");
    let mut pieces = path.iter().map(|e| e.to_str().unwrap());

    // Skip the first root
    if path.is_absolute() {
        pieces.next();
    }
    // TODO: Remove relative path

    let piece = pieces.next().unwrap();

    let mut node = tree
        .get_children()
        .into_iter()
        .find(|child| child.element.name() == piece)
        .unwrap();

    'pieces: for piece in pieces {
        for child in node.get_children().unwrap().into_iter() {
            if child.element.name() == piece {
                node = child;
                continue 'pieces;
            }
        }
    }

    node.export_to_file();

    // tree.export_all_to_file();
}
