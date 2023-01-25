tool
extends EditorPlugin

var import_plugin

func _enter_tree():
	import_plugin = ImportPlugin.new(self)
	add_import_plugin(import_plugin)


func _exit_tree():
	remove_import_plugin(import_plugin)
	import_plugin = null

class ImportPlugin extends EditorImportPlugin:
	var plugin

	enum Presets {
		DONT_IMPORT,
		# IMPORT_AS_SINGLE_TEXTURE,
		IMPORT_AS_SEPERATE_TEXTURES,
		# IMPORT_ALL_AS_SINGLE_TEXTURE,
		# IMPORT_ALL_AS_SEPERATE_TEXTURES
	}

	func _init(_plugin):
		plugin = _plugin
		print('Initialized PSD Importer plugin')


	func get_importer_name():
		return 'bram.dingelstad.works.psd_importer'


	func get_visible_name():
		return 'PSD Importer'


	func get_recognized_extensions():
		return ['psd']


	func get_save_extension():
		return 'tres'


	func get_resource_type():
		return 'Resource'


	func get_preset_count():
		return Presets.size()


	func get_preset_name(preset):
		match preset:
			Presets.DONT_IMPORT:
				return 'Dont import'

			# Presets.IMPORT_AS_SINGLE_TEXTURE:
			# 	return 'Import as single texture'
            #
			Presets.IMPORT_AS_SEPERATE_TEXTURES:
				return 'Import as seperate textures'
            #
			# Presets.IMPORT_ALL_AS_SINGLE_TEXTURE:
			# 	return 'Import all as single texture'
            #
			# Presets.IMPORT_ALL_AS_SEPERATE_TEXTURES:
			# 	return 'Import all as seperate textures'

			_:
				return 'Unknown'


	func get_import_options(preset):
		return [
			{
				name = 'dont_import',
				default_value = preset == Presets.DONT_IMPORT
			},
			# {
			# 	name = 'import_only_visible',
			# 	default_value = [
			# 		Presets.IMPORT_ALL_AS_SINGLE_TEXTURE,
			# 		Presets.IMPORT_ALL_AS_SEPERATE_TEXTURES
			# 	].has(preset)
			# },
			# {
			# 	name = 'single_import',
			# 	default_value = [
			# 		Presets.IMPORT_AS_SINGLE_TEXTURE,
			# 		Presets.IMPORT_ALL_AS_SINGLE_TEXTURE
			# 	].has(preset)
			# },
			{
				name = 'custom_import_script',
				default_value = '',
				property_hint = PROPERTY_HINT_FILE,
				hint = '*.gd'
			}
		]


	func get_option_visibility(option, options):
		return true


	func import(source_file, save_path, options, platform_variants, gen_files):
		print('Importing a PSD')

		var base_directory = source_file.trim_suffix(source_file.get_file())

		if options['dont_import']: return FAILED
		var importer = load('res://PsdImporter.gdns').new()

		var file = File.new()
		file.open(source_file, File.READ)
		var bytes = file.get_buffer(file.get_len())
		file.close()

		importer.load(bytes)

		var script
		if options['custom_import_script'] \
				and ResourceLoader.exists(options['custom_import_script']):
			script = load(options['custom_import_script']).new()

			if not script.has_method('import'):
				printerr('Your custom script doesn\'t extend PsdImportScript found in res://addons/psd_importer/psd_importer_script.gd')
				return ERR_UNCONFIGURED

		else:
			script = DefaultPsdImportScript.new()

		if not script:
			return FAILED

		script.import(plugin, importer, options, base_directory)

		print('Done importing!')

		# Save a stub
		return ResourceSaver.save('%s.%s' % [save_path, get_save_extension()], Resource.new())


class PsdImportScript:
	func import(plugin, importer, options, base_directory):
		return ERR_UNCONFIGURED


class DefaultPsdImportScript extends PsdImportScript:
	func import(plugin, importer, options, base_directory):
		import_children(plugin, importer.get_children(), base_directory)

		return OK


	func import_children(plugin, children, base_directory):
		var directory = Directory.new()

		for child in children:
			match child.node_type:
				'Group':
					import_children(plugin, child.get_children(), base_directory)

				'Layer':
					var directory_path = child.path.trim_suffix(child.path.get_file())
					var image_path = '%s%s.png' % [base_directory, child.path.trim_prefix('/')]
					# Save node's name for later, because it'll be lost in the thread :/
					var name = child.name

					directory.make_dir_recursive('%s%s' % [base_directory, directory_path])
					
					# Wait a frame every 
					yield(plugin.get_tree(), 'idle_frame')
					child.get_image()

					var image = yield(child, 'image')
					if image:
						image.save_png(image_path)
						print('Imported "%s" to "%s"' % [name, image_path])
					else:
						printerr('Tried saving image to "%s" but something went wrong' % image_path)

		return OK
