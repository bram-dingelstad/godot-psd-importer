use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::BufWriter;
use std::rc::Rc;

use psd::{ColorMode, Psd, PsdGroup, PsdLayer};

struct PsdTree {
    pub psd: Psd
}

impl PsdTree {
    fn new(psd: Psd) -> Self {
        PsdTree { psd }
    }

    fn children(&self) -> Vec<PsdNode> {
        let tree = Rc::from(self);
        let groups = tree.psd 
            .group_ids_in_order()
            .iter()
            .filter_map(
                |id| {
                    let group = tree.psd.groups().get(id).unwrap();

                    match group.parent_id() {
                        Some(_) => None,
                        None => {
                            let element = PsdElement::Group(group.to_owned());

                            Some(PsdNode::new(element, tree.clone(), 0))
                        }
                    }
                }
            )
            .collect::<Vec<PsdNode>>();

        let layers = tree.psd
            .layers()
            .iter()
            .filter_map(
                |layer| {
                    match layer.parent_id() {
                        Some(_) => None,
                        None => {
                            let element = PsdElement::Layer(layer.to_owned());
                            
                            Some(PsdNode::new(element, tree.clone(), 0))
                        }
                    }
                }
            )
            .collect::<Vec<PsdNode>>();

        // TODO: Add IDs to layers to determine order in root, assume bottom for now
        [groups, layers].concat()
    }

    pub fn print(&self) {
        for node in &self.children() {
            node.print()
        }
    }

    pub fn export_all(self) {
        for node in &self.children() {
            if let PsdElement::Layer(_) = &node.element {
                node.export();
                break
            } else {
                node.export_all();
            }
        }
    }
}

#[derive(Clone)]
enum PsdElement {
    Group(PsdGroup),
    Layer(PsdLayer)
}

impl PsdElement {
    fn name(&self) -> String {
        match &self {
            PsdElement::Group(group) => group.name().to_string(),
            PsdElement::Layer(layer) => layer.name().to_string()
        }
    }
}

#[derive(Clone)]
struct PsdNode<'a> {
    pub tree: Rc<&'a PsdTree>,
    pub element: PsdElement,
    pub children: Option<Vec<PsdNode<'a>>>,
    pub depth: usize
}

impl PsdNode<'_> {
    fn new(element: PsdElement, tree: Rc<&PsdTree>, depth: usize) -> PsdNode {
        let children = if let PsdElement::Group(group) = &element {
            let groups = tree.psd
                .groups()
                .iter()
                .filter_map(
                    |(id, sub_group)| {
                        if let Some(parent_id) = sub_group.parent_id() {
                            if group.id() == parent_id {
                                Some(
                                    PsdNode::new(
                                        PsdElement::Group(tree.psd.groups().get(id).unwrap().to_owned()), 
                                        tree.clone(),
                                        depth + 1
                                    )
                                )
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                )
                .collect::<Vec<PsdNode>>();

                let layers = tree.psd
                    .layers()
                    .iter()
                    .filter_map(
                        |layer| {
                            if let Some(parent_id) = layer.parent_id() {
                                if group.id() == parent_id {
                                    Some(
                                        PsdNode::new(
                                            PsdElement::Layer(layer.to_owned()),
                                            tree.clone(),
                                            depth + 1
                                        )
                                    )
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }
                    )
                    .collect::<Vec<PsdNode>>();

            Some([groups, layers].concat())
        } else {
            None
        };

        PsdNode {
            tree,
            element,
            children,
            depth
        }
    }

    // TODO: Clean this up with a decent implementation
    fn get_path(&self) -> PathBuf {
        let mut parts: Vec<String> = vec![self.element.name()];

        let parent_id = match &self.element {
            PsdElement::Layer(layer) => layer.parent_id(),
            PsdElement::Group(group) => group.parent_id()
        };

        let mut cursor = match parent_id {
            Some(parent_id) => self.tree.psd.groups().get(&parent_id).unwrap(),
            None => return PathBuf::from(format!("/{}", self.element.name()))
        };

        while let Some(parent_id) = cursor.parent_id() {
            parts.push(cursor.name().to_string());
            cursor = self.tree.psd.groups().get(&parent_id).unwrap();
        }

        parts.push(cursor.name().to_string());

        parts.reverse();
        PathBuf::from(format!("/{}", parts.join("/")))
    }

    fn export(&self) {
        if let PsdElement::Layer(layer) = &self.element {
            let path = PathBuf::from(
                format!(
                    "./psd-output{}.png", 
                    self.get_path().to_str().unwrap()
                )
            );

            println!("Exporting to {}", path.to_str().unwrap());

            write_to_png(path.as_path(), (self.tree.psd.width(), self.tree.psd.height()), layer.rgba());
        }
    }

    fn export_all(&self) {
        if let PsdElement::Group(_) = &self.element {
            if let Some(children) = &self.children {
                for child in children {
                    match &child.element {
                        PsdElement::Group(_) => child.export_all(),
                        PsdElement::Layer(_) => child.export()
                    }
                }
            }
        }

    }

    pub fn print(&self) {
        match &self.element {
            PsdElement::Group(group) => {
                if let Some(children) = &self.children {
                    println!("{}[G] {}", "\t".repeat(self.depth), group.name());

                    for node in children {
                        node.print();
                    }
                }
            },
            PsdElement::Layer(layer) => {
                println!("{}[L] {}", "\t".repeat(self.depth), layer.name());
            }
        }
    }
}


fn main() {
    let psd = include_bytes!("../test.psd");
    // TODO: find a way to get bytes from Godot's resource format?
    let psd = Psd::from_bytes(psd).unwrap();

    // TODO: Find a way to strictly require this in the Godot editor.
    assert_eq!(psd.color_mode(), ColorMode::Rgb);

    let tree = PsdTree::new(psd);

    tree.print();
    tree.export_all();
}


fn write_to_png(path: &Path, size: (u32, u32), bytes: Vec<u8>) {
    std::fs::DirBuilder::new()
                .recursive(true)
                .create(path.parent().unwrap()).unwrap();

    let file = File::create(path).unwrap();

    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, size.0, size.1);

    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));     // 1.0 / 2.2, unscaled, but rounded
    let source_chromaticities = png::SourceChromaticities::new(     // Using unscaled instantiation here
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000)
    );
    encoder.set_source_chromaticities(source_chromaticities);

    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&bytes).unwrap();
}
