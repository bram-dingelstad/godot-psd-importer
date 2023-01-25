<p align="center">
<p align="center" style="font-size: 2rem" >PSD Importer for Godot</p>
<!-- Get a nice banner for this -->
<p align="center" style="font-style: italic">Speed up your import workflow ✨<br>An artisanally made PSD Importer for Godot 3.5, written in Rust.</p>
<p align="center"><a href="README.md#Getting-Started">✨ Getting Started</a> | <a href="README.md#Documentation">📚 Documentation</a> | <a href="README.md#Tutorial">🧑‍🏫 Tutorial</a>  | <a href="https://dingelstad.xyz/mastodon">🐘 Follow me</a> | <a href="https://store.steampowered.com/franchise/PlaceholderGameworks">🕹️ Buy our games</a></p>
</p> 

---

**PSD Importer** is a tool for speeding up your import workflow using Photoshop's PSD files. If you use a different program but export to PSD it should also work.
You can use this tool to mass-export all of your layers as seperate assets or as single files depending on import settings. 

If you have a developer on your team that can write code, you can write custom import code per-file so you can make your import specific to your needs!


## Getting started
This repo contains the code for the project, but you'll need to be at the release repository to download addon for Godot.
I'll try and have an addon available for easy download inside of Godoteasy download inside of Godot.


### Download from AssetLib
Unfortunately this option isn't available yet. Stay tuned!


### Clone this repository / [download the zip](https://github.com/bram-dingelstad/godot-psd-importer/archive/refs/heads/main.zip)

