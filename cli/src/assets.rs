use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../docker/configs/"]
pub struct ConfigAssets;

#[derive(RustEmbed)]
#[folder = "../"]
#[include = "docker-compose.yml"]
pub struct ComposeAsset;
