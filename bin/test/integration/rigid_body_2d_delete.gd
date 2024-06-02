extends RigidBody2D


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	position = Vector2(randf(), randf())


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	if randf() < 0.02:
		queue_free()
