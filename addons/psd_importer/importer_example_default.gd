extends 'res://addons/psd_importer/psd_importer_script.gd'

func import(plugin, importer, options, base_directory):
	print('Default importer, but as a custom script! :)')
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
