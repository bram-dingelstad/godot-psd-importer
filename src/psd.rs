use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use auto_image_cropper::imagecrop::ImageCrop;
use psd::{Psd, PsdGroup, PsdLayer};

pub use psd;

#[derive(Debug, Clone)]
pub struct PsdTree {
    pub psd: Arc<Psd>,
}

impl PsdTree {
    pub fn new(psd: Psd) -> Self {
        PsdTree {
            psd: Arc::from(psd),
        }
    }

    pub fn get_children(&self) -> Vec<PsdNode> {
        let tree = Arc::from(self.clone());
        let groups = tree
            .psd
            .group_ids_in_order()
            .iter()
            .filter_map(|id| {
                let group = tree.psd.groups().get(id).unwrap();

                match group.parent_id() {
                    Some(_) => None,
                    None => {
                        let element = PsdElement::Group(group.to_owned());

                        Some(PsdNode::new(element, tree.clone(), 0))
                    }
                }
            })
            .collect::<Vec<PsdNode>>();

        let layers = tree
            .psd
            .layers()
            .iter()
            .filter_map(|layer| match layer.parent_id() {
                Some(_) => None,
                None => {
                    let element = PsdElement::Layer(layer.to_owned());

                    Some(PsdNode::new(element, tree.clone(), 0))
                }
            })
            .collect::<Vec<PsdNode>>();

        // TODO: Add IDs to layers to determine order in root, assume bottom for now
        [groups, layers].concat()
    }

    pub fn list(&self) -> Vec<String> {
        let mut strings = vec![];
        for node in &self.get_children() {
            strings.append(&mut node.list())
        }

        strings
    }

    pub fn export_all_to_file(self) {
        for node in &self.get_children() {
            if let PsdElement::Layer(_) = &node.element {
                node.export_to_file();
            } else {
                node.export_all_to_file();
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum PsdElement {
    Group(PsdGroup),
    Layer(PsdLayer),
}

impl PsdElement {
    pub fn name(&self) -> String {
        match &self {
            PsdElement::Group(group) => group.name().to_string(),
            PsdElement::Layer(layer) => layer.name().to_string(),
        }
        .trim_matches(char::from(0))
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub struct PsdNode {
    pub tree: Arc<PsdTree>,
    pub element: PsdElement,
    pub depth: usize,
}

impl PsdNode {
    fn new(element: PsdElement, tree: Arc<PsdTree>, depth: usize) -> PsdNode {
        PsdNode {
            tree,
            element,
            depth,
        }
    }

    pub fn get_children(&self) -> Option<Vec<PsdNode>> {
        if let PsdElement::Group(group) = &self.element {
            let groups = self
                .tree
                .psd
                .groups()
                .iter()
                .filter_map(|(id, sub_group)| {
                    if let Some(parent_id) = sub_group.parent_id() {
                        if group.id() == parent_id {
                            Some(PsdNode::new(
                                PsdElement::Group(
                                    self.tree.psd.groups().get(id).unwrap().to_owned(),
                                ),
                                self.tree.clone(),
                                &self.depth + 1,
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<PsdNode>>();

            let layers = self
                .tree
                .psd
                .layers()
                .iter()
                .filter_map(|layer| {
                    if let Some(parent_id) = layer.parent_id() {
                        if group.id() == parent_id {
                            Some(PsdNode::new(
                                PsdElement::Layer(layer.to_owned()),
                                self.tree.clone(),
                                &self.depth + 1,
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect::<Vec<PsdNode>>();

            Some([groups, layers].concat())
        } else {
            None
        }
    }

    // TODO: Clean this up with a decent implementation
    // TODO: Strip layer names to not have whitespace in names
    pub fn get_path(&self) -> PathBuf {
        let mut parts: Vec<String> = vec![self.element.name()];

        let parent_id = match &self.element {
            PsdElement::Layer(layer) => layer.parent_id(),
            PsdElement::Group(group) => group.parent_id(),
        };

        let mut cursor = match parent_id {
            Some(parent_id) => self.tree.psd.groups().get(&parent_id).unwrap(),
            None => return PathBuf::from(format!("/{}", self.element.name())),
        };

        while let Some(parent_id) = cursor.parent_id() {
            parts.push(cursor.name().trim_matches(char::from(0)).to_string());
            cursor = self.tree.psd.groups().get(&parent_id).unwrap();
        }

        parts.push(cursor.name().trim_matches(char::from(0)).to_string());

        parts.reverse();

        PathBuf::from(
            format!("/{}", parts.join("/"))
                .trim_matches(char::from(0))
                .to_string(),
        )
    }

    pub fn export_to_file(&self) {
        if let PsdElement::Layer(layer) = &self.element {
            let path = PathBuf::from(format!(
                "./psd-output{}.png",
                self.get_path().to_str().unwrap()
            ));

            println!("Exporting to {}", path.to_str().unwrap());

            let buffer = match std::panic::catch_unwind(|| {
                let mut image = ImageCrop::from_buffer(
                    self.tree.psd.width(),
                    self.tree.psd.height(),
                    layer.rgba(),
                )
                .unwrap();

                let (top_left, bottom_right, width, height, crop) = image.auto_crop();

                let opacity = layer.opacity();
                let mut bytes = crop.into_rgba8().into_raw();
                for byte in bytes.iter_mut().step_by(3) {
                    *byte *= opacity / 255;
                }

                (top_left, bottom_right, width, height, bytes)
            }) {
                Ok(buffer) => buffer,
                Err(error) => {
                    println!("Something happened oh noes! {error:#?}");
                    return;
                }
            };

            let (_, _, width, height, buffer) = buffer;

            write_to_png(path.as_path(), (width, height), buffer);
            println!("Done exporting {}", path.to_str().unwrap());
        }
    }

    fn export_all_to_file(&self) {
        if let PsdElement::Group(_) = &self.element {
            if let Some(children) = self.get_children() {
                for child in children {
                    match &child.element {
                        PsdElement::Group(_) => child.export_all_to_file(),
                        PsdElement::Layer(_) => child.export_to_file(),
                    }
                }
            }
        }
    }

    pub fn list(&self) -> Vec<String> {
        let mut strings = vec![];
        match &self.element {
            PsdElement::Group(group) => {
                if let Some(children) = self.get_children() {
                    let name = group.name().trim_matches(char::from(0));
                    strings.push(format!("{}[G] {}", "\t".repeat(self.depth), name));

                    for node in children {
                        strings.append(&mut node.list());
                    }
                }
            }
            PsdElement::Layer(layer) => {
                let name = layer.name().trim_matches(char::from(0));
                strings.push(format!("{}[L] {}", "\t".repeat(self.depth), name));
            }
        }

        strings
    }
}

fn write_to_png(path: &Path, size: (u32, u32), bytes: Vec<u8>) {
    std::fs::DirBuilder::new()
        .recursive(true)
        .create(path.parent().unwrap())
        .unwrap();

    let file = File::create(path).unwrap();

    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, size.0, size.1);

    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
    encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2)); // 1.0 / 2.2, unscaled, but rounded
    let source_chromaticities = png::SourceChromaticities::new(
        // Using unscaled instantiation here
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000),
    );
    encoder.set_source_chromaticities(source_chromaticities);

    let mut writer = encoder.write_header().unwrap();

    writer.write_image_data(&bytes).unwrap();
}
