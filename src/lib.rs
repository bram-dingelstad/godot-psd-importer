pub mod psd;

use std::path::PathBuf;
use std::sync::Arc;

use auto_image_cropper::imagecrop::ImageCrop;
use gdnative::prelude::*;

pub use crate::psd as psd_lib;
use crate::psd::{
    psd::{ColorMode, Psd, PsdError},
    PsdElement, PsdNode as InternalPsdNode, PsdTree,
};

#[derive(NativeClass)]
#[inherit(Object)]
pub struct PsdImporter(Option<PsdTree>);

#[methods]
impl PsdImporter {
    fn new(_base: &Object) -> Self {
        PsdImporter(None)
    }

    #[method]
    fn load(&mut self, bytes: PoolArray<u8>) {
        let bytes = bytes.to_vec();
        let psd = match Psd::from_bytes(&bytes) {
            Ok(psd) => psd,
            Err(error) => match error {
                PsdError::HeaderError(error) => panic!("Failed to parse PSD header: {error:#?}"),
                PsdError::LayerError(error) => panic!("Failed to parse PSD layer: {error:#?}"),
                PsdError::ImageError(error) => {
                    panic!("Failed to parse PSD data section: {error:#?}")
                }
                PsdError::ResourceError(error) => {
                    panic!("Failed to parse PSD resource section: {error:#?}")
                }
                error => panic!("Failed to parse PSD: {error:#?}"),
            },
        };

        match psd.color_mode() {
            ColorMode::Rgb => {
                let tree = PsdTree::new(psd);

                self.0 = Some(tree);
            },
            other_format => godot_error!("[PSD] You tried loading in format {other_format:#?}, but we only support ColorMode::RGB for now :/"),
        }
    }

    #[method]
    fn print_tree(&self) {
        match &self.0 {
            None => godot_error!("[PSD] You tried printing all the layers and groups, but you didn't load a PSD file (succesfully) yet."),
            Some(tree) => {
                godot_print!("{}", tree.list().join("\n"))
            }
        }
    }

    #[method]
    fn get_node(&self, path: String) -> Option<Instance<PsdNode>> {
        match &self.0 {
            None => {
                godot_error!("[PSD] You tried getting a node (layer or group), but you didn't load a PSD file (succesfully) yet.");
                None
            }
            Some(tree) => {
                let path = PathBuf::from(path);
                let mut pieces = path.iter().map(|e| e.to_str().unwrap());

                // Skip the first root
                if path.is_absolute() {
                    pieces.next();
                }
                // TODO: Remove relative path

                let piece = pieces.next()?;

                let mut node = tree
                    .get_children()
                    .into_iter()
                    .find(|child| child.element.name() == piece)?;

                'pieces: for piece in pieces {
                    for child in node.get_children()?.into_iter() {
                        if child.element.name() == piece {
                            node = child;
                            continue 'pieces;
                        }
                    }

                    return None;
                }

                Some(PsdNode::from(node).emplace().into_shared())
            }
        }
    }

    #[method]
    fn get_children(&self) -> Vec<Instance<PsdNode>> {
        match &self.0 {
            None => {
                godot_error!("[PSD] You tried getting children of a node (layer or group), but you didn't load a PSD file (succesfully) yet.");
                vec![]
            }
            Some(tree) => tree
                .get_children()
                .into_iter()
                .map(|internal_node| PsdNode::from(internal_node).emplace().into_shared())
                .collect::<Vec<Instance<PsdNode>>>(),
        }
    }

    #[method]
    fn get_layers(&self) -> Vec<Instance<PsdNode>> {
        match &self.0 {
            None => {
                godot_error!("[PSD] You tried getting all layers at the root of the tree, but you didn't load a PSD file (succesfully) yet.");
                vec![]
            }
            Some(tree) => tree
                .get_children()
                .into_iter()
                .filter_map(|internal_node| match internal_node.element {
                    PsdElement::Layer(_) => {
                        Some(PsdNode::from(internal_node).emplace().into_shared())
                    }
                    _ => None,
                })
                .collect::<Vec<Instance<PsdNode>>>(),
        }
    }

    #[method]
    fn get_groups(&self) -> Vec<Instance<PsdNode>> {
        match &self.0 {
            None => {
                godot_error!("[PSD] You tried getting all groups at the root of the tree, but you didn't load a PSD file (succesfully) yet.");
                vec![]
            }
            Some(tree) => tree
                .get_children()
                .into_iter()
                .filter_map(|internal_node| match internal_node.element {
                    PsdElement::Group(_) => {
                        Some(PsdNode::from(internal_node).emplace().into_shared())
                    }
                    _ => None,
                })
                .collect::<Vec<Instance<PsdNode>>>(),
        }
    }
}