1. Extract the repository in a folder of your choice.
2. Import the project in Godot.
3. Run the scene to get a taste of the PSD Importer!
4. Move the addons folder to your Godot project.
5. Enable the plugin in your Project Settings.
6. Setup the importer using the [documentation](README.md#Documentation) or [tutorial](README.md#Tutorial)!

## Roadmap

There are a few things I want to tackle before calling a v1:

- [ ] A mascot for the page and project
- [ ] Releasing the addon on the AssetLib
- [ ] Resolve problems with `get_node` function and path formatting.
    - Currently there are small nuances that can cause certain paths to not behave in a manner the user wants.
- [ ] Add more sensible defaults for the default import script.
- [ ] Try to eliminate all of the `unwrap` calls or replace them with `expect`.
- [ ] Try to eliminate all `unsafe` code or properly document it.

### Feature requests

If for whatever reason the importer doesn't 100% fit your needs, feel free to [reach out to me](https://bram.dingelstad.works), but make sure you can't easily script this functionality using the custom script option.

## Getting Help

There are several places you can get help with PSD Importer for Godot & stay up to date with developments:

* [Reach out on Mastodon](https://dingelstad.xyz/mastodon) or [Twitter](https://dingelstad.xyz/twitter)
* Open an issue on [Github](https://github.com/bram-dingelstad/godot-psd-importer/issues)
* Email bram [at] dingelstad.works for more indepth questions or inqueries for consultancy. (You can also [hire me](https://hire.bram.dingelstad.works) for all your Godot needs)

## License

PSD Importer is available under the [MIT License](LICENSE.md). This means that you can use it in any commercial or noncommercial project. The only requirement is that you need to include attribution in your game's docs. A credit would be very, very nice, too, but isn't required. If you'd like to know more about what this license lets you do, tldrlegal.com have a [very nice write up about the MIT license](https://tldrlegal.com/license/mit-license) that you might find useful.

## Made by Bram Dingelstad & Placeholder Gameworks
PSD Importer was originally made for a game called CraftCraft, but we found it nice to share this technology with the broader world ✨

Support Placeholder Gameworks [by buying our games](https://store.steampowered.com/franchise/PlaceholderGameworks).

Originally written by [Bram Dingelstad](https://bram.dingelstad.works).

# Tutorial

Using the standard addon is quite simple. Simply enable the plugin after [installing](README.md#Getting-started), and you should see `*.psd` files pop up in Godot's file explorer.
Click on any of them and uncheck the "Dont import" option in the import tab. By default, the importer puts the files in the same directory as the import itself, so perhaps move it to a place where you want your final files to be.

## Writing a custom importer
Made a GDScript `.gd` file and start by extending `PsdImportScript`.
`PsdImportScript` has one method which is `import`. It has a few arguments:

1. `plugin` which is a reference to the EditorPlugin instance, used to skipping frames
* `importer` which is a pre-setup [`PsdImporter`](README.md#PsdImporter) instance with your PSD file pre-loaded.
* `options` which is a [Dictionary](https://docs.godotengine.org/en/3.5/classes/class_dictionary.html) holding different import options set by the user.
* `base_directory` which is a path [String](https://docs.godotengine.org/en/3.5/classes/class_string.html) where the PSD file is situated.

You can navigate the PSD by calling methods on the `importer` and its resulting [`PsdNode`](README.md#PsdNode) children, an example below is exporting all of the layers on the root of PSD that start with the letter A.


```gdscript
extends PsdImportScript

func import(plugin, importer, options, base_directory):
	# Go through all the children in the root of the PSD
	for child in importer.get_children():
		match child.node_type:
			'Layer':
				var name = child.name

				# When it the name of the layer doesn't start with A or a, skip this (continue)
				if not name.to_lower().starts_with('a'):
					continue

				var image_path = '%s%s.png' % [base_directory, name]

				# Wait a frame every 
				yield(plugin.get_tree(), 'idle_frame')

				# Get the image
				child.get_image()
				var image = yield(child, 'image') # And await its result
				if image:
					image.save_png(image_path)
					print('Imported "%s" to "%s"' % [name, image_path])
				else:
					printerr('Tried saving image to "%s" but something went wrong' % image_path)

	return OK
```
Another example, which is default import script, can be found [here](addon/psd-importer/importer_example_default.gd). It features a recursive exporter, which will go into groups and export sublayers.

# Documentation

## `PsdImporter` 
*Inherits from [Object](https://docs.godotengine.org/en/3.5/classes/class_object.html)*

Object for all interaction with PSD Importer.

### Description
This the main importer class that is the entry point for the importer code. You'll use this class to get access to the different layers and groups inside of the PSD.
Usually, most users should not have to interact with this class however.

Make sure that you `load` valid PSD data before resuming with other methods.


### Methods
| Return value                          | Method name                                                                                                                |
|---------------------------------------|----------------------------------------------------------------------------------------------------------------------------|
| void                                  | load( [PoolByteArray](https://docs.godotengine.org/en/3.5/classes/class_poolbytearray.html) psd_bytes )                 |
| void                                  | print_tree ( )                                                                                                             |
| [PsdNode](README.md#PsdNode)          | get_node ( [String](https://docs.godotengine.org/en/3.5/classes/class_string.html) path )                               |
| [Array<PsdNode>](README.md#PsdNode)   | get_children ( )                                                                                                           |
| [Array<PsdNode>](README.md#PsdNode)   | get_layers ( )                                                                                                             |
| [Array<PsdNode>](README.md#PsdNode)   | get_groups ( )                                                                                                             |


### Method Descriptions

* void **load** ( [PoolByteArray](https://docs.godotengine.org/en/3.5/classes/class_poolbytearray.html) psd_bytes ) 

  Loads in the PSD data into the class to do operations on. You can get `psd_bytes` by reading from a file using [`File`](https://docs.godotengine.org/en/3.5/classes/class_file.html).

* void **print_tree** ( )

  Will print an entire tree of the PSD with layers and groups into Godot's stdout (including your debug console).
  It can help you track down issues in your code or confirm if your file has been loaded correctly during debugging.

  An example of how that looks:

  ```
  [G] Group name
        [G] Another sub-group
            [L] A sub-sub layer (you get the idea)
        [L] Layer that is a child of "Group name" above
        [L] Another Layer
  [L] Layer on the root
  ```

* [PsdNode](README.md#PsdNode) **get_node** ( [String](https://docs.godotengine.org/en/3.5/classes/class_string.html) path )

  Get a single [`PsdNode`](README.md#PsdNode) (either a Layer or a Group) and return it. Works similarly to `Node.get_node`, should be familiar for most Godot programmers.

* [Array<PsdNode>](README.md#PsdNode) **get_children** ( )

  Gets all direct children as [`PsdNode`](README.md#PsdNode)s (either Layer or a Group) of the Root of the PSD file.

* [Array<PsdNode>](README.md#PsdNode) **get_layers** ( )

  Gets all direct children as [`PsdNode`](README.md#PsdNode)s of the Root of the PSD file, guaranteed to be Layers.

* [Array<PsdNode>](README.md#PsdNode) **get_groups** ( )

  Gets all direct children as [`PsdNode`](README.md#PsdNode)s of the Root of the PSD file, guaranteed to be Groups.

## `PsdNode` 
*Inherits from [Reference](https://docs.godotengine.org/en/3.5/classes/class_reference.html)*

A generic node representing either a Layer or a Group in a PSD file.

### Description

The result of any `get_node` call, either on `PsdImporter` or `PsdNode`. You can

### Properties
| Type                                                                                  | Property          | Default value                                                                                         |
|---------------------------------------------------------------------------------------|-------------------|-------------------------------------------------------------------------------------------------------|
| [String](https://docs.godotengine.org/en/3.5/classes/class_string.html)               | name              | Name of layer / group                                                                                 |
| [String](https://docs.godotengine.org/en/3.5/classes/class_string.html)               | path              | The path of the layer / group relative from the root                                                  |
| [String](https://docs.godotengine.org/en/3.5/classes/class_string.html)               | node_type         | The type of node either `"Layer"` or `"Group"` can be used to differentiate (e.g in match statements) |
| [Dictionary](https://docs.godotengine.org/en/3.5/classes/class_dictionary.html)       | properties        | Properties of the Layer if `node_type` is of type `"Layer"`.                                          |

### Methods
| Return value                          | Method name                                                                                                                |
|---------------------------------------|----------------------------------------------------------------------------------------------------------------------------|
| [PsdNode](README.md#PsdNode)          | get_node ( [String](https://docs.godotengine.org/en/3.5/classes/class_string.html) path )                                  |
| [Array<PsdNode>](README.md#PsdNode)   | get_children ( )                                                                                                           |
| [Array<PsdNode>](README.md#PsdNode)   | get_layers ( )                                                                                                             |
| [Array<PsdNode>](README.md#PsdNode)   | get_groups ( )                                                                                                             |
| void                                  | get_image ( )                                                                                                              |

### Signals

* image ( [Image](https://docs.godotengine.org/en/3.5/classes/class_image.html) image ) 

  Emitted when the node is done rendering an image after executing `get_image`

### Property Descriptions
* [String](https://docs.godotengine.org/en/3.5/classes/class_string.html) name 
  
  The name of the layer or group, as saved in the PSD file.

* [String](https://docs.godotengine.org/en/3.5/classes/class_string.html) path 
  
  The path to the layer or group, relative from the root of the PSD file.

* [String](https://docs.godotengine.org/en/3.5/classes/class_string.html) node_type 
  
  The type of the `PsdNode`, can either be `"Layer"` or `"Group"`. You can use this to guarantee the type in your code.

  An example, to make sure that you can execute `get_image`:

  ```gdscript
    var psd_node = importer.get_node('/Path/to/arbitrary/node')

    var image
    match psd_node.node_type:
        "Layer":
            psd_node.get_image()
            image = yield(psd_node, 'image')

        _:
            printerr("not a layer containing an image")
            
  ```

* [Dictionary](https://docs.godotengine.org/en/3.5/classes/class_dictionary.html) properties

  A dictionary containing information of the node in case `node_type` has the value of `"Layer"`. If the `node_type` has the value of `"Group"`, this property will be `null`.
 
  It has the following values:

  ```gdscript
  {
    visible: bool,
    pub opacity: int,
    pub width: int,
    pub height: int
  }
  ```

### Method Descriptions

* [PsdNode](README.md#PsdNode) **get_node** ( [String](https://docs.godotengine.org/en/3.5/classes/class_string.html) path )

  Get a single [`PsdNode`](README.md#PsdNode) (either a Layer or a Group) and return it. Works similarly to `Node.get_node`, should be familiar for most Godot programmers.

* [Array<PsdNode>](README.md#PsdNode) **get_children** ( )

  Gets all direct children as [`PsdNode`](README.md#PsdNode)s (either Layer or a Group) of the Root of the PSD file.

* [Array<PsdNode>](README.md#PsdNode) **get_layers** ( )

  Gets all direct children as [`PsdNode`](README.md#PsdNode)s of the Root of the PSD file, guaranteed to be Layers.

* [Array<PsdNode>](README.md#PsdNode) **get_groups** ( )

  Gets all direct children as [`PsdNode`](README.md#PsdNode)s of the Root of the PSD file, guaranteed to be Groups.

* void **get_image** ( )

  Starts converting the Layer into an [`Image`](https://docs.godotengine.org/en/3.5/classes/class_image.html), will only work if `node_type` is of type `"Layer"`.
  Result of this function is captured using the `image` signal.

  An example of how to get an image:

    ```gdscript
    var bytes = ... # PoolByteArray with PSD data

    # Create an importer
    var importer = PsdImporter.new()
    importer.load(bytes)

    # Get the psd node, which we assume to be of `node_type` "Layer"
    var psd_node = importer.get_node('/Path/to/a/Layer')
    
    # Start image rendering process
    psd_node.get_image()
    # Wait until it's done
    var image = yield(psd_node, 'image')

    ```
