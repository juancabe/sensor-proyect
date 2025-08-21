# cross build --release --target=x86_64-unknown-linux-gnu && cp ./target/x86_64-unknown-linux-gnu/release/sensor-server ./bin/sensor-server-$(date +%s)
cross build --release --target=x86_64-unknown-linux-gnu && cp ./target/x86_64-unknown-linux-gnu/release/sensor-server ./bin/sensor-server
