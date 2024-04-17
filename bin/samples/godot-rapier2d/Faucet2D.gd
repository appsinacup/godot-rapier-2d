class_name Faucet2D
extends Node2D

@export var fluid : Fluid2D

var points: PackedVector2Array
var velocities: PackedVector2Array

func _ready():
	points = fluid.points
	velocities.resize(points.size())
	velocities.fill(Vector2(0, 980))

func _on_timer_timeout():
	if len(fluid.points) > 4000:
		return
	fluid.add_points_and_velocities(points, velocities)
	#print(len(fluid.points))
	#print(len(fluid.get_create_times()))
	#print(fluid.get_create_times())
