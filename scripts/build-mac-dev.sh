cargo --features="single_dim2" fmt -- --config-path rustfmt.toml
cargo clippy --fix --allow-dirty
cargo build --features="single_dim2,simd-stable" --no-default-features
rm bin2d/addons/godot-rapier2d/bin/libgodot_rapier.macos.framework/libgodot_rapier.macos.dylib
rm /Users/dragosdaian/Documents/Godot-Physics-Tests/addons/godot-rapier2d/bin/libgodot_rapier.macos.framework/libgodot_rapier.macos.dylib
cp target/debug/libgodot_rapier.dylib bin2d/addons/godot-rapier2d/bin/libgodot_rapier.macos.framework/libgodot_rapier.macos.dylib
cp target/debug/libgodot_rapier.dylib /Users/dragosdaian/Documents/Godot-Physics-Tests/addons/godot-rapier2d/bin/libgodot_rapier.macos.framework/libgodot_rapier.macos.dylib


xattr -rc bin2d/addons/godot-rapier2d/bin/libgodot_rapier.macos.framework/libgodot_rapier.macos.dylib
xattr -rc /Users/dragosdaian/Documents/Godot-Physics-Tests/addons/godot-rapier2d/bin/libgodot_rapier.macos.framework/libgodot_rapier.macos.dylib
