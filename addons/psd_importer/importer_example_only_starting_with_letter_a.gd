extends 'res://addons/psd_importer/psd_importer_script.gd'

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

