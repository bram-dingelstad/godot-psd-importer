extends Control

func _ready():
	var file = File.new()
	file.open('res://test_import/test.psd', File.READ)
	var bytes = file.get_buffer(file.get_len())
	file.close()

	$PsdImporter.load(bytes)
	$PsdImporter.print_tree()

	var layer = $PsdImporter.get_node('/Tail/0/4')
	print(layer.name)
	print(layer.properties)
	print(layer.node_type)
	print(layer.get_rect2())
	var image_texture = ImageTexture.new()
	layer.get_image(true)
	var tuple = yield(layer, 'image')
	print(tuple)
	var image = tuple[0]
	var rect = tuple[1]

	print(rect)
	print(image)

	image_texture.create_from_image(image)

	$TextureRect.texture = image_texture

func _process(delta):
	$TextureRect.rect_rotation += 180 * delta
