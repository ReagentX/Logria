# Create output directory
mkdir -p output

# Build for Apple Silicon
cargo build --target aarch64-apple-darwin --release
cp target/aarch64-apple-darwin/release/logria output/logria-aarch64-apple-darwin

# Build for 64-bit Intel MacOS
cargo build --target x86_64-apple-darwin --release
cp target/x86_64-apple-darwin/release/logria output/logria-x86_64-apple-darwin