#[derive(NativeClass)]
#[no_constructor]
#[register_with(Self::register_signals)]
pub struct PsdNode {
    internal_node: Arc<InternalPsdNode>,
    thread: Option<std::thread::JoinHandle<(Rect2, Ref<Image, Unique>)>>,

    #[property]
    name: String,
    #[property]
    path: String,
    #[property]
    properties: Option<LayerProperties>,
    #[property]
    node_type: PsdType,
}

#[methods]
impl PsdNode {
    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.signal("image").done();
    }

    #[method]
    fn cleanup_thread(&mut self, #[base] owner: &Reference) {
        let thread = std::mem::replace(&mut self.thread, None);

        if let Some(thread) = thread {
            let (rect, image) = thread.join().unwrap();

            owner.emit_signal("image", &[Variant::new(image), Variant::new(rect)]);
        }
    }

    #[method]
    fn get_node(&self, path: String) -> Option<Instance<PsdNode>> {
        let path = PathBuf::from(path);
        let mut pieces = path.iter().map(|e| e.to_str().unwrap());

        // Skip the first root
        if path.is_absolute() {
            pieces.next();
        }
        // TODO: Remove relative path

        let piece = pieces.next()?;

        let mut node = self
            .internal_node
            .get_children()?
            .into_iter()
            .find(|child| child.element.name() == piece)?;

        'pieces: for piece in pieces {
            for child in node.get_children()?.into_iter() {
                if child.element.name() == piece {
                    node = child;
                    continue 'pieces;
                }
            }

            return None;
        }

        Some(PsdNode::from(node).emplace().into_shared())
    }

    #[method]
    fn get_children(&self) -> Vec<Instance<PsdNode>> {
        match self.internal_node.get_children() {
            None => vec![],
            Some(children) => children
                .into_iter()
                .map(|internal_node| PsdNode::from(internal_node).emplace().into_shared())
                .collect::<Vec<Instance<PsdNode>>>(),
        }
    }

    #[method]
    fn get_layers(&self) -> Vec<Instance<PsdNode>> {
        match self.internal_node.get_children() {
            None => vec![],
            Some(children) => children
                .into_iter()
                .filter_map(|internal_node| match internal_node.element {
                    PsdElement::Layer(_) => {
                        Some(PsdNode::from(internal_node).emplace().into_shared())
                    }
                    _ => None,
                })
                .collect::<Vec<Instance<PsdNode>>>(),
        }
    }

    #[method]
    fn get_groups(&self) -> Vec<Instance<PsdNode>> {
        match self.internal_node.get_children() {
            None => vec![],
            Some(children) => children
                .into_iter()
                .filter_map(|internal_node| match internal_node.element {
                    PsdElement::Group(_) => {
                        Some(PsdNode::from(internal_node).emplace().into_shared())
                    }
                    _ => None,
                })
                .collect::<Vec<Instance<PsdNode>>>(),
        }
    }

    #[method]
    fn get_rect2(&self) -> Rect2 {
        match self.internal_node.element.clone() {
            PsdElement::Layer(layer) => Rect2::new(
                Vector2::new(layer.layer_left() as f32, layer.layer_top() as f32),
                Vector2::new(layer.width().into(), layer.height().into()),
            ),
            PsdElement::Group(group) => Rect2::new(
                Vector2::new(group.layer_left() as f32, group.layer_top() as f32),
                Vector2::new(group.width().into(), group.height().into()),
            ),
        }
    }

    #[method]
    fn get_image(&mut self, #[base] owner: TRef<Reference>, cropped: bool) {
        match &self.internal_node.element {
            PsdElement::Layer(_) => {
                let internal_node = self.internal_node.clone();
                let width = self.internal_node.tree.psd.width();
                let height = self.internal_node.tree.psd.height();

                // Cleanup thread after execution of this method
                unsafe {
                    owner.call_deferred("cleanup_thread", &[]);
                }

                let thread = std::thread::spawn(move || {
                    let image = Image::new();

                    let rect = if let PsdElement::Layer(layer) = &internal_node.element {
                        match std::panic::catch_unwind(|| {
                            let mut rect = Rect2::new(Vector2::ZERO, Vector2::ZERO);

                            if cropped {
                                let (top_left, _, width, height, bytes) =
                                    ImageCrop::from_buffer(width, height, layer.rgba())
                                        .unwrap()
                                        .auto_crop();

                                rect.size = Vector2::new(width as f32, height as f32);
                                rect.position = Vector2::new(top_left.x as f32, top_left.y as f32);

                                // Pre-multiply layer alpha
                                let opacity = layer.opacity();
                                let mut bytes = bytes.into_rgba8().into_raw();
                                let mut index = 0;
                                for byte in bytes.iter_mut() {
                                    if index % 4 == 3 {
                                        if byte > &mut 0u8 {
                                            *byte = (*byte as f32 * opacity as f32 / 255.0).round()
                                                as u8
                                        }
                                    }
                                    index += 1;
                                }

                                let data = PoolArray::from_vec(bytes);
                                image.create_from_data(
                                    width.into(),
                                    height.into(),
                                    false,
                                    Image::FORMAT_RGBA8,
                                    data,
                                );

                                // image.premultiply_alpha();

                                rect
                            } else {
                                let data = PoolArray::from_vec(layer.rgba());
                                rect.size = Vector2::new(width as f32, height as f32);

                                image.create_from_data(
                                    width.into(),
                                    height.into(),
                                    false,
                                    Image::FORMAT_RGBA8,
                                    data,
                                );

                                rect
                            }
                        }) {
                            Ok(result) => result,
                            _ => Rect2::new(Vector2::ZERO, Vector2::ZERO),
                        }
                    } else {
                        Rect2::new(Vector2::ZERO, Vector2::ZERO)
                    };

                    (rect, image)
                });

                self.thread = Some(thread);
            }
            _ => {}
        }
    }

    #[method]
    fn _to_string(&self) -> String {
        match &self.node_type {
            PsdType::Layer => format!("Layer[{}]", self.internal_node.get_path().to_str().unwrap()),
            PsdType::Group => format!("Group[{}]", self.internal_node.get_path().to_str().unwrap()),
        }
    }
}

