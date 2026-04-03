use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/configs/"]
pub struct ConfigAssets;

#[derive(RustEmbed)]
#[folder = "assets/"]
#[include = "docker-compose.yml"]
pub struct ComposeAsset;
