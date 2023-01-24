extends Control

func _ready():
	var file = File.new()
	file.open('res://test_import/test.psd', File.READ)
	var bytes = file.get_buffer(file.get_len())
	file.close()

	$PsdImporter.load(bytes)
	$PsdImporter.print_tree()

	var layer = $PsdImporter.get_node('/Expression/Festive/Happy')
	print(layer.name)
	print(layer.properties)
	print(layer.node_type)
	var image_texture = ImageTexture.new()
	layer.get_image()
	var image = yield(layer, 'image')
	image_texture.create_from_image(image)

	$TextureRect.texture = image_texture

func _process(delta):
	$TextureRect.rect_rotation += 180 * delta