impl From<InternalPsdNode> for PsdNode {
    fn from(internal_node: InternalPsdNode) -> PsdNode {
        PsdNode {
            name: internal_node.element.name(),
            path: internal_node.get_path().to_str().unwrap().to_string(),
            node_type: match internal_node.element {
                PsdElement::Group(_) => PsdType::Group,
                PsdElement::Layer(_) => PsdType::Layer,
            },
            properties: match &internal_node.element {
                PsdElement::Layer(layer) => Some(LayerProperties {
                    visible: layer.visible(),
                    opacity: layer.opacity(),
                    width: u32::try_from(layer.width()).unwrap(),
                    height: u32::try_from(layer.height()).unwrap(),
                    group_id: layer.parent_id(),
                }),
                _ => None,
            },

            internal_node: Arc::from(internal_node),
            thread: None,
        }
    }
}

impl From<Arc<InternalPsdNode>> for PsdNode {
    fn from(internal_node: Arc<InternalPsdNode>) -> PsdNode {
        PsdNode {
            name: internal_node.element.name(),
            path: internal_node.get_path().to_str().unwrap().to_string(),
            node_type: match internal_node.element {
                PsdElement::Group(_) => PsdType::Group,
                PsdElement::Layer(_) => PsdType::Layer,
            },
            properties: match &internal_node.element {
                PsdElement::Layer(layer) => Some(LayerProperties {
                    visible: layer.visible(),
                    opacity: layer.opacity(),
                    width: u32::try_from(layer.width()).unwrap(),
                    height: u32::try_from(layer.height()).unwrap(),
                    group_id: layer.parent_id(),
                }),
                _ => None,
            },

            internal_node,
            thread: None,
        }
    }
}

#[derive(FromVariant, ToVariant)]
#[variant(enum = "str")]
pub enum PsdType {
    Group,
    Layer,
}

impl gdnative::export::Export for PsdType {
    type Hint = ();

    fn export_info(_hint: Option<Self::Hint>) -> ExportInfo {
        ExportInfo::new(VariantType::GodotString)
    }
}

#[derive(FromVariant, ToVariant, Clone)]
pub struct LayerProperties {
    pub visible: bool,
    pub opacity: u8,
    pub width: u32,
    pub height: u32,
    pub group_id: Option<u32>,
}

impl gdnative::export::Export for LayerProperties {
    type Hint = ();

    fn export_info(_hint: Option<Self::Hint>) -> ExportInfo {
        ExportInfo::new(VariantType::Dictionary)
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<PsdImporter>();
    handle.add_class::<PsdNode>();
}

godot_init!(init);
