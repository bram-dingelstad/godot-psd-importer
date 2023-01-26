pub mod psd;

use std::sync::Arc;
use std::path::PathBuf;

use gdnative::prelude::*;
use crate::psd::{PsdTree, PsdNode as InternalPsdNode, PsdElement, psd::{Psd, ColorMode}};

pub use crate::psd as psd_lib;

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
        let psd = Psd::from_bytes(&bytes).unwrap();

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
            },
            Some(tree) => {
                let path = PathBuf::from(path);
                let mut pieces = path.iter().map(|e| e.to_str().unwrap());

                // Skip the first root 
                if path.is_absolute() {
                    pieces.next();
                }
                // TODO: Remove relative path

                let piece = pieces.next()?;

                let mut node = tree.get_children()
                    .into_iter()
                    .find(|child| child.element.name() == piece)?;

                'pieces: for piece in pieces {
                    for child in node.get_children()?.into_iter() {
                        if child.element.name() == piece {
                            node = child;
                            continue 'pieces
                        }
                    }

                    return None
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
            },
            Some(tree) => {
                tree.get_children()
                    .into_iter()
                    .map(|internal_node| PsdNode::from(internal_node).emplace().into_shared())
                    .collect::<Vec<Instance<PsdNode>>>()
            }
        }
    }

    #[method]
    fn get_layers(&self) -> Vec<Instance<PsdNode>> {
        match &self.0 {
            None => {
                godot_error!("[PSD] You tried getting all layers at the root of the tree, but you didn't load a PSD file (succesfully) yet.");
                vec![]
            },
            Some(tree) => {
                tree.get_children()
                    .into_iter()
                    .filter_map(
                        |internal_node| {
                            match internal_node.element {
                                PsdElement::Layer(_) => Some(PsdNode::from(internal_node).emplace().into_shared()),
                                _ => None
                            }
                        }
                    )
                    .collect::<Vec<Instance<PsdNode>>>()
            }
        }
    }

    #[method]
    fn get_groups(&self) -> Vec<Instance<PsdNode>> {
        match &self.0 {
            None => {
                godot_error!("[PSD] You tried getting all groups at the root of the tree, but you didn't load a PSD file (succesfully) yet.");
                vec![]
            },
            Some(tree) => {
                tree.get_children()
                    .into_iter()
                    .filter_map(
                        |internal_node| {
                            match internal_node.element {
                                PsdElement::Group(_) => Some(PsdNode::from(internal_node).emplace().into_shared()),
                                _ => None
                            }
                        }
                    )
                    .collect::<Vec<Instance<PsdNode>>>()
            }
        }
    }
}

#[derive(NativeClass)]
#[no_constructor]
#[register_with(Self::register_signals)]
pub struct PsdNode {
    internal_node: Arc<InternalPsdNode>,
    thread: Option<std::thread::JoinHandle<Ref<Image, Unique>>>,

    #[property]
    name: String,
    #[property]
    path: String,
    #[property]
    properties: Option<LayerProperties>,
    #[property]
    node_type: PsdType
}

#[methods]
impl PsdNode {
    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.signal("image")
            .done();
    }

    #[method]
    fn cleanup_thread(&mut self, #[base] owner: &Reference) {
        let thread = std::mem::replace(&mut self.thread, None);

        if let Some(thread) = thread {
            let image = thread.join().unwrap();

            owner.emit_signal("image", &[Variant::new(image)]);
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

        let mut node = self.internal_node
            .get_children()?
            .into_iter()
            .find(|child| child.element.name() == piece)?;

        'pieces: for piece in pieces {
            for child in node.get_children()?.into_iter() {
                if child.element.name() == piece {
                    node = child;
                    continue 'pieces
                }
            }

            return None
        }

        Some(PsdNode::from(node).emplace().into_shared())
    }

    #[method]
    fn get_children(&self) -> Vec<Instance<PsdNode>> {
        match self.internal_node.get_children() {
            None => vec![],
            Some(children) => {
                children
                    .into_iter()
                    .map(|internal_node| PsdNode::from(internal_node).emplace().into_shared())
                    .collect::<Vec<Instance<PsdNode>>>()
            }
        }
    }

    #[method]
    fn get_layers(&self) -> Vec<Instance<PsdNode>> {
        match self.internal_node.get_children() {
            None => vec![],
            Some(children) => {
                children
                    .into_iter()
                    .filter_map(
                        |internal_node| {
                            match internal_node.element {
                                PsdElement::Layer(_) => Some(PsdNode::from(internal_node).emplace().into_shared()),
                                _ => None
                            }
                        }
                    )
                   .collect::<Vec<Instance<PsdNode>>>()
            }
        }
    }

    #[method]
    fn get_groups(&self) -> Vec<Instance<PsdNode>> {
        match self.internal_node.get_children() {
            None => vec![],
            Some(children) => {
                children
                    .into_iter()
                    .filter_map(
                        |internal_node| {
                            match internal_node.element {
                                PsdElement::Group(_) => Some(PsdNode::from(internal_node).emplace().into_shared()),
                                _ => None
                            }
                        }
                    )
                   .collect::<Vec<Instance<PsdNode>>>()
            }
        }
    }

    #[method]
    fn get_image(&mut self, #[base] owner: TRef<Reference>){
        match &self.internal_node.element {
            PsdElement::Layer(_) => {
                let internal_node = self.internal_node.clone();
                let width = i64::from(self.internal_node.tree.psd.width());
                let height = i64::from(self.internal_node.tree.psd.height());


                // Cleanup thread after execution of this method
                unsafe { owner.call_deferred("cleanup_thread", &[]); }

                let thread = std::thread::spawn(move || {
                    let image = Image::new();

                    if let PsdElement::Layer(layer) = &internal_node.element {
                        let data = PoolArray::from_vec(layer.rgba());
                        image.create_from_data(width, height, false, Image::FORMAT_RGBA8, data);
                    }

                    image
                });

                self.thread = Some(thread);
            },
            _ => {}
        }
    }

    #[method]
    fn _to_string(&self) -> String {
        match &self.node_type {
            PsdType::Layer => format!("Layer[{}]", self.internal_node.get_path().to_str().unwrap()),
            PsdType::Group => format!("Group[{}]", self.internal_node.get_path().to_str().unwrap())
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
                PsdElement::Layer(_) => PsdType::Layer
            },
            properties: match &internal_node.element {
                PsdElement::Layer(layer) => {
                    Some(
                        LayerProperties {
                            visible: layer.visible(),
                            opacity: layer.opacity(),
                            width: u32::try_from(layer.width()).unwrap(),
                            height: u32::try_from(layer.height()).unwrap(),
                            group_id: layer.parent_id()
                        }
                        )
                },
                _ => None
            },

            internal_node: Arc::from(internal_node),
            thread: None
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
                PsdElement::Layer(_) => PsdType::Layer
            },
            properties: match &internal_node.element {
                PsdElement::Layer(layer) => {
                    Some(
                        LayerProperties {
                            visible: layer.visible(),
                            opacity: layer.opacity(),
                            width: u32::try_from(layer.width()).unwrap(),
                            height: u32::try_from(layer.height()).unwrap(),
                            group_id: layer.parent_id()
                        }
                        )
                },
                _ => None
            },

            internal_node,
            thread: None
        }
    }
}



#[derive(FromVariant, ToVariant)]
#[variant(enum = "str")]
pub enum PsdType {
    Group,
    Layer
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
    pub group_id: Option<u32>
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

