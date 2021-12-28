# Create output directory
mkdir -p output

# Get latest tags
git pull

# Get version number from tag name
export VERSION=$(git describe --tags $(git rev-list --tags --max-count=1))

# Update version number in Cargo.toml for build
# MacOS sed requires the weird empty string param
# Otherwise it returns `invalid command code C`
sed -i '' "s/0.0.0/$VERSION/g" Cargo.toml

# Build for Apple Silicon
cargo build --target aarch64-apple-darwin --release
cp target/aarch64-apple-darwin/release/logria output/logria-aarch64-apple-darwin

# Build for 64-bit Intel MacOS
cargo build --target x86_64-apple-darwin --release
cp target/x86_64-apple-darwin/release/logria output/logria-x86_64-apple-darwin

# Put the version number back
sed -i '' "s/$VERSION/0.0.0/g" Cargo.toml
